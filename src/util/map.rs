use base64::prelude::*;
use quick_xml::{events::Event, name::QName, Reader, Writer};
use serde::forward_to_deserialize_any;

use crate::error::ParseError;
use crate::util::{ReaderExt, WriterExt};
use crate::{Error, Result};

use super::{ValueDeserializer, ValueSerializer};

#[doc(hidden)]
pub struct MapSerializer<'a, W>
where
    W: std::io::Write,
{
    writer: &'a mut Writer<W>,
}

impl<'a, W> MapSerializer<'a, W>
where
    W: std::io::Write,
{
    pub fn new(writer: &'a mut Writer<W>) -> Result<Self> {
        let ret = MapSerializer { writer };
        ret.writer.write_start_tag("value")?;
        ret.writer.write_start_tag("struct")?;
        Ok(ret)
    }
}

impl<'a, W> serde::ser::SerializeMap for MapSerializer<'a, W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        self.writer.write_start_tag("member")?;
        key.serialize(MapKeySerializer::new(self.writer))?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(ValueSerializer::new(self.writer))?;
        self.writer.write_end_tag("member")?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.write_end_tag("struct")?;
        self.writer.write_end_tag("value")?;
        Ok(())
    }
}

impl<'a, W> serde::ser::SerializeStruct for MapSerializer<'a, W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeMap::serialize_key(self, key)?;
        serde::ser::SerializeMap::serialize_value(self, value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeMap::end(self)
    }
}

impl<'a, W> serde::ser::SerializeStructVariant for MapSerializer<'a, W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeMap::serialize_key(self, key)?;
        serde::ser::SerializeMap::serialize_value(self, value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeMap::end(self)
    }
}

#[doc(hidden)]
pub struct MapKeySerializer<'a, W>
where
    W: std::io::Write,
{
    writer: &'a mut Writer<W>,
}

impl<'a, W> MapKeySerializer<'a, W>
where
    W: std::io::Write,
{
    fn new(writer: &'a mut Writer<W>) -> Self {
        MapKeySerializer { writer }
    }
}

impl<'a, W> serde::Serializer for MapKeySerializer<'a, W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.writer
            .write_safe_tag("name", if v { "1" } else { "0" })?;
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
        self.writer.write_safe_tag("name", &v.to_string())
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
        self.writer.write_safe_tag("name", &v.to_string())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.writer.write_safe_tag("name", &v.to_string())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.writer.write_safe_tag("name", &v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.writer.write_tag("name", v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.writer
            .write_safe_tag("name", &BASE64_STANDARD.encode(v))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T>(self, _v: &T) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(key_must_be_a_string())
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
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(key_must_be_a_string())
    }
}

fn key_must_be_a_string() -> Error {
    Error::from(ParseError::KeyMustBeString)
}

#[doc(hidden)]
pub struct MapDeserializer<'a, 'r> {
    reader: &'a mut Reader<&'r [u8]>,
    end: &'a [u8],
}

impl<'a, 'r> MapDeserializer<'a, 'r> {
    pub fn new(reader: &'a mut Reader<&'r [u8]>, end: &'a [u8]) -> Self {
        MapDeserializer { reader, end }
    }
}

impl<'de, 'a, 'r> serde::de::MapAccess<'de> for MapDeserializer<'a, 'r> {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.reader.read_event() {
            // The base case is that we found a closing tag for the tag we were
            // looking for.
            Ok(Event::End(ref e)) if e.name() == QName(self.end) => Ok(None),

            // If we got a member start tag, we know there's a key and value
            // coming.
            Ok(Event::Start(ref e)) if e.name() == QName(b"member") => {
                self.reader.expect_tag(QName(b"name"))?;
                Ok(Some(seed.deserialize(MapKeyDeserializer::new(
                    self.reader,
                    b"name",
                ))?))
            }

            // Any other event or error is unexpected and is an actual error.
            Ok(e) => Err(ParseError::UnexpectedEvent(format!("map key read: {:?}", e)).into()),
            Err(e) => Err(ParseError::from(e).into()),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let ret = match self.reader.read_event() {
            Ok(Event::Start(ref e)) if e.name() == QName(b"value") => {
                Ok(seed.deserialize(ValueDeserializer::new(self.reader)?)?)
            }
            Ok(e) => Err(ParseError::UnexpectedEvent(format!("map value read: {:?}", e)).into()),
            Err(e) => Err(ParseError::from(e).into()),
        };

        self.reader
            .read_to_end(QName(b"member"))
            .map_err(ParseError::from)?;

        ret
    }
}

#[doc(hidden)]
pub struct MapKeyDeserializer<'a, 'r> {
    reader: &'a mut Reader<&'r [u8]>,
    end: &'a [u8],
}

impl<'a, 'r> MapKeyDeserializer<'a, 'r> {
    pub fn new(reader: &'a mut Reader<&'r [u8]>, end: &'a [u8]) -> Self {
        MapKeyDeserializer { reader, end }
    }
}

impl<'de, 'a, 'r> serde::Deserializer<'de> for MapKeyDeserializer<'a, 'r> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_string(
            self.reader
                .read_text(QName(self.end))
                .map_err(ParseError::from)?
                .into(),
        )
    }

    forward_to_deserialize_any!(
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any option
    );
}
