use base64::prelude::*;
use quick_xml::{events::Event, name::QName, Reader};
use serde::forward_to_deserialize_any;
use std::convert::TryInto;

use crate::error::DecodingError;
use crate::xml_ext::ReaderExt;
use crate::{Error, Result};

#[doc(hidden)]
pub struct Deserializer<'a, 'r> {
    pub(crate) reader: &'a mut Reader<&'r [u8]>,
}

impl<'a, 'r> Deserializer<'a, 'r> {
    pub fn new(reader: &'a mut Reader<&'r [u8]>) -> Result<Self> {
        let ret = Deserializer { reader };
        Ok(ret)
    }
}

impl<'de, 'a, 'r> serde::Deserializer<'de> for Deserializer<'a, 'r> {
    type Error = Error;

    #[allow(clippy::cognitive_complexity)]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let ret = match self.reader.read_event() {
            // If we got text, this is a String value. This is an edge case
            // because it's valid to have a string value without the inner
            // "string" tag.
            Ok(Event::Text(e)) => visitor.visit_str::<Self::Error>(
                e.unescape()
                    .map_err(DecodingError::from)?.as_ref(),
            )?,

            // Alternatively, if we got the matching end tag, this is an empty
            // string value. Note that we need to return early here so the end
            // doesn't try to read the closing tag.
            Ok(Event::End(ref e)) if e.name() == QName(b"value") => return visitor.visit_str(""),

            Ok(Event::Start(ref e)) => match e.name() {
                QName(b"int") | QName(b"i4") | QName(b"i8") => {
                    let text = self
                        .reader
                        .read_text(e.name())
                        .map_err(DecodingError::from)?;

                    let val: i64 = text.parse().map_err(DecodingError::from)?;

                    if let Ok(val) = val.try_into() {
                        visitor.visit_i8::<Self::Error>(val)?
                    } else if let Ok(val) = val.try_into() {
                        visitor.visit_i16::<Self::Error>(val)?
                    } else if let Ok(val) = val.try_into() {
                        visitor.visit_i32::<Self::Error>(val)?
                    } else {
                        visitor.visit_i64::<Self::Error>(val)?
                    }
                }

                QName(b"boolean") => {
                    let text = self
                        .reader
                        .read_text(e.name())
                        .map_err(DecodingError::from)?;
                    match text.as_ref() {
                        "1" => visitor.visit_bool::<Self::Error>(true),
                        "0" => visitor.visit_bool::<Self::Error>(false),
                        _ => return Err(DecodingError::BooleanDecodeError(text.into_owned()).into()),
                    }?
                }

                QName(b"string") => {
                    visitor.visit_str::<Self::Error>(
                        self.reader
                            .read_text(e.name())
                            .map_err(DecodingError::from)?.as_ref(),
                    )?
                }

                QName(b"double") => {
                    let text = self
                        .reader
                        .read_text(e.name())
                        .map_err(DecodingError::from)?;
                    visitor.visit_f64::<Self::Error>(text.parse().map_err(DecodingError::from)?)?
                }

                QName(b"dateTime.iso8601") => {
                    visitor.visit_str::<Self::Error>(
                        self.reader
                            .read_text(e.name())
                            .map_err(DecodingError::from)?.as_ref(),
                    )?
                }

                QName(b"base64") => {
                    let text = self
                        .reader
                        .read_text(e.name())
                        .map_err(DecodingError::from)?;
                    visitor.visit_byte_buf::<Self::Error>(
                       BASE64_STANDARD.decode(text.as_ref()).map_err(DecodingError::from)?,
                    )?
                }

                QName(b"struct") => visitor.visit_map(MapDeserializer::new(self.reader))?,

                QName(b"array") => {
                    visitor.visit_seq(SeqDeserializer::new(self.reader, QName(b"data"), Some(QName(b"array")))?)?
                }

                QName(b"nil") => {
                    self.reader
                        .read_to_end(e.name())
                        .map_err(DecodingError::from)?;
                    visitor.visit_unit::<Self::Error>()?
                }

                _ => {
                    return Err(DecodingError::UnexpectedTag(
                        String::from_utf8_lossy(e.name().into_inner()).into(),
                        "one of int|i4|i8|boolean|string|double|dateTime.iso8601|base64|struct|array|nil"
                            .into(),
                    )
                    .into())
                }
            },

            // Possible error states
            Ok(Event::Eof) => {
                return Err(DecodingError::UnexpectedEOF(
                    "one of int|i4|i8|boolean|string|double|dateTime.iso8601|base64|struct|array|nil"
                        .into(),
                )
                .into())
            }

            Ok(_) => {
                return Err(DecodingError::UnexpectedEvent(
                    "one of int|i4|i8|boolean|string|double|dateTime.iso8601|base64|struct|array|nil"
                        .into(),
                )
                .into())
            }

            Err(e) => return Err(DecodingError::from(e).into()),
        };

        self.reader
            .read_to_end(QName(b"value"))
            .map_err(DecodingError::from)?;

        Ok(ret)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        // Clone the reader and walk through it - if we get starting nil tag, we
        // read to the end of the nil tag, replace the inner reader, and call
        // visit_none. Otherwise, we defer to visitor.visit_some(self).
        //
        // We clone the reader (rather than using self.reader directly) as a way
        // to "peek" at the next event.
        let mut reader = self.reader.clone();

        if let Ok(Event::Start(ref e)) = reader.read_event() {
            if e.name() == QName(b"nil") {
                reader.read_to_end(e.name()).map_err(DecodingError::from)?;
                *self.reader = reader;
                return visitor.visit_none::<Self::Error>();
            }
        }

        visitor.visit_some(self)
    }

    forward_to_deserialize_any!(
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    );
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
            Ok(Event::Start(ref e)) if e.name() == QName(b"value") => {
                Ok(Some(seed.deserialize(Deserializer::new(self.reader)?)?))
            }
            Ok(_) => Err(DecodingError::UnexpectedEvent("one of value".to_string()).into()),
            Err(e) => Err(DecodingError::from(e).into()),
        }
    }
}

#[doc(hidden)]
pub struct MapDeserializer<'a, 'r> {
    reader: &'a mut Reader<&'r [u8]>,
}

impl<'a, 'r> MapDeserializer<'a, 'r> {
    pub fn new(reader: &'a mut Reader<&'r [u8]>) -> Self {
        MapDeserializer { reader }
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
            Ok(Event::End(ref e)) if e.name() == QName(b"struct") => Ok(None),

            // If we got a member start tag, we know there's a key and value
            // coming.
            Ok(Event::Start(ref e)) if e.name() == QName(b"member") => {
                self.reader.expect_tag(QName(b"name"))?;
                Ok(Some(
                    seed.deserialize(MapKeyDeserializer::new(self.reader))?,
                ))
            }

            // Any other event or error is unexpected and is an actual error.
            Ok(e) => Err(DecodingError::UnexpectedEvent(format!("map key read: {:?}", e)).into()),
            Err(e) => Err(DecodingError::from(e).into()),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let ret = match self.reader.read_event() {
            Ok(Event::Start(ref e)) if e.name() == QName(b"value") => {
                Ok(seed.deserialize(Deserializer::new(self.reader)?)?)
            }
            Ok(e) => Err(DecodingError::UnexpectedEvent(format!("map value read: {:?}", e)).into()),
            Err(e) => Err(DecodingError::from(e).into()),
        };

        self.reader
            .read_to_end(QName(b"member"))
            .map_err(DecodingError::from)?;

        ret
    }
}

#[doc(hidden)]
pub struct MapKeyDeserializer<'a, 'r> {
    reader: &'a mut Reader<&'r [u8]>,
}

impl<'a, 'r> MapKeyDeserializer<'a, 'r> {
    pub fn new(reader: &'a mut Reader<&'r [u8]>) -> Self {
        MapKeyDeserializer { reader }
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
                .read_text(QName(b"name"))
                .map_err(DecodingError::from)?
                .into(),
        )
    }

    forward_to_deserialize_any!(
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any option
    );
}
