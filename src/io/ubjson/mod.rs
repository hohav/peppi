pub(crate) mod de;
pub(crate) mod ser;

pub(crate) use de::read_map;
pub(crate) use ser::write_map;

use crate::array_map::ArrayMap;

#[cfg(feature = "serde")]
use serde_json::{Number as JNumber, Value as JValue};
#[cfg(feature = "serde")]
pub(crate) type JMap = serde_json::Map<String, JValue>;

pub type Map = ArrayMap<String, Value>;

#[allow(non_snake_case)]
#[cfg(feature = "serde")]
pub(crate) fn Map(values: Vec<(String, Value)>) -> Map {
	ArrayMap(values) // TODO: check for duplicates?
}

#[derive(Debug, PartialEq)]
pub enum Value {
	Null,
	Bool(bool),
	Int8(i8),
	UInt8(u8),
	Int16(i16),
	Int32(i32),
	Int64(i64),
	Float32(f32),
	Float64(f64),
	String(String),
	Array(Vec<Value>),
	Object(Map),
}

#[cfg(feature = "serde")]
impl From<Map> for JMap {
	fn from(m: Map) -> Self {
		m.into_iter().map(|(k, v)| (k, v.into())).collect()
	}
}

#[cfg(feature = "serde")]
impl From<JMap> for Map {
	fn from(m: JMap) -> Self {
		Map(m.into_iter().map(|(k, v)| (k, v.into())).collect())
	}
}

#[cfg(feature = "serde")]
impl Into<JValue> for Value {
	fn into(self) -> JValue {
		use Value::*;
		match self {
			Null => JValue::Null,
			Bool(x) => JValue::Bool(x),
			Int8(x) => JValue::Number(x.into()),
			UInt8(x) => JValue::Number(x.into()),
			Int16(x) => JValue::Number(x.into()),
			Int32(x) => JValue::Number(x.into()),
			Int64(x) => JValue::Number(x.into()),
			Float32(x) => JValue::Number(JNumber::from_f64(x.into()).expect("invalid f32")),
			Float64(x) => JValue::Number(JNumber::from_f64(x).expect("invalid f64")),
			String(s) => JValue::String(s),
			Array(arr) => JValue::Array(arr.into_iter().map(|x| x.into()).collect()),
			Object(obj) => JValue::Object(obj.into_iter().map(|(k, v)| (k, v.into())).collect()),
		}
	}
}

#[cfg(feature = "serde")]
impl Into<Value> for JValue {
	fn into(self) -> Value {
		use JValue::*;
		match self {
			Null => Value::Null,
			Bool(x) => Value::Bool(x),
			String(s) => Value::String(s),
			Array(arr) => Value::Array(arr.into_iter().map(|x| x.into()).collect()),
			Object(obj) => {
				Value::Object(Map(obj.into_iter().map(|(k, v)| (k, v.into())).collect()))
			}
			Number(x) => {
				if x.is_i64() {
					let x = x.as_i64().unwrap();
					if x < i32::MAX as i64 {
						Value::Int32(x as i32)
					} else {
						Value::Int64(x)
					}
				} else if x.is_f64() {
					let x = x.as_f64().unwrap();
					if x < f32::MAX as f64 {
						Value::Float32(x as f32)
					} else {
						Value::Float64(x)
					}
				} else {
					panic!("unsupported numeric value: {}", x);
				}
			}
		}
	}
}
