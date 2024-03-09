use std::num::{ParseFloatError, ParseIntError};
use std::string::FromUtf8Error;

use base64::DecodeError;
use quick_xml::Error as XmlError;
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

/// Errors that can occur when trying to perform an XML-RPC request.
///
/// This can be a lower-level error (for example, the HTTP request failed), a problem with the
/// server (maybe it's not implementing XML-RPC correctly), or just a failure to execute the
/// operation.
#[derive(ThisError, Debug)]
pub enum Error {
    /// The response could not be decoded. This can happen when the server doesn't correctly
    /// implement the XML-RPC spec or malformed XML is sent.
    #[error("decoding error: {0}")]
    DecodingError(#[from] DecodingError),

    /// The response could not be encoded.
    #[error("encoding error: {0}")]
    EncodingError(#[from] EncodingError),

    /// The server returned a `<fault>` response, indicating that the execution of the call
    /// encountered a problem (for example, an invalid (number of) arguments was passed).
    #[error("server fault: {0}")]
    Fault(#[from] Fault),
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        DecodingError::SerdeError(msg.to_string()).into()
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        EncodingError::SerdeError(msg.to_string()).into()
    }
}

/// Error while parsing XML.
#[derive(ThisError, Debug)]
pub enum DecodingError {
    #[error("malformed XML: {0}")]
    XmlError(#[from] XmlError),

    #[error("malformed XML: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("malformed XML: {0}")]
    ParseFloatError(#[from] ParseFloatError),

    #[error("malformed XML: {0}")]
    Base64DecodeError(#[from] DecodeError),

    #[error("malformed XML: invalid boolean value: {0}")]
    BooleanDecodeError(String),

    #[error("malformed UTF-8: {0}")]
    Utf8Error(#[from] FromUtf8Error),

    #[error("unexpected tag: found {0}, expected {1}")]
    UnexpectedTag(String, String),

    #[error("unexpected event: expected tag {0}")]
    UnexpectedEvent(String),

    #[error("unexpected EOF: expected tag {0}")]
    UnexpectedEOF(String),

    #[error("key must be convertable to a string")]
    KeyMustBeString,

    #[error("serde: {0}")]
    SerdeError(String),
}

impl serde::de::Error for DecodingError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        DecodingError::SerdeError(msg.to_string())
    }
}

/// Error while encoding XML.
#[allow(clippy::enum_variant_names)]
#[derive(ThisError, Debug)]
pub enum EncodingError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("malformed UTF-8: {0}")]
    Utf8Error(#[from] FromUtf8Error),

    #[error("XML error: {0}")]
    XmlError(#[from] XmlError),

    #[error("invalid key type: key must be an {0}")]
    InvalidKeyType(String),

    #[error("serde: {0}")]
    SerdeError(String),
}

impl serde::ser::Error for EncodingError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        EncodingError::SerdeError(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

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

    use crate::Value;

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
