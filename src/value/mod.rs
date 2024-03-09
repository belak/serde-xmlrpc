use std::{collections::BTreeMap, convert::TryFrom};

use iso8601::DateTime;

pub mod de;
pub mod ser;

pub use de::Deserializer;
pub use ser::Serializer;

/// Convert a `T` into `serde_xmlrpc::Value` which is an enum that can represent
/// any valid XML-RPC data.
///
/// # Example
///
/// ```
/// #[derive(serde::Serialize)]
/// struct Custom {
///     field: i32,
/// }
///
/// let param = Custom {
///     field: 42
/// };
///
/// let _value = serde_xmlrpc::to_value(param).unwrap();
/// ```
pub fn to_value<T>(value: T) -> crate::Result<Value>
where
    T: serde::Serialize,
{
    value.serialize(Serializer)
}

/// Attempts to deserialize the Value into the given type, equivalent API of
/// [serde_json::from_value](https://docs.rs/serde_json/latest/serde_json/fn.from_value.html).
/// ```
/// use serde_xmlrpc::{from_value, Value};
/// let val = Value::Array(vec![Value::Int(3), Value::String("Test".to_string())]);
/// let (x, y): (i32, String) = from_value(val).unwrap();
/// ```
pub fn from_value<T: serde::de::DeserializeOwned>(value: Value) -> crate::Result<T> {
    let d = Deserializer::from_value(value);
    T::deserialize(d)
}

/// Represents any single valid xmlrpc "Value"
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// A 32-bit signed integer (`<i4>` or `<int>`).
    Int(i32),
    /// A 64-bit signed integer (`<i8>`).
    Int64(i64),
    /// A boolean value (`<boolean>`, 0 == `false`, 1 == `true`).
    Bool(bool),
    /// A string (`<string>`).
    String(String),
    /// A double-precision IEEE 754 floating point number (`<double>`).
    Double(f64),
    /// An ISO 8601 formatted date/time value (`<dateTime.iso8601>`).
    DateTime(DateTime),
    /// Base64-encoded binary data (`<base64>`).
    Base64(Vec<u8>),
    /// A mapping of named values (`<struct>`).
    Struct(BTreeMap<String, Value>),
    /// A list of arbitrary (heterogeneous) values (`<array>`).
    Array(Vec<Value>),
    /// The empty (Unit) value (`<nil/>`).
    Nil,
}

// Public API definitions
impl Value {
    /// Returns an inner struct or array value indexed by `index`.
    ///
    /// Returns `None` if the member doesn't exist or `self` is neither a struct nor an array.
    ///
    /// You can also use Rust's square-bracket indexing syntax to perform this operation if you want
    /// a default value instead of an `Option`. Refer to the top-level [examples](#examples) for
    /// details.
    /*
    pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
        index.get(self)
    }
    */

    /// If the `Value` is a normal integer (`Value::Int`), returns associated value. Returns `None`
    /// otherwise.
    ///
    /// In particular, `None` is also returned if `self` is a `Value::Int64`. Use [`as_i64`] to
    /// handle this case.
    ///
    /// [`as_i64`]: #method.as_i64
    pub fn as_i32(&self) -> Option<i32> {
        match *self {
            Value::Int(i) => Some(i),
            _ => None,
        }
    }

    /// If the `Value` is an integer, returns associated value. Returns `None` otherwise.
    ///
    /// This works with both `Value::Int` and `Value::Int64`.
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Int(i) => Some(i64::from(i)),
            Value::Int64(i) => Some(i),
            _ => None,
        }
    }

    /// If the `Value` is a boolean, returns associated value. Returns `None` otherwise.
    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// If the `Value` is a string, returns associated value. Returns `None` otherwise.
    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref s) => Some(s),
            _ => None,
        }
    }

    /// If the `Value` is a floating point number, returns associated value. Returns `None`
    /// otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Double(d) => Some(d),
            _ => None,
        }
    }

    /// If the `Value` is a date/time, returns associated value. Returns `None` otherwise.
    pub fn as_datetime(&self) -> Option<DateTime> {
        match *self {
            Value::DateTime(dt) => Some(dt),
            _ => None,
        }
    }

    /// If the `Value` is base64 binary data, returns associated value. Returns `None` otherwise.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match *self {
            Value::Base64(ref data) => Some(data),
            _ => None,
        }
    }

    /// If the `Value` is a struct, returns associated map. Returns `None` otherwise.
    pub fn as_struct(&self) -> Option<&BTreeMap<String, Value>> {
        match *self {
            Value::Struct(ref map) => Some(map),
            _ => None,
        }
    }

    /// If the `Value` is an array, returns associated slice. Returns `None` otherwise.
    pub fn as_array(&self) -> Option<&[Value]> {
        match *self {
            Value::Array(ref array) => Some(array),
            _ => None,
        }
    }
}

// Conversions into and from Value

impl From<i32> for Value {
    fn from(other: i32) -> Self {
        Value::Int(other)
    }
}

impl<'a> TryFrom<&'a Value> for i32 {
    type Error = ();
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int(i) => Ok(*i),
            _ => Err(()),
        }
    }
}

impl From<i64> for Value {
    fn from(other: i64) -> Self {
        Value::Int64(other)
    }
}

impl<'a> TryFrom<&'a Value> for &'a i64 {
    type Error = ();
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int64(i) => Ok(i),
            _ => Err(()),
        }
    }
}

impl From<bool> for Value {
    fn from(other: bool) -> Self {
        Value::Bool(other)
    }
}

impl<'a> TryFrom<&'a Value> for &'a bool {
    type Error = ();
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(i) => Ok(i),
            _ => Err(()),
        }
    }
}

impl From<String> for Value {
    fn from(other: String) -> Self {
        Value::String(other)
    }
}

impl<'a> TryFrom<&'a Value> for String {
    type Error = ();

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        if let Some(val) = value.as_str() {
            Ok(val.to_string())
        } else {
            Err(())
        }
    }
}

impl From<&str> for Value {
    fn from(other: &str) -> Self {
        Value::String(other.to_string())
    }
}

impl<'a> TryFrom<&'a Value> for &'a str {
    type Error = ();

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        if let Some(val) = value.as_str() {
            Ok(val)
        } else {
            Err(())
        }
    }
}

impl From<f64> for Value {
    fn from(other: f64) -> Self {
        Value::Double(other)
    }
}

impl<'a> TryFrom<&'a Value> for &'a f64 {
    type Error = ();
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Double(i) => Ok(i),
            _ => Err(()),
        }
    }
}

impl From<DateTime> for Value {
    fn from(other: DateTime) -> Self {
        Value::DateTime(other)
    }
}

impl<'a> TryFrom<&'a Value> for &'a DateTime {
    type Error = ();
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::DateTime(i) => Ok(i),
            _ => Err(()),
        }
    }
}

impl From<Vec<Value>> for Value {
    fn from(other: Vec<Value>) -> Value {
        Value::Array(other)
    }
}

impl<'a> TryFrom<&'a Value> for &'a Vec<Value> {
    type Error = ();
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(i) => Ok(i),
            _ => Err(()),
        }
    }
}

impl From<BTreeMap<String, Value>> for Value {
    fn from(other: BTreeMap<String, Value>) -> Value {
        Value::Struct(other)
    }
}

impl<'a> TryFrom<&'a Value> for &'a BTreeMap<String, Value> {
    type Error = ();
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Struct(i) => Ok(i),
            _ => Err(()),
        }
    }
}

impl From<Vec<u8>> for Value {
    fn from(other: Vec<u8>) -> Self {
        Value::Base64(other)
    }
}

impl<'a> TryFrom<&'a Value> for &'a Vec<u8> {
    type Error = ();
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Base64(i) => Ok(i),
            _ => Err(()),
        }
    }
}
