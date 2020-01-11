use std::collections::BTreeMap;
use std::convert::TryFrom;

use base64::{decode as decode_base64, encode as encode_base64};
use iso8601::{datetime as parse_datetime, DateTime};
use quick_xml::events::Event;
use quick_xml::{Reader, Writer};

use crate::error::{EncodingError, Error, Fault, ParseError, Result};
use crate::utils::{ReaderExt, WriterExt};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// A 32-bit signed integer (`<i4>` or `<int>`).
    Int(i32),
    /// A 64-bit signed integer (`<i8>`).
    Int64(i64),
    /// A boolean value (`<boolean>`, 0 == `false`, 1 == `true`).
    Bool(bool),
    /// A string (`<string>`).
    String(String),
    /// A double-precision IEEE 754 floating point number (`<double>`).
    Double(f64),
    /// An ISO 8601 formatted date/time value (`<dateTime.iso8601>`).
    DateTime(DateTime),
    /// Base64-encoded binary data (`<base64>`).
    Base64(Vec<u8>),
    /// A mapping of named values (`<struct>`).
    Struct(BTreeMap<String, Value>),
    /// A list of arbitrary (heterogeneous) values (`<array>`).
    Array(Vec<Value>),
    /// The empty (Unit) value (`<nil/>`).
    Nil,
}

// Public API definitions
impl Value {
    pub fn stringify(&self) -> Result<String> {
        let mut buf = Vec::new();
        let mut writer = Writer::new(&mut buf);

        writer.write_start_tag(b"value")?;

        match *self {
            Value::Int(i) => {
                writer.write_safe_tag(b"i4", &i.to_string()[..])?;
            }
            Value::Int64(i) => {
                writer.write_safe_tag(b"i8", &i.to_string()[..])?;
            }
            Value::Bool(b) => {
                writer.write_safe_tag(b"boolean", if b { "1" } else { "0" })?;
            }
            Value::String(ref s) => {
                writer.write_tag(b"string", &s[..])?;
            }
            Value::Double(d) => {
                writer.write_safe_tag(b"double", &d.to_string()[..])?;
            }
            Value::DateTime(date_time) => {
                writer.write_safe_tag(b"dateTime.iso8601", &format!("{}", date_time)[..])?;
            }
            Value::Base64(ref data) => {
                writer.write_safe_tag(b"base64", &encode_base64(data)[..])?;
            }
            Value::Struct(ref map) => {
                writer.write_start_tag(b"struct")?;
                for (ref name, ref value) in map {
                    writer.write_start_tag(b"member")?;
                    writer.write_tag(b"name", &name[..])?;
                    writer
                        .write(&value.stringify()?.as_ref())
                        .map_err(EncodingError::from)?;
                    writer.write_end_tag(b"member")?;
                }
                writer.write_end_tag(b"struct")?;
            }
            Value::Array(ref array) => {
                writer.write_start_tag(b"array")?;
                writer.write_start_tag(b"data")?;
                for value in array {
                    // Raw write the value to the buffer because it's encoded xml.
                    writer
                        .write(&value.stringify()?.as_ref())
                        .map_err(EncodingError::from)?;
                }
                writer.write_end_tag(b"data")?;
                writer.write_end_tag(b"array")?;
            }
            Value::Nil => {
                writer.write(b"<nil />").map_err(EncodingError::from)?;
            }
        }
        writer.write_end_tag(b"value")?;

        Ok(String::from_utf8(buf).map_err(EncodingError::from)?)
    }

    /// Returns an inner struct or array value indexed by `index`.
    ///
    /// Returns `None` if the member doesn't exist or `self` is neither a struct nor an array.
    ///
    /// You can also use Rust's square-bracket indexing syntax to perform this operation if you want
    /// a default value instead of an `Option`. Refer to the top-level [examples](#examples) for
    /// details.
    /*
    pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
        index.get(self)
    }
    */

    /// If the `Value` is a normal integer (`Value::Int`), returns associated value. Returns `None`
    /// otherwise.
    ///
    /// In particular, `None` is also returned if `self` is a `Value::Int64`. Use [`as_i64`] to
    /// handle this case.
    ///
    /// [`as_i64`]: #method.as_i64
    pub fn as_i32(&self) -> Option<i32> {
        match *self {
            Value::Int(i) => Some(i),
            _ => None,
        }
    }

    /// If the `Value` is an integer, returns associated value. Returns `None` otherwise.
    ///
    /// This works with both `Value::Int` and `Value::Int64`.
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Int(i) => Some(i64::from(i)),
            Value::Int64(i) => Some(i),
            _ => None,
        }
    }

    /// If the `Value` is a boolean, returns associated value. Returns `None` otherwise.
    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// If the `Value` is a string, returns associated value. Returns `None` otherwise.
    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref s) => Some(s),
            _ => None,
        }
    }

    /// If the `Value` is a floating point number, returns associated value. Returns `None`
    /// otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Double(d) => Some(d),
            _ => None,
        }
    }

    /// If the `Value` is a date/time, returns associated value. Returns `None` otherwise.
    pub fn as_datetime(&self) -> Option<DateTime> {
        match *self {
            Value::DateTime(dt) => Some(dt),
            _ => None,
        }
    }

    /// If the `Value` is base64 binary data, returns associated value. Returns `None` otherwise.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match *self {
            Value::Base64(ref data) => Some(data),
            _ => None,
        }
    }

    /// If the `Value` is a struct, returns associated map. Returns `None` otherwise.
    pub fn as_struct(&self) -> Option<&BTreeMap<String, Value>> {
        match *self {
            Value::Struct(ref map) => Some(map),
            _ => None,
        }
    }

    /// If the `Value` is an array, returns associated slice. Returns `None` otherwise.
    pub fn as_array(&self) -> Option<&[Value]> {
        match *self {
            Value::Array(ref array) => Some(array),
            _ => None,
        }
    }
}

// Crate local definitions
impl Value {
    pub(crate) fn read_response_from_reader(
        mut reader: &mut Reader<&[u8]>,
        mut buf: &mut Vec<u8>,
    ) -> Result<Self> {
        let ret = match reader.read_event(&mut buf) {
            // If we got a start tag, we need to handle each of the value types.
            Ok(Event::Start(ref e)) => match e.name() {
                b"fault" => {
                    reader.expect_tag(b"value", &mut buf)?;
                    let val = Self::read_value_from_reader(&mut reader, &mut buf)?;
                    reader
                        .read_to_end(b"fault", &mut buf)
                        .map_err(ParseError::from)?;

                    let f = Fault::try_from(val)?;
                    let e = Error::from(f);
                    Err(e)
                }
                b"params" => {
                    reader.expect_tag(b"param", &mut buf)?;
                    reader.expect_tag(b"value", &mut buf)?;
                    let val = Self::read_value_from_reader(&mut reader, &mut buf)?;
                    reader
                        .read_to_end(b"param", &mut buf)
                        .map_err(ParseError::from)?;
                    reader
                        .read_to_end(b"params", &mut buf)
                        .map_err(ParseError::from)?;
                    Ok(val)
                }
                _ => {
                    return Err(ParseError::UnexpectedTag(
                        String::from_utf8_lossy(e.name()).into(),
                        "one of fault|params".into(),
                    )
                    .into())
                }
            },

            // Possible error states
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEOF("one of fault|params".into()))?,

            Err(e) => return Err(ParseError::from(e))?,

            _ => return Err(ParseError::UnexpectedEvent("one of fault|params".into()))?,
        };

        reader
            .read_to_end(b"methodResponse", &mut buf)
            .map_err(ParseError::from)?;

        ret
    }

    pub(crate) fn read_value_from_reader(
        mut reader: &mut Reader<&[u8]>,
        mut buf: &mut Vec<u8>,
    ) -> Result<Self> {
        let mut txt = Vec::new();

        // Read the next event. If it's text or the value closing tag, we know
        // we've got a string. If it's a start tag, we've got more work to do.
        let ret: Self = match reader.read_event(&mut buf) {
            // If we got text, this is a String value.
            Ok(Event::Text(e)) => e
                .unescape_and_decode(reader)
                .map(Value::from)
                .map_err(ParseError::from)?,

            // Alternatively, if we got the matching end tag, this is an empty
            // string value. Note that we need to return early here so the end
            // doesn't try to read the closing tag.
            Ok(Event::End(ref e)) if e.name() == b"value" => return Ok("".to_string().into()),

            // If we got a start tag, we need to handle each of the value types.
            Ok(Event::Start(ref e)) => match e.name() {
                b"i4" | b"int" => Self::read_int_from_reader(e.name(), &mut reader, &mut txt)?,
                b"i8" => Self::read_long_from_reader(b"i8", &mut reader, &mut txt)?,
                b"boolean" => Self::read_boolean_from_reader(b"boolean", &mut reader, &mut txt)?,
                b"string" => Self::read_string_from_reader(b"string", &mut reader, &mut txt)?,
                b"double" => Self::read_double_from_reader(b"double", &mut reader, &mut txt)?,
                b"dateTime.iso8601" => {
                    Self::read_datetime_from_reader(b"dateTime.iso8601", &mut reader, &mut txt)?
                }
                b"base64" => Self::read_base64_from_reader(b"base64", &mut reader, &mut txt)?,
                b"struct" => Self::read_struct_from_reader(b"struct", &mut reader, &mut txt)?,
                b"array" => Self::read_array_from_reader(b"array", &mut reader, &mut txt)?,
                b"nil" => Self::read_nil_from_reader(b"nil", &mut reader, &mut txt)?,

                _ => {
                    return Err(ParseError::UnexpectedTag(
                        String::from_utf8_lossy(e.name()).into(),
                        "one of i4|int|i8|boolean|string|double|dateTime.iso8601|base64|struct|array|nil".into(),
                    ))?;
                }
            },

            // Possible error states
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEOF(
                "one of i4|int|i8|boolean|string|double|dateTime.iso8601|base64|struct|array|nil"
                    .into(),
            ))?,

            Err(e) => return Err(ParseError::from(e))?,

            _ => return Err(ParseError::UnexpectedEvent(
                "one of i4|int|i8|boolean|string|double|dateTime.iso8601|base64|struct|array|nil"
                    .into(),
            ))?,
        };

        // Make sure we consume the closing value tag.
        reader
            .read_to_end("value", &mut buf)
            .map_err(ParseError::from)?;

        Ok(ret)
    }

    fn read_int_from_reader<K: AsRef<[u8]>>(
        end: K,
        reader: &mut Reader<&[u8]>,
        buf: &mut Vec<u8>,
    ) -> Result<Self> {
        let text = reader.read_text(end, buf).map_err(ParseError::from)?;
        Ok(text.parse::<i32>().map_err(ParseError::from)?.into())
    }

    fn read_long_from_reader<K: AsRef<[u8]>>(
        end: K,
        reader: &mut Reader<&[u8]>,
        buf: &mut Vec<u8>,
    ) -> Result<Self> {
        let text = reader.read_text(end, buf).map_err(ParseError::from)?;
        Ok(text.parse::<i64>().map_err(ParseError::from)?.into())
    }

    fn read_boolean_from_reader<K: AsRef<[u8]>>(
        end: K,
        reader: &mut Reader<&[u8]>,
        buf: &mut Vec<u8>,
    ) -> Result<Self> {
        let val = reader.read_text(end, buf).map_err(ParseError::from)?;
        let val = val.as_ref();
        match val {
            "1" => Ok(Value::Bool(true)),
            "0" => Ok(Value::Bool(false)),
            _ => Err(ParseError::BooleanDecodeError(val.into()).into()),
        }
    }

    fn read_string_from_reader<K: AsRef<[u8]>>(
        end: K,
        reader: &mut Reader<&[u8]>,
        buf: &mut Vec<u8>,
    ) -> Result<Self> {
        Ok(reader
            .read_text(end, buf)
            .map(Value::from)
            .map_err(ParseError::from)?)
    }

    fn read_double_from_reader<K: AsRef<[u8]>>(
        end: K,
        reader: &mut Reader<&[u8]>,
        buf: &mut Vec<u8>,
    ) -> Result<Self> {
        let text = reader.read_text(end, buf).map_err(ParseError::from)?;
        Ok(text.parse::<f64>().map_err(ParseError::from)?.into())
    }

    fn read_datetime_from_reader<K: AsRef<[u8]>>(
        end: K,
        reader: &mut Reader<&[u8]>,
        buf: &mut Vec<u8>,
    ) -> Result<Self> {
        let text = reader.read_text(end, buf).map_err(ParseError::from)?;
        Ok(parse_datetime(text.as_ref())
            .map(Self::from)
            .map_err(|e| ParseError::DateTimeDecodeError(e))?)
    }

    fn read_struct_from_reader<K: AsRef<[u8]>>(
        end: K,
        mut reader: &mut Reader<&[u8]>,
        mut buf: &mut Vec<u8>,
    ) -> Result<Self> {
        let mut ret: BTreeMap<String, Self> = BTreeMap::new();

        // Read the next event. If it's text or the value closing tag, we know
        // we've got a string. If it's a start tag, we've got more work to do.
        loop {
            match reader.read_event(&mut buf) {
                // If we got the matching end struct tag, we need to exit the
                // loop so we can handle cleanup.
                Ok(Event::End(ref e)) if e.name() == end.as_ref() => {
                    break;
                }

                // If we got a start tag, we need to handle each of the value types.
                Ok(Event::Start(ref e)) => match e.name() {
                    b"member" => {
                        reader.expect_tag(b"name", &mut buf)?;
                        let name = reader
                            .read_text(b"name", &mut buf)
                            .map_err(ParseError::from)?;
                        reader.expect_tag(b"value", &mut buf)?;
                        let val = Self::read_value_from_reader(&mut reader, &mut buf)?;
                        reader
                            .read_to_end(b"member", &mut buf)
                            .map_err(ParseError::from)?;
                        ret.insert(name, val);
                    }
                    _ => {
                        return Err(ParseError::UnexpectedTag(
                            String::from_utf8_lossy(e.name()).into(),
                            "member".into(),
                        ))?;
                    }
                },

                // Possible error states
                Ok(Event::Eof) => return Err(ParseError::UnexpectedEOF("member".into()))?,

                Err(e) => return Err(ParseError::from(e))?,

                _ => return Err(ParseError::UnexpectedEvent("member".into()))?,
            }
        }

        Ok(ret.into())
    }

    fn read_base64_from_reader<K: AsRef<[u8]>>(
        end: K,
        reader: &mut Reader<&[u8]>,
        buf: &mut Vec<u8>,
    ) -> Result<Self> {
        let text = reader.read_text(end, buf).map_err(ParseError::from)?;
        Ok(decode_base64(&text).map_err(ParseError::from)?.into())
    }

    fn read_array_from_reader<K: AsRef<[u8]>>(
        end: K,
        mut reader: &mut Reader<&[u8]>,
        mut buf: &mut Vec<u8>,
    ) -> Result<Self> {
        let mut ret: Vec<Self> = Vec::new();

        // The inner tag of an array should be a data tag.
        reader.expect_tag(b"data", buf)?;

        // Read the next event. If it's text or the value closing tag, we know
        // we've got a string. If it's a start tag, we've got more work to do.
        loop {
            match reader.read_event(&mut buf) {
                // If we got the matching end data tag, we need to exit the loop
                // so we can handle cleanup.
                Ok(Event::End(ref e)) if e.name() == b"data" => {
                    break;
                }

                // If we got a start tag, we need to handle each of the value types.
                Ok(Event::Start(ref e)) => match e.name() {
                    b"value" => ret.push(Value::read_value_from_reader(&mut reader, &mut buf)?),
                    _ => {
                        return Err(ParseError::UnexpectedTag(
                            String::from_utf8_lossy(e.name()).into(),
                            "value".into(),
                        ))?;
                    }
                },

                // Possible error states
                Ok(Event::Eof) => return Err(ParseError::UnexpectedEOF("value".into()))?,

                Err(e) => return Err(ParseError::from(e))?,

                _ => return Err(ParseError::UnexpectedEvent("value".into()))?,
            }
        }

        reader
            .read_to_end(end, &mut buf)
            .map_err(ParseError::from)?;

        Ok(ret.into())
    }

    fn read_nil_from_reader<K: AsRef<[u8]>>(
        end: K,
        reader: &mut Reader<&[u8]>,
        buf: &mut Vec<u8>,
    ) -> Result<Self> {
        reader.read_to_end(end, buf).map_err(ParseError::from)?;
        Ok(Self::Nil)
    }
}

impl From<i32> for Value {
    fn from(other: i32) -> Self {
        Value::Int(other)
    }
}

impl From<i64> for Value {
    fn from(other: i64) -> Self {
        Value::Int64(other)
    }
}

impl From<bool> for Value {
    fn from(other: bool) -> Self {
        Value::Bool(other)
    }
}

impl From<String> for Value {
    fn from(other: String) -> Self {
        Value::String(other)
    }
}

impl From<&str> for Value {
    fn from(other: &str) -> Self {
        Value::String(other.to_string())
    }
}

impl From<f64> for Value {
    fn from(other: f64) -> Self {
        Value::Double(other)
    }
}

impl From<DateTime> for Value {
    fn from(other: DateTime) -> Self {
        Value::DateTime(other)
    }
}

impl From<Vec<Value>> for Value {
    fn from(other: Vec<Value>) -> Value {
        Value::Array(other)
    }
}

impl From<BTreeMap<String, Value>> for Value {
    fn from(other: BTreeMap<String, Value>) -> Value {
        Value::Struct(other)
    }
}

impl From<Vec<u8>> for Value {
    fn from(other: Vec<u8>) -> Self {
        Value::Base64(other)
    }
}
