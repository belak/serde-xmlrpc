use quick_xml::{events::Event, Reader, Writer};
use serde::{Deserialize};
use serde_transcode::transcode;

mod error;
mod util;
mod value;

use util::{ReaderExt, ValueDeserializer, ValueSerializer, WriterExt};

pub use error::{Error, Fault, Result};
pub use value::Value;

pub fn response_from_str<T>(input: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let mut reader = Reader::from_str(input);
    reader.expand_empty_elements(true);
    reader.trim_text(true);

    // Check the first event. This will determine if we're loading a Fault or a
    // Value.
    let mut buf = Vec::new();
    loop {
        match reader
            .read_event(&mut buf)
            .map_err(error::ParseError::from)?
        {
            Event::Decl(_) => continue,
            Event::Start(e) if e.name() == b"methodResponse" => {
                break;
            }
            e => return Err(error::ParseError::UnexpectedEvent(format!("{:?}", e)).into()),
        };
    }

    match reader
        .read_event(&mut buf)
        .map_err(error::ParseError::from)?
    {
        Event::Start(e) if e.name() == b"params" => {
            let mut buf = Vec::new();
            reader.expect_tag(b"param", &mut buf)?;
            let mut deserializer = ValueDeserializer::new(reader)?;
            let ret = T::deserialize(&mut deserializer)?;
            let mut reader = deserializer.into_inner();
            reader
                .read_to_end(b"param", &mut buf)
                .map_err(error::ParseError::from)?;
            reader
                .read_to_end(e.name(), &mut buf)
                .map_err(error::ParseError::from)?;
            Ok(ret)
        }
        Event::Start(e) if e.name() == b"fault" => {
            // The inner portion of a fault is just a Value tag, so we
            // deserialize it from a value.
            let mut deserializer = ValueDeserializer::new(reader)?;
            let fault: Fault = Fault::deserialize(&mut deserializer)?;

            // Pull the reader back out so we can verify the end tag.
            let mut reader = deserializer.into_inner();

            let mut buf = Vec::new();
            reader
                .read_to_end(e.name(), &mut buf)
                .map_err(error::ParseError::from)?;

            Err(fault.into())
        }
        e => Err(error::ParseError::UnexpectedEvent(format!("{:?}", e)).into()),
    }
}

/// Expects an input string which is xmlrpc request body, and parses out the method name and parameters from it.
/// This function would typically be used by a server to parse incoming requests.
/// Returns a tuple of (method name, Arguments).
/// This does not parse the types of the arguments, as typically the server needs to resolve
/// the method name prior know expected types.
pub fn request_from_str(request: &str) -> Result<(String, Vec<Value>)> {
    let mut reader = Reader::from_str(request);
    reader.expand_empty_elements(true);
    reader.trim_text(true);

    // Search for methodCall start
    let mut buf = Vec::new();
    loop {
        match reader
            .read_event(&mut buf)
            .map_err(error::ParseError::from)?
        {
            Event::Decl(_) => continue,
            Event::Start(e) if e.name() == b"methodCall" => {
                break;
            }
            e => return Err(error::ParseError::UnexpectedEvent(format!("{:?}", e)).into()),
        };
    }

    // This code currently assumes that the <methodName> will always precede <params>
    // in the xmlrpc request, I'm not certain that this is actually enforced by the
    // specification, but could find not counter example where it wasn't true... -Carter

    let method_name = match reader
        .read_event(&mut buf)
        .map_err(error::ParseError::from)?
    {
        Event::Start(e) if e.name() == b"methodName" => {
            let mut buf = Vec::new();
            reader
                .read_text(e.name(), &mut buf)
                .map_err(error::ParseError::from)?
        }
        e => return Err(error::ParseError::UnexpectedEvent(format!("{:?}", e)).into()),
    };

    match reader
        .read_event(&mut buf)
        .map_err(error::ParseError::from)?
    {
        Event::Start(e) if e.name() == b"params" => {
            let mut buf = Vec::new();
            let mut params = Vec::new();
            // Read each parameter into a Value
            while let Ok(_) = reader.expect_tag(b"param", &mut buf) {
                // This feels wrong / inefficient, but was the best way I could come up with
                // from looking at the general structure of this code:
                let mut reader2 = Reader::from_str(&request[reader.buffer_position()..]);
                reader2.expand_empty_elements(true);
                reader2.trim_text(true);

                let mut deserializer = ValueDeserializer::new(reader2)?;
                let serializer = value::Serializer::new();
                let x = transcode(&mut deserializer, serializer)?;
                params.push(x);
                reader
                    .read_to_end(b"param", &mut buf)
                    .map_err(error::ParseError::from)?;
            }
            Ok((method_name, params))
        }
        e => Err(error::ParseError::UnexpectedEvent(format!("{:?}", e)).into()),
    }
}

pub fn request_to_string(name: &str, args: Vec<Value>) -> Result<String> {
    let mut writer = Writer::new(Vec::new());

    writer
        .write(br#"<?xml version="1.0" encoding="utf-8"?>"#)
        .map_err(error::EncodingError::from)?;

    writer.write_start_tag(b"methodCall")?;
    writer.write_tag(b"methodName", name)?;

    writer.write_start_tag(b"params")?;
    for value in args {
        writer.write_start_tag(b"param")?;

        let deserializer = value::Deserializer::from_value(value);
        let serializer = ValueSerializer::new(&mut writer);
        transcode(deserializer, serializer)?;

        writer.write_end_tag(b"param")?;
    }
    writer.write_end_tag(b"params")?;
    writer.write_end_tag(b"methodCall")?;

    Ok(String::from_utf8(writer.into_inner()).map_err(error::EncodingError::from)?)
}

pub fn value_from_str(input: &str) -> Result<Value> {
    let mut reader = Reader::from_str(input);
    reader.expand_empty_elements(true);
    reader.trim_text(true);

    let mut deserializer = ValueDeserializer::new(reader)?;
    let serializer = value::Serializer::new();
    transcode(&mut deserializer, serializer)
}

pub fn value_to_string<I>(val: I) -> Result<String>
where
    I: Into<Value>,
{
    let d = value::Deserializer::from_value(val.into());
    let mut writer = Writer::new(Vec::new());
    let s = ValueSerializer::new(&mut writer);
    transcode(d, s)?;
    Ok(String::from_utf8(writer.into_inner()).map_err(error::EncodingError::from)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stringify_request() {
        assert_eq!(
            request_to_string("hello world", vec![]).unwrap(),
            r#"<?xml version="1.0" encoding="utf-8"?><methodCall><methodName>hello world</methodName><params></params></methodCall>"#.to_owned()
        )
    }

    /// A 32-bit signed integer (`<i4>` or `<int>`).
    #[test]
    fn parse_int_values() {
        assert_eq!(
            value_from_str("<value><int>42</int></value>")
                .unwrap()
                .as_i32(),
            Some(42)
        );

        assert_eq!(
            value_from_str("<value><int>-42</int></value>")
                .unwrap()
                .as_i32(),
            Some(-42)
        );

        assert_eq!(
            value_from_str("<value><int>2147483647</int></value>")
                .unwrap()
                .as_i32(),
            Some(2147483647)
        );
    }

    /// A 64-bit signed integer (`<i8>`).
    #[test]
    fn parse_long_values() {
        assert_eq!(
            value_from_str("<value><int>42</int></value>")
                .unwrap()
                .as_i64(),
            Some(42)
        );

        assert_eq!(
            value_from_str("<value><int>9223372036854775807</int></value>")
                .unwrap()
                .as_i64(),
            Some(9223372036854775807)
        );
    }

    /// A boolean value (`<boolean>`, 0 == `false`, 1 == `true`).
    #[test]
    fn parse_boolean_values() {
        assert_eq!(
            value_from_str("<value><boolean>1</boolean></value>")
                .unwrap()
                .as_bool(),
            Some(true)
        );
        assert_eq!(
            value_from_str("<value><boolean>0</boolean></value>")
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
            value_from_str("<value><string>hello</string></value>")
                .unwrap()
                .as_str(),
            Some("hello")
        );

        assert_eq!(
            value_from_str("<value>world</value>").unwrap().as_str(),
            Some("world")
        );

        assert_eq!(value_from_str("<value />").unwrap().as_str(), Some(""));
    }

    /// A double-precision IEEE 754 floating point number (`<double>`).
    #[test]
    fn parse_double_values() {
        assert_eq!(
            value_from_str("<value><double>1</double></value>")
                .unwrap()
                .as_f64(),
            Some(1.0)
        );
        assert_eq!(
            value_from_str("<value><double>0</double></value>")
                .unwrap()
                .as_f64(),
            Some(0.0)
        );
        assert_eq!(
            value_from_str("<value><double>42</double></value>")
                .unwrap()
                .as_f64(),
            Some(42.0)
        );
        assert_eq!(
            value_from_str("<value><double>3.14</double></value>")
                .unwrap()
                .as_f64(),
            Some(3.14)
        );
        assert_eq!(
            value_from_str("<value><double>-3.14</double></value>")
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
            value_from_str("<value><base64>aGVsbG8gd29ybGQ=</base64></value>")
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
            value_from_str(
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
        assert_eq!(
            value_from_str("<value><nil /></value>").unwrap(),
            Value::Nil
        );
    }

    #[test]
    fn parse_fault() {
        let err = response_from_str::<String>(
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
                println!("{:?}", err);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_value() {
        let val: String = response_from_str(
            r#"<?xml version="1.0" encoding="utf-8"?>
            <methodResponse>
              <params>
                <param><value><string>hello world</string></value></param>
              </params>
            </methodResponse>"#,
        )
        .unwrap();

        assert_eq!(val, "hello world".to_string());
    }

    #[test]
    fn test_parse_request() {
        // Example data taken from a ROS node connection negotation
        let val = r#"<?xml version=\"1.0\"?>
          <methodCall>
            <methodName>requestTopic</methodName>
            <params>
              <param><value>/rosout</value></param>
            </params>
          </methodCall>"#;

        let (method_name, arg) = request_from_str(&val).unwrap();
        assert_eq!(arg.get(0).unwrap().as_str().unwrap(), "/rosout");
        assert_eq!(&method_name, "requestTopic");
    }

    /// This test is currently failing
    /// the code adapted from response_to_string is not varadic against multiple params
    #[test]
    fn test_parse_request_multiple_params() {
        // Example data taken from a ROS node connection negotation
        let val = r#"<?xml version=\"1.0\"?>
          <methodCall>
            <methodName>requestTopic</methodName>
            <params>
              <param><value>/rosout</value></param>
              <param><value>/rosout</value></param>
              <param><value><array><data><value><array><data><value>TCPROS</value></data></array></value></data></array></value></param>
            </params>
          </methodCall>"#;

        let (method, vals) = request_from_str(val).unwrap();
        assert_eq!(vals.len(), 3);
        assert_eq!(&method, "requestTopic");
        assert_eq!(vals.get(0).unwrap().as_str().unwrap(), "/rosout");
        assert_eq!(vals.get(1).unwrap().as_str().unwrap(), "/rosout");
        assert_eq!(
            vals.get(2)
                .unwrap()
                .as_array()
                .unwrap()
                .get(0)
                .unwrap()
                .as_array()
                .unwrap()
                .get(0)
                .unwrap()
                .as_str()
                .unwrap(),
            "TCPROS"
        );
    }
}
