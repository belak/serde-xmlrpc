use std::collections::BTreeMap;
use std::error;
use std::io;
use std::fmt::{self, Display, Formatter};
use std::result;

use thiserror::Error;
use xml::common::TextPosition;
use xml::reader::Error as XmlError;

use crate::Value;

/// Errors that can occur when trying to perform an XML-RPC request.
///
/// This can be a lower-level error (for example, the HTTP request failed), a problem with the
/// server (maybe it's not implementing XML-RPC correctly), or just a failure to execute the
/// operation.
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /// The response could not be parsed. This can happen when the server doesn't correctly
    /// implement the XML-RPC spec.
    #[error("parse error: {0}")]
    ParseError(#[from] ParseError),

    /// The server returned a `<fault>` response, indicating that the execution of the call
    /// encountered a problem (for example, an invalid (number of) arguments was passed).
    #[error("server fault: {} ({})", .0.fault_string, .0.fault_code)]
    Fault(#[from] Fault),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        ParseError::from(e).into()
    }
}

pub type Result<T> = result::Result<T, Error>;

/// Describes possible error that can occur when parsing a `Response`.
#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    /// Error while parsing (malformed?) XML.
    #[error("malformed XML: {0}")]
    XmlError(#[from] XmlError),

    /// Could not parse the given CDATA as XML-RPC value.
    ///
    /// For example, `<value><int>AAA</int></value>` describes an invalid value.
    #[error("invalid value for type '{for_type}' at {position}: {found}")]
    InvalidValue {
        /// The type for which an invalid value was supplied (eg. `int` or `dateTime.iso8601`).
        for_type: &'static str,
        /// The value we encountered, as a string.
        found: String,
        /// The position of the invalid value inside the XML document.
        position: TextPosition,
    },

    /// Found an unexpected tag, attribute, etc.
    #[error("unexpected XML at {position} (expected {expected})")]
    UnexpectedXml {
        /// A short description of the kind of data that was expected.
        expected: String,
        // TODO: Fix found display
        found: Option<String>,
        /// The position of the unexpected data inside the XML document.
        position: TextPosition,
    },
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        ParseError::XmlError(XmlError::from(e))
    }
}

/// A `<fault>` response, indicating that a request failed.
///
/// The XML-RPC specification requires that a `<faultCode>` and `<faultString>` is returned in the
/// `<fault>` case, further describing the error.
#[derive(Debug, PartialEq, Eq)]
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

impl Display for Fault {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.fault_string, self.fault_code)
    }
}

impl error::Error for Fault {
    fn description(&self) -> &str {
        &self.fault_string
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
