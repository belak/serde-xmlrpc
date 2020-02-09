use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::num::{ParseFloatError, ParseIntError};
use std::result;
use std::string::FromUtf8Error;

use base64::DecodeError;
use quick_xml::Error as XmlError;
use thiserror::Error as ThisError;

use crate::Value;

/// Errors that can occur when trying to perform an XML-RPC request.
///
/// This can be a lower-level error (for example, the HTTP request failed), a problem with the
/// server (maybe it's not implementing XML-RPC correctly), or just a failure to execute the
/// operation.
#[derive(ThisError, Debug)]
pub enum Error {
    /// The response could not be parsed. This can happen when the server doesn't correctly
    /// implement the XML-RPC spec.
    #[error("parse error: {0}")]
    ParseError(#[from] ParseError),

    /// The response could not be encoded.
    #[error("encoding error: {0}")]
    EncodingError(#[from] EncodingError),

    /// The server returned a `<fault>` response, indicating that the execution of the call
    /// encountered a problem (for example, an invalid (number of) arguments was passed).
    #[error("server fault: {0}")]
    Fault(#[from] Fault),

    #[error("serde decoding error: {0}")]
    DecodeError(String),

    #[error("serde encoding error: {0}")]
    EncodeError(String),
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::DecodeError(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::EncodeError(msg.to_string())
    }
}

/// Error while parsing XML.
#[derive(ThisError, Debug)]
pub enum ParseError {
    #[error("malformed XML: {0}")]
    XmlError(#[from] XmlError),

    #[error("malformed XML: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("malformed XML: {0}")]
    ParseFloatError(#[from] ParseFloatError),

    #[error("malformed XML: {0}")]
    Base64DecodeError(#[from] DecodeError),

    #[error("malformed XML: {0}")]
    DateTimeDecodeError(String),

    #[error("malformed XML: invalid boolean value: {0}")]
    BooleanDecodeError(String),

    #[error("malformed UTF-8: {0}")]
    Utf8Error(#[from] FromUtf8Error),

    #[error("unexpected tag: found {0}, expected {1}")]
    UnexpectedTag(String, String),

    #[error("unexpected error: {0}, expected tag {1}")]
    UnexpectedError(anyhow::Error, String),

    #[error("unexpected event: expected tag {0}")]
    UnexpectedEvent(String),

    #[error("unexpected EOF: expected tag {0}")]
    UnexpectedEOF(String),

    #[error("tag not found: {0}")]
    TagNotFound(String),

    #[error("key must be convertable to a string")]
    KeyMustBeString,

    #[error("fault: {0}")]
    ParseFaultError(String),
}

/// Error while encoding XML.
#[derive(ThisError, Debug)]
pub enum EncodingError {
    #[error("malformed UTF-8: {0}")]
    Utf8Error(#[from] FromUtf8Error),

    #[error("XML error: {0}")]
    XmlError(#[from] XmlError),
}

pub type Result<T> = result::Result<T, Error>;

/// A `<fault>` response, indicating that a request failed.
///
/// The XML-RPC specification requires that a `<faultCode>` and `<faultString>` is returned in the
/// `<fault>` case, further describing the error.
#[derive(ThisError, Debug, PartialEq, Eq)]
#[error("{fault_string} ({fault_code})")]
pub struct Fault {
    /// An application-specific error code.
    pub fault_code: i32,
    /// Human-readable error description.
    pub fault_string: String,
}

/// Creates a `Fault` from a `Value`.
///
/// The `Value` must be a `Value::Struct` with a `faultCode` and `faultString` field (and no
/// other fields).
impl TryFrom<Value> for Fault {
    type Error = ParseError;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Struct(ref map) => {
                match (map.get("faultCode"), map.get("faultString")) {
                    (Some(&Value::Int(fault_code)), Some(&Value::String(ref fault_string))) => {
                        if map.len() != 2 {
                            // incorrect field count
                            Err(ParseError::ParseFaultError(
                                "extra fields returned in fault".into(),
                            ))
                        } else {
                            Ok(Fault {
                                fault_code,
                                fault_string: fault_string.to_string(),
                            })
                        }
                    }
                    _ => Err(ParseError::ParseFaultError(
                        "missing either faultCode or faultString".into(),
                    )),
                }
            }
            _ => Err(ParseError::ParseFaultError("expected struct".into())),
        }
    }
}

/// Turns this `Fault` into an equivalent `Value`.
///
/// The returned value can be parsed back into a `Fault` using `Fault::try_from`
/// or returned as a `<fault>` error response by serializing it into a
/// `<fault></fault>` tag.
impl From<&Fault> for Value {
    fn from(other: &Fault) -> Self {
        let mut map = BTreeMap::new();
        map.insert("faultCode".to_string(), Value::from(other.fault_code));
        map.insert(
            "faultString".to_string(),
            Value::from(other.fault_string.clone()),
        );

        Value::Struct(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error;

    #[test]
    fn fault_roundtrip() {
        let input = Fault {
            fault_code: -123456,
            fault_string: "The Bald Lazy House Jumps Over The Hyperactive Kitten".to_string(),
        };

        assert_eq!(Fault::try_from(Value::from(&input)).unwrap(), input);
    }

    #[test]
    fn error_impls_error() {
        fn assert_error<T: error::Error>() {}

        assert_error::<Error>();
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<Error>();
    }
}
