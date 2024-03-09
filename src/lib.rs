use std::result;

use thiserror::Error as ThisError;

/// Errors that can occur when trying to perform an XML-RPC request.
///
/// This can be a lower-level error (for example, the HTTP request failed), a problem with the
/// server (maybe it's not implementing XML-RPC correctly), or just a failure to execute the
/// operation.
#[deprecated(since = "0.1.1", note = "please use `serde_xmlrpc::Error` instead")]
#[derive(ThisError, Debug)]
pub enum Error {
    /// The response could not be parsed. This can happen when the server doesn't correctly
    /// implement the XML-RPC spec.
    #[error("parse error: {0}")]
    ParseError(String),

    /// The response could not be encoded.
    #[error("encoding error: {0}")]
    EncodingError(String),

    /// The server returned a `<fault>` response, indicating that the execution of the call
    /// encountered a problem (for example, an invalid (number of) arguments was passed).
    #[error("server fault: {0}")]
    Fault(#[from] Fault),
}

impl From<serde_xmlrpc::Error> for Error {
    fn from(err: serde_xmlrpc::Error) -> Self {
        match err {
            serde_xmlrpc::Error::DecodingError(err) => Error::ParseError(err.to_string()),
            serde_xmlrpc::Error::EncodingError(err) => Error::EncodingError(err.to_string()),
            serde_xmlrpc::Error::Fault(fault) => Error::Fault(fault.into()),
        }
    }
}

#[deprecated(since = "0.1.1", note = "please use `serde_xmlrpc::Result` instead")]
pub type Result<T> = result::Result<T, Error>;

/// A `<fault>` response, indicating that a request failed.
///
/// The XML-RPC specification requires that a `<faultCode>` and `<faultString>` is returned in the
/// `<fault>` case, further describing the error.
#[deprecated(since = "0.1.2", note = "please use `serde_xmlrpc::Fault` instead")]
#[derive(ThisError, Debug, PartialEq, Eq)]
#[error("{fault_string} ({fault_code})")]
pub struct Fault {
    /// An application-specific error code.
    pub fault_code: i32,
    /// Human-readable error description.
    pub fault_string: String,
}

impl From<serde_xmlrpc::Fault> for Fault {
    fn from(fault: serde_xmlrpc::Fault) -> Self {
        Fault {
            fault_code: fault.fault_code,
            fault_string: fault.fault_string,
        }
    }
}

#[deprecated(since = "0.1.2", note = "please use `serde_xmlrpc::Value` instead")]
pub type Value = serde_xmlrpc::Value;

#[deprecated(
    since = "0.1.1",
    note = "please use `serde_xmlrpc::response_from_str` instead"
)]
pub fn parse_response(data: &str) -> Result<Value> {
    Ok(serde_xmlrpc::response_from_str(data)?)
}

#[deprecated(
    since = "0.1.1",
    note = "please use `serde_xmlrpc::value_from_str` instead"
)]
pub fn parse_value(data: &str) -> Result<Value> {
    Ok(serde_xmlrpc::value_from_str(data)?)
}

#[deprecated(
    since = "0.1.1",
    note = "please use `serde_xmlrpc::request_to_string` instead"
)]
pub fn stringify_request(name: &str, args: &[Value]) -> Result<String> {
    Ok(serde_xmlrpc::request_to_string(name, args.to_vec())?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stringify_request() {
        assert_eq!(
            stringify_request("hello world", &[]).unwrap(),
            r#"<?xml version="1.0" encoding="utf-8"?><methodCall><methodName>hello world</methodName><params></params></methodCall>"#.to_owned()
        )
    }

    /// A 32-bit signed integer (`<i4>` or `<int>`).
    #[test]
    fn parse_int_values() {
        assert_eq!(
            parse_value("<value><i4>42</i4></value>").unwrap().as_i32(),
            Some(42)
        );

        assert_eq!(
            parse_value("<value><int>-42</int></value>")
                .unwrap()
                .as_i32(),
            Some(-42)
        );

        assert_eq!(
            parse_value("<value><int>2147483647</int></value>")
                .unwrap()
                .as_i32(),
            Some(2147483647)
        );
    }

    /// A 64-bit signed integer (`<i8>`).
    #[test]
    fn parse_long_values() {
        assert_eq!(
            parse_value("<value><i8>42</i8></value>").unwrap().as_i64(),
            Some(42)
        );

        assert_eq!(
            parse_value("<value><i8>9223372036854775807</i8></value>")
                .unwrap()
                .as_i64(),
            Some(9223372036854775807)
        );
    }

    /// A boolean value (`<boolean>`, 0 == `false`, 1 == `true`).
    #[test]
    fn parse_boolean_values() {
        assert_eq!(
            parse_value("<value><boolean>1</boolean></value>")
                .unwrap()
                .as_bool(),
            Some(true)
        );
        assert_eq!(
            parse_value("<value><boolean>0</boolean></value>")
                .unwrap()
                .as_bool(),
            Some(false)
        );
    }

    /// A string (`<string>`). Note that these can also appear as a raw
    /// value tag as well.
    #[test]
    fn parse_string_values() {
        assert_eq!(
            parse_value("<value><string>hello</string></value>")
                .unwrap()
                .as_str(),
            Some("hello")
        );

        assert_eq!(
            parse_value("<value>world</value>").unwrap().as_str(),
            Some("world")
        );

        assert_eq!(parse_value("<value />").unwrap().as_str(), Some(""));
    }

    /// A double-precision IEEE 754 floating point number (`<double>`).
    #[test]
    fn parse_double_values() {
        assert_eq!(
            parse_value("<value><double>1</double></value>")
                .unwrap()
                .as_f64(),
            Some(1.0)
        );
        assert_eq!(
            parse_value("<value><double>0</double></value>")
                .unwrap()
                .as_f64(),
            Some(0.0)
        );
        assert_eq!(
            parse_value("<value><double>42</double></value>")
                .unwrap()
                .as_f64(),
            Some(42.0)
        );
        assert_eq!(
            parse_value("<value><double>3.14</double></value>")
                .unwrap()
                .as_f64(),
            Some(3.14)
        );
        assert_eq!(
            parse_value("<value><double>-3.14</double></value>")
                .unwrap()
                .as_f64(),
            Some(-3.14)
        );
    }

    /// An ISO 8601 formatted date/time value (`<dateTime.iso8601>`).

    /// Base64-encoded binary data (`<base64>`).
    #[test]
    fn parse_base64_values() {
        assert_eq!(
            parse_value("<value><base64>aGVsbG8gd29ybGQ=</base64></value>")
                .unwrap()
                .as_bytes(),
            Some(&b"hello world"[..])
        );
    }

    /// A mapping of named values (`<struct>`).

    /// A list of arbitrary (heterogeneous) values (`<array>`).
    #[test]
    fn parse_array_values() {
        assert_eq!(
            parse_value(
                "<value><array><data><value></value><value><nil /></value></data></array></value>"
            )
            .unwrap()
            .as_array(),
            Some(&[Value::String("".to_owned()), Value::Nil][..])
        );
    }

    /// The empty (Unit) value (`<nil/>`).
    #[test]
    fn parse_nil_values() {
        assert_eq!(parse_value("<value><nil /></value>").unwrap(), Value::Nil);
    }

    #[test]
    fn parse_fault() {
        let err = parse_response(
            r#"<?xml version="1.0" encoding="utf-8"?>
           <methodResponse>
             <fault>
               <value>
                 <struct>
                   <member>
                     <name>faultCode</name>
                     <value><int>4</int></value>
                   </member>
                   <member>
                     <name>faultString</name>
                     <value><string>Too many parameters.</string></value>
                   </member>
                 </struct>
                </value>
              </fault>
            </methodResponse>"#,
        )
        .unwrap_err();

        match err {
            Error::Fault(f) => assert_eq!(
                f,
                Fault {
                    fault_code: 4,
                    fault_string: "Too many parameters.".into(),
                }
            ),
            _ => {
                assert!(false);
            }
        }
    }
}
