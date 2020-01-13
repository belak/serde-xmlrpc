use quick_xml::{Reader, Writer};

mod error;
mod utils;
mod value;

use utils::{ReaderExt, WriterExt};

pub use crate::error::{Error, Result};
pub use crate::value::Value;

pub fn parse_response(data: &str) -> Result<Value> {
    let mut reader = Reader::from_str(data);
    reader.expand_empty_elements(true);
    reader.trim_text(true);

    let mut buf = Vec::new();

    // We expect a value tag first, followed by a value. Note that the inner
    // read will properly handle ensuring we get a closing value tag.
    reader.expect_tag(b"methodResponse", &mut buf)?;

    Value::read_response_from_reader(&mut reader, &mut buf)
}

pub fn parse_value(data: &str) -> Result<Value> {
    let mut reader = Reader::from_str(data);
    reader.expand_empty_elements(true);
    reader.trim_text(true);

    let mut buf = Vec::new();

    // We expect a value tag first, followed by a value. Note that the inner
    // read will properly handle ensuring we get a closing value tag.
    reader.expect_tag(b"value", &mut buf)?;

    Value::read_value_from_reader(&mut reader, &mut buf)
}

pub fn stringify_request(name: &str, args: &[Value]) -> Result<String> {
    let mut buf = Vec::new();
    let mut writer = Writer::new(&mut buf);

    writer
        .write(br#"<?xml version="1.0" encoding="utf-8"?>"#)
        .map_err(error::EncodingError::from)?;

    writer.write_start_tag(b"methodCall")?;
    writer.write_tag(b"methodName", name)?;

    writer.write_start_tag(b"params")?;
    for value in args {
        writer.write_start_tag(b"param")?;

        writer
            .write(value.stringify()?.as_ref())
            .map_err(error::EncodingError::from)?;

        writer.write_end_tag(b"param")?;
    }
    writer.write_end_tag(b"params")?;
    writer.write_end_tag(b"methodCall")?;

    Ok(String::from_utf8(buf).map_err(error::EncodingError::from)?)
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
            error::Error::Fault(f) => assert_eq!(
                f,
                error::Fault {
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
