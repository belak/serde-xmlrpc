use std::io::{self, Write};
use xml::escape::{escape_str_pcdata as escape_xml};

use crate::Value;

/// A request to call a procedure.
#[derive(Clone, Debug)]
pub struct Request<'a> {
    name: &'a str,
    args: &'a [Value],
}

impl<'a> Request<'a> {
    /// Creates a new request to call a function named `name`.
    pub fn new(name: &'a str, args: &'a [Value]) -> Self {
        Request {
            name,
            args,
        }
    }

    /// Formats this `Request` as a UTF-8 encoded XML document.
    ///
    /// # Errors
    ///
    /// Any errors reported by the writer will be propagated to the caller. If the writer never
    /// returns an error, neither will this method.
    pub fn write_as_xml<W: Write>(&self, fmt: &mut W) -> io::Result<()> {
        write!(fmt, r#"<?xml version="1.0" encoding="utf-8"?>"#)?;
        write!(fmt, r#"<methodCall>"#)?;
        write!(
            fmt,
            r#"    <methodName>{}</methodName>"#,
            escape_xml(self.name)
        )?;
        write!(fmt, r#"    <params>"#)?;
        for value in self.args {
            write!(fmt, r#"        <param>"#)?;
            value.write_as_xml(fmt)?;
            write!(fmt, r#"        </param>"#)?;
        }
        write!(fmt, r#"    </params>"#)?;
        write!(fmt, r#"</methodCall>"#)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn escapes_method_names() {
        let mut output: Vec<u8> = Vec::new();
        let req = Request::new("x<&x".into(), &[]);

        req.write_as_xml(&mut output).unwrap();
        assert!(str::from_utf8(&output)
            .unwrap()
            .contains("<methodName>x&lt;&amp;x</methodName>"));
    }
}
