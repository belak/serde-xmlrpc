use std::collections::BTreeMap;
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
    #[error("server fault: {} ({})", .0.fault_string, .0.fault_code)]
    Fault(#[from] Fault),
}

#[derive(ThisError, Debug)]
pub enum ParseError {
    /// Error while parsing (malformed?) XML.
    #[error("malformed XML: {0}")]
    XmlError(#[from] XmlError),

    #[error("malformed XML: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("malformed XML: {0}")]
    ParseFloatError(#[from] ParseFloatError),

    #[error("malformed XML: {0}")]
    Base64DecodeError(#[from] DecodeError),

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

    #[error("generic error: {0}")]
    Generic(String),
}

#[derive(ThisError, Debug)]
pub enum EncodingError {
    #[error("malformed UTF-8: {0}")]
    Utf8Error(#[from] FromUtf8Error),

    /// Error while parsing (malformed?) XML.
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

impl Fault {
    /// Creates a `Fault` from a `Value`.
    ///
    /// The `Value` must be a `Value::Struct` with a `faultCode` and `faultString` field (and no
    /// other fields).
    ///
    /// Returns `None` if the value isn't a valid `Fault`.
    pub fn from_value(value: &Value) -> Option<Self> {
        match *value {
            Value::Struct(ref map) => {
                if map.len() != 2 {
                    // incorrect field count
                    return None;
                }

                match (map.get("faultCode"), map.get("faultString")) {
                    (Some(&Value::Int(fault_code)), Some(&Value::String(ref fault_string))) => {
                        Some(Fault {
                            fault_code,
                            fault_string: fault_string.to_string(),
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Turns this `Fault` into an equivalent `Value`.
    ///
    /// The returned value can be parsed back into a `Fault` using `Fault::from_value` or returned
    /// as a `<fault>` error response by serializing it into a `<fault></fault>` tag.
    pub fn to_value(&self) -> Value {
        let mut map = BTreeMap::new();
        map.insert("faultCode".to_string(), Value::from(self.fault_code));
        map.insert(
            "faultString".to_string(),
            Value::from(self.fault_string.as_ref()),
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

        assert_eq!(Fault::from_value(&input.to_value()), Some(input));
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
