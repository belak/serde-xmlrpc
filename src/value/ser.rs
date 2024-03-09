use std::collections::BTreeMap;

use serde::Serialize;

use crate::error::EncodingError;
use crate::{Error, Result, Value};

pub struct Serializer;

impl Serializer {
    pub fn new() -> Self {
        Serializer {}
    }
}

impl serde::Serializer for Serializer {
    type Error = Error;
    type Ok = Value;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeVec;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeMap;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        Ok(Value::Int(v as i32))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        Ok(Value::Int(v as i32))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        Ok(Value::Int(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        Ok(Value::Int64(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        Ok(Value::Int(v as i32))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        Ok(Value::Int(v as i32))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        Ok(Value::Int64(v as i64))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok> {
        // This type doesn't fit inside an i32 or i64 which are the only
        // officially supported int types in xmlrpc.

        // TODO: replace with Error
        unimplemented!();
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        Ok(Value::Double(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(Value::Double(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        Ok(Value::Base64(v.into()))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(Value::Nil)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(Serializer)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(Value::Nil)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Ok(Value::Struct(BTreeMap::new()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(Serializer)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        // TODO: replace with Error
        unimplemented!();
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.serialize_tuple(len.unwrap_or(0))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_tuple(len)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeMap {
            map: BTreeMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_map(Some(len))
    }
}

#[doc(hidden)]
pub struct SerializeVec {
    vec: Vec<Value>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.vec.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Value::Array(self.vec))
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

#[doc(hidden)]
pub struct SerializeMap {
    map: BTreeMap<String, Value>,
    next_key: Option<String>,
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        // While we could technically allow for any type which can be serialized
        // to a string to be used as a key, it's a bit cleaner to only allow
        // "string" types.
        match key.serialize(Serializer)? {
            Value::String(s) => {
                self.next_key = Some(s);
                Ok(())
            }
            _ => Err(EncodingError::InvalidKeyType("string".to_string()).into()),
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let key = self
            .next_key
            .take()
            .expect("serialize_value called before serialize_key");
        let value = value.serialize(Serializer)?;

        self.map.insert(key, value);

        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Struct(self.map))
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Value> {
        serde::ser::SerializeMap::end(self)
    }
}

impl serde::ser::SerializeStructVariant for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Value> {
        serde::ser::SerializeMap::end(self)
    }
}

#[cfg(test)]
mod test {
    use serde::Serialize;

    use super::Serializer;

    #[derive(Serialize, Debug, PartialEq)]
    struct Test {
        hello: String,
    }

    #[derive(Serialize, Debug, PartialEq)]
    struct Test2 {
        val: Option<String>,
    }

    #[test]
    fn test_serde() {
        use std::collections::BTreeMap;
        use std::iter::FromIterator;

        use crate::Value;

        let x = Value::Int(42);
        let y: i32 = 42;
        let y = y.serialize(Serializer).unwrap();
        assert_eq!(y, x);

        let x = Value::Array(vec![Value::String("hello world".to_string())]);
        let y: Vec<String> = vec!["hello world".to_string()];
        let y = y.serialize(Serializer).unwrap();
        assert_eq!(y, x);

        let x = Value::Array(vec![Value::String("hello world".to_string())]);
        let y: Vec<String> = vec!["hello world".to_string()];
        let y = y.serialize(Serializer).unwrap();
        assert_eq!(y, x);

        let x = Value::Struct(BTreeMap::from_iter(
            vec![("hello".to_string(), Value::String("world".to_string()))].into_iter(),
        ));
        let y = Test {
            hello: "world".to_string(),
        };
        let y = y.serialize(Serializer).unwrap();
        assert_eq!(y, x,);

        let x = Value::Struct(BTreeMap::from_iter(
            vec![("val".to_string(), Value::Nil)].into_iter(),
        ));
        let y = Test2 { val: None };
        let y = y.serialize(Serializer).unwrap();
        assert_eq!(y, x);

        let x = Value::Struct(BTreeMap::from_iter(
            vec![("val".to_string(), Value::String("hello".to_string()))].into_iter(),
        ));
        let y = Test2 {
            val: Some("hello".to_string()),
        };
        let y = y.serialize(Serializer).unwrap();
        assert_eq!(y, x,);
    }
}
