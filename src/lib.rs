mod error;
mod parser;
mod request;
mod value;

/// Encodes a request to bytes.
pub fn encode_request(name: &str, args: &[Value]) -> Result<Vec<u8>> {
    let mut ret = Vec::new();

    let req = request::Request::new(name, args);
    req.write_as_xml(&mut ret)?;

    Ok(ret)
}

/// Parses a response.
pub fn parse_response(data: &str) -> Result<Value> {
    parser::Parser::new(&mut data.as_bytes())?.parse_response()
}

pub use error::{Error, Result};
pub use value::Value;
