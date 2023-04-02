use quick_xml::{events::Event, Writer};

use crate::error::ParseError;
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
        ret.writer.write_start_tag(b"value")?;
        ret.writer.write_start_tag(b"array")?;
        ret.writer.write_start_tag(b"data")?;
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
        self.writer.write_end_tag(b"data")?;
        self.writer.write_end_tag(b"array")?;
        self.writer.write_end_tag(b"value")?;

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
pub struct SeqDeserializer<'a, R>
where
    R: std::io::BufRead,
{
    inner: &'a mut ValueDeserializer<R>,
    buf: Vec<u8>,
    end: &'a [u8],
    end_maybe: Option<&'a [u8]>,
}

impl<'a, R> SeqDeserializer<'a, R>
where
    R: std::io::BufRead,
{
    pub fn new(
        inner: &'a mut ValueDeserializer<R>,
        end: &'a [u8],
        end_maybe: Option<&'a [u8]>,
    ) -> Result<Self> {
        let mut ret = SeqDeserializer {
            inner,
            buf: Vec::new(),
            end,
            end_maybe,
        };

        ret.inner.reader.expect_tag(ret.end, &mut ret.buf)?;

        Ok(ret)
    }
}

impl<'de, 'a, R> serde::de::SeqAccess<'de> for SeqDeserializer<'a, R>
where
    R: std::io::BufRead,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.inner.reader.read_event(&mut self.buf) {
            Ok(Event::End(ref e)) if e.name() == self.end => {
                if let Some(end) = self.end_maybe {
                    self.inner
                        .reader
                        .read_to_end(end, &mut self.buf)
                        .map_err(ParseError::from)?;
                }
                Ok(None)
            }
            Ok(Event::Start(ref e)) if e.name() == b"value" => {
                Ok(Some(seed.deserialize(&mut *self.inner)?))
            }
            Ok(_) => Err(ParseError::UnexpectedEvent("one of value".to_string()).into()),
            Err(e) => Err(ParseError::from(e).into()),
        }
    }
}
