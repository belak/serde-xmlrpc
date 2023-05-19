use quick_xml::Reader;
use quick_xml::{events::Event, name::QName, Writer};

use crate::error::DecodingError;
use crate::util::{ReaderExt, WriterExt};
use crate::{Error, Result};

use super::{ValueDeserializer, ValueSerializer};

#[doc(hidden)]
pub struct SeqSerializer<'a, W>
where
    W: std::io::Write,
{
    writer: &'a mut Writer<W>,
}

impl<'a, W> SeqSerializer<'a, W>
where
    W: std::io::Write,
{
    pub fn new(writer: &'a mut Writer<W>) -> Result<Self> {
        let ret = SeqSerializer { writer };
        ret.writer.write_start_tag("value")?;
        ret.writer.write_start_tag("array")?;
        ret.writer.write_start_tag("data")?;
        Ok(ret)
    }
}

impl<'a, W> serde::ser::SerializeSeq for SeqSerializer<'a, W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(ValueSerializer::new(self.writer))
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.write_end_tag("data")?;
        self.writer.write_end_tag("array")?;
        self.writer.write_end_tag("value")?;

        Ok(())
    }
}
impl<'a, W> serde::ser::SerializeTuple for SeqSerializer<'a, W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'a, W> serde::ser::SerializeTupleStruct for SeqSerializer<'a, W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'a, W> serde::ser::SerializeTupleVariant for SeqSerializer<'a, W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

#[doc(hidden)]
pub struct SeqDeserializer<'a, 'r> {
    reader: &'a mut Reader<&'r [u8]>,
    end: QName<'a>,
    end_maybe: Option<QName<'a>>,
}

impl<'a, 'r> SeqDeserializer<'a, 'r> {
    pub fn new(
        reader: &'a mut Reader<&'r [u8]>,
        end: QName<'a>,
        end_maybe: Option<QName<'a>>,
    ) -> Result<Self> {
        let ret = SeqDeserializer {
            reader,
            end,
            end_maybe,
        };

        ret.reader.expect_tag(ret.end)?;

        Ok(ret)
    }
}

impl<'de, 'a, 'r> serde::de::SeqAccess<'de> for SeqDeserializer<'a, 'r> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.reader.read_event() {
            Ok(Event::End(ref e)) if e.name() == self.end => {
                if let Some(end) = self.end_maybe {
                    self.reader.read_to_end(end).map_err(DecodingError::from)?;
                }
                Ok(None)
            }
            Ok(Event::Start(ref e)) if e.name() == QName(b"value") => Ok(Some(
                seed.deserialize(ValueDeserializer::new(self.reader)?)?,
            )),
            Ok(_) => Err(DecodingError::UnexpectedEvent("one of value".to_string()).into()),
            Err(e) => Err(DecodingError::from(e).into()),
        }
    }
}
