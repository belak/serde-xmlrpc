use base64::{decode as base64_decode, encode as base64_encode};
use quick_xml::{events::Event, Reader, Writer};
use serde::forward_to_deserialize_any;

use crate::error::ParseError;
use crate::util::{ReaderExt, WriterExt};
use crate::{Error, Result};

use super::{MapDeserializer, MapSerializer};
use super::{SeqDeserializer, SeqSerializer};

#[doc(hidden)]
pub struct Deserializer<B>
where
    B: std::io::BufRead,
{
    pub(crate) reader: Reader<B>,
    buf: Vec<u8>,
}

impl<B> Deserializer<B>
where
    B: std::io::BufRead,
{
    pub fn new(reader: Reader<B>) -> Result<Self>
    where
        B: std::io::BufRead,
    {
        let mut ret = Deserializer {
            reader,
            buf: Vec::new(),
        };
        ret.reader.expect_tag(b"value", &mut ret.buf)?;
        Ok(ret)
    }

    #[doc(hidden)]
    pub fn new_without_value(reader: Reader<B>) -> Self
    where
        B: std::io::BufRead,
    {
        Deserializer {
            reader,
            buf: Vec::new(),
        }
    }

    pub fn into_inner(self) -> Reader<B>
    where
        B: std::io::BufRead,
    {
        self.reader
    }
}

impl<'de, 'a, B> serde::Deserializer<'de> for &'a mut Deserializer<B>
where
    B: std::io::BufRead,
{
    type Error = Error;

    #[allow(clippy::cognitive_complexity)]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let ret =
            match self.reader.read_event(&mut self.buf) {
                // If we got text, this is a String value. This is an edge case
                // because it's valid to have a string value without the inner
                // "string" tag.
                Ok(Event::Text(e)) => visitor.visit_string::<Self::Error>(
                    e.unescape_and_decode(&self.reader)
                        .map_err(ParseError::from)?,
                )?,

                // Alternatively, if we got the matching end tag, this is an empty
                // string value. Note that we need to return early here so the end
                // doesn't try to read the closing tag.
                Ok(Event::End(ref e)) if e.name() == b"value" => return visitor.visit_str(""),

                Ok(Event::Start(ref e)) => match e.name() {
                    b"int" => {
                        let mut buf = Vec::new();
                        let text = self
                            .reader
                            .read_text(e.name(), &mut buf)
                            .map_err(ParseError::from)?;
                        visitor.visit_i64::<Self::Error>(text.parse().map_err(ParseError::from)?)?
                    }

                    b"boolean" => {
                        let mut buf = Vec::new();
                        let text = self
                            .reader
                            .read_text(e.name(), &mut buf)
                            .map_err(ParseError::from)?;
                        match text.as_ref() {
                            "1" => visitor.visit_bool::<Self::Error>(true),
                            "0" => visitor.visit_bool::<Self::Error>(false),
                            _ => return Err(ParseError::BooleanDecodeError(text).into()),
                        }?
                    }

                    b"string" => {
                        let mut buf = Vec::new();
                        visitor.visit_string::<Self::Error>(
                            self.reader
                                .read_text(e.name(), &mut buf)
                                .map_err(ParseError::from)?,
                        )?
                    }

                    b"double" => {
                        let mut buf = Vec::new();
                        let text = self
                            .reader
                            .read_text(e.name(), &mut buf)
                            .map_err(ParseError::from)?;
                        visitor.visit_f64::<Self::Error>(text.parse().map_err(ParseError::from)?)?
                    }

                    b"dateTime.iso8601" => {
                        unimplemented!();
                    }

                    b"base64" => {
                        let mut buf = Vec::new();
                        let text = self
                            .reader
                            .read_text(e.name(), &mut buf)
                            .map_err(ParseError::from)?;
                        visitor.visit_byte_buf::<Self::Error>(
                            base64_decode(&text).map_err(ParseError::from)?,
                        )?
                    }

                    b"struct" => visitor.visit_map(MapDeserializer::new(self, b"struct"))?,

                    b"array" => {
                        visitor.visit_seq(SeqDeserializer::new(self, b"data", Some(b"array"))?)?
                    }
                    b"nil" => {
                        let mut buf = Vec::new();
                        self.reader
                            .read_to_end(e.name(), &mut buf)
                            .map_err(ParseError::from)?;
                        visitor.visit_unit::<Self::Error>()?
                    }

                    _ => return Err(ParseError::UnexpectedTag(
                        String::from_utf8_lossy(e.name()).into(),
                        "one of int|boolean|string|double|dateTime.iso8601|base64|struct|array|nil"
                            .into(),
                    )
                    .into()),
                },

                // Possible error states
                Ok(Event::Eof) => {
                    return Err(ParseError::UnexpectedEOF(
                        "one of int|boolean|string|double|dateTime.iso8601|base64|struct|array|nil"
                            .into(),
                    )
                    .into())
                }

                Ok(_) => {
                    return Err(ParseError::UnexpectedEvent(
                        "one of int|boolean|string|double|dateTime.iso8601|base64|struct|array|nil"
                            .into(),
                    )
                    .into())
                }

                Err(e) => return Err(ParseError::from(e).into()),
            };

        self.reader
            .read_to_end(b"value", &mut self.buf)
            .map_err(ParseError::from)?;

        Ok(ret)
    }

    forward_to_deserialize_any!(
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any option
    );
}

#[doc(hidden)]
pub struct Serializer<'a, W>
where
    W: std::io::Write,
{
    writer: &'a mut Writer<W>,
}

impl<'a, W> Serializer<'a, W>
where
    W: std::io::Write,
{
    pub fn new(writer: &'a mut Writer<W>) -> Self {
        Serializer { writer }
    }
}

impl<'a, W> serde::Serializer for Serializer<'a, W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SeqSerializer<'a, W>;
    type SerializeTuple = SeqSerializer<'a, W>;
    type SerializeTupleStruct = SeqSerializer<'a, W>;
    type SerializeTupleVariant = SeqSerializer<'a, W>;
    type SerializeMap = MapSerializer<'a, W>;
    type SerializeStruct = MapSerializer<'a, W>;
    type SerializeStructVariant = MapSerializer<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.writer.write_start_tag(b"value")?;
        self.writer
            .write_safe_tag(b"boolean", if v { "1" } else { "0" })?;
        self.writer.write_end_tag(b"value")?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.writer.write_start_tag(b"value")?;
        self.writer.write_safe_tag(b"int", &v.to_string())?;
        self.writer.write_end_tag(b"value")?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.writer.write_start_tag(b"value")?;
        self.writer.write_safe_tag(b"int", &v.to_string())?;
        self.writer.write_end_tag(b"value")?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.writer.write_start_tag(b"value")?;
        self.writer.write_safe_tag(b"double", &v.to_string())?;
        self.writer.write_end_tag(b"value")?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.writer.write_start_tag(b"value")?;
        self.writer.write_tag(b"string", &v.to_string())?;
        self.writer.write_end_tag(b"value")?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.writer.write_start_tag(b"value")?;
        self.writer.write_tag(b"string", v)?;
        self.writer.write_end_tag(b"value")?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.writer.write_start_tag(b"value")?;
        self.writer.write_safe_tag(b"string", &base64_encode(v))?;
        self.writer.write_end_tag(b"value")?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, v: &T) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize,
    {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.writer.write_start_tag(b"value")?;
        self.writer.write(b"<nil />").map_err(ParseError::from)?;
        self.writer.write_end_tag(b"value")?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize,
    {
        unimplemented!();
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Self::SerializeSeq::new(self.writer)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Self::SerializeMap::new(self.writer)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_map(Some(len))
    }
}

#[doc(hidden)]
#[allow(dead_code)]
pub fn from_str<T>(val: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let mut reader = Reader::from_str(val);
    reader.expand_empty_elements(true);
    reader.trim_text(true);

    let mut deserializer = Deserializer::new(reader)?;
    T::deserialize(&mut deserializer)
}

#[doc(hidden)]
#[allow(dead_code)]
pub fn to_string<T>(val: &T) -> Result<String>
where
    T: ?Sized + serde::Serialize,
{
    let mut writer = Writer::new(Vec::new());
    let ser = Serializer::new(&mut writer);
    val.serialize(ser)?;
    Ok(String::from_utf8(writer.into_inner()).map_err(ParseError::from)?)
}

#[cfg(test)]
mod tests {
    use super::{from_str, to_string};

    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Test {
        hello: String,
    }

    #[test]
    fn test_from_str() {
        let x: i32 = from_str("<value><int>42</int></value>").unwrap();
        assert_eq!(x, 42);

        let x: bool = from_str("<value><boolean>1</boolean></value>").unwrap();
        assert_eq!(x, true);

        let x: Vec<i32> = from_str("<value><array><data><value><int>1</int></value><value><int>2</int></value><value><int>3</int></value></data></array></value>").unwrap();
        assert_eq!(x, vec![1, 2, 3]);

        let x: Test = from_str("<value><struct><member><name>hello</name><value><string>world</string></value></member></struct></value>").unwrap();
        assert_eq!(
            x,
            Test {
                hello: "world".to_string()
            }
        );
    }

    #[test]
    fn test_to_string() {
        assert_eq!(&to_string(&42).unwrap(), "<value><int>42</int></value>");

        assert_eq!(
            &to_string(&true).unwrap(),
            "<value><boolean>1</boolean></value>"
        );

        assert_eq!(
            &to_string(&vec![1, 2, 3]).unwrap(),
            "<value><array><data><value><int>1</int></value><value><int>2</int></value><value><int>3</int></value></data></array></value>"
        );

        assert_eq!(
            &to_string(&Test {
                hello: "world".to_string()
            }).unwrap(),
            "<value><struct><member><name>hello</name><value><string>world</string></value></member></struct></value>",
        )
    }
}
