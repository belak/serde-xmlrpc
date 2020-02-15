use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::num::{ParseFloatError, ParseIntError};
use std::result;
use std::string::FromUtf8Error;

use base64::DecodeError;
use quick_xml::Error as XmlError;
use serde::{Deserialize, Serialize};
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
#[derive(ThisError, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[error("{fault_string} ({fault_code})")]
#[serde(rename_all = "camelCase")]
pub struct Fault {
    /// An application-specific error code.
    pub fault_code: i32,
    /// Human-readable error description.
    pub fault_string: String,
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

        let value: Value = input.serialize(crate::value::Serializer::new()).unwrap();
        let deserializer = crate::value::Deserializer::from_value(value);
        let new_input: Fault = Fault::deserialize(deserializer).unwrap();

        assert_eq!(new_input, input);
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
