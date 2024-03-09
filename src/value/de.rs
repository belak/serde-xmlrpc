use std::collections::BTreeMap;

use serde::de::Visitor;
use serde::forward_to_deserialize_any;

use crate::{Error, Result, Value};

impl<'de> serde::Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Int(v) => visitor.visit_i32(v),
            Value::Int64(v) => visitor.visit_i64(v),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::String(v) => visitor.visit_string(v),
            Value::Double(v) => visitor.visit_f64(v),
            Value::DateTime(v) => visitor.visit_string(v.to_string()),
            Value::Base64(v) => visitor.visit_bytes(v.as_slice()),
            Value::Struct(v) => {
                let map_deserializer = MapDeserializer::new(v);
                visitor.visit_map(map_deserializer)
            }
            Value::Array(v) => {
                let seq_deserializer = SeqDeserializer::new(v);
                visitor.visit_seq(seq_deserializer)
            }
            Value::Nil => visitor.visit_none(),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::Nil = self {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    forward_to_deserialize_any!(
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    );
}

struct SeqDeserializer {
    iter: std::vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> serde::de::SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }
}

struct MapDeserializer {
    iter: <BTreeMap<String, Value> as IntoIterator>::IntoIter,
    value: Option<Value>,
}

impl MapDeserializer {
    fn new(map: BTreeMap<String, Value>) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> serde::de::MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(Value::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: this isn't quite right - the error type doesn't match up.
        // Unfortunately, there doesn't seem to be a way to solve this, as it
        // requires the error type to match up between the serializer and
        // deserializer, and they have incompatible error types.
        serde_transcode::transcode(deserializer, super::Serializer::new())
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct Test {
        hello: String,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Test2 {
        val: Option<String>,
    }

    #[test]
    fn test_serde() {
        use std::collections::BTreeMap;
        use std::iter::FromIterator;

        use crate::Value;

        let x = Value::Int(42);
        let y = i32::deserialize(x).unwrap();
        assert_eq!(y, 42);

        let x = Value::Array(vec![Value::String("hello world".to_string())]);
        let y: Vec<String> = Vec::deserialize(x).unwrap();
        assert_eq!(y, vec!["hello world".to_string()]);

        let x = Value::Struct(BTreeMap::from_iter(
            vec![("hello".to_string(), Value::String("world".to_string()))].into_iter(),
        ));
        let y = Test::deserialize(x).unwrap();
        assert_eq!(
            y,
            Test {
                hello: "world".to_string(),
            },
        );

        let x = Value::Struct(BTreeMap::new());
        let y = Test2::deserialize(x).unwrap();
        assert_eq!(y, Test2 { val: None },);

        let x = Value::Struct(BTreeMap::from_iter(
            vec![("val".to_string(), Value::Nil)].into_iter(),
        ));
        let y = Test2::deserialize(x).unwrap();
        assert_eq!(y, Test2 { val: None },);

        let x = Value::Struct(BTreeMap::from_iter(
            vec![("val".to_string(), Value::String("hello".to_string()))].into_iter(),
        ));
        let y = Test2::deserialize(x).unwrap();
        assert_eq!(
            y,
            Test2 {
                val: Some("hello".to_string())
            },
        );
    }
}
