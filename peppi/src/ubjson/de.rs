use std::io::{Error, ErrorKind, Read, Result};

use byteorder::{BigEndian, ReadBytesExt};
use serde_json::{Map, Value};

fn to_utf8<R: Read>(r: &mut R) -> Result<String> {
	let length = r.read_u8()?;
	let mut buf = vec![0; length as usize];
	r.read_exact(&mut buf)?;
	String::from_utf8(buf).map_err(|e| Error::new(ErrorKind::InvalidData, e))
}

fn to_val<R: Read>(r: &mut R) -> Result<Value> {
	match r.read_u8()? {
		// "S": str
		0x53 => match r.read_u8()? {
			0x55 => Ok(Value::String(to_utf8(r)?)),
			c => Err(err!("Expected 0x55 for string length, but got: {}", c)),
		},
		// "l": i32
		0x6c => Ok(Value::Number(serde_json::Number::from(
			r.read_i32::<BigEndian>()?,
		))),
		// "{": map
		0x7b => Ok(Value::Object(to_map(r)?)),
		c => Err(err!("unexpected UBJSON value type: {}", c)),
	}
}

fn to_key<R: Read>(r: &mut R) -> Result<Option<String>> {
	match r.read_u8()? {
		0x55 => Ok(Some(to_utf8(r)?)),
		0x7d => Ok(None),
		c => Err(err!("unexpected UBJSON key type: {}", c)),
	}
}

pub fn to_map<R: Read>(r: &mut R) -> Result<Map<String, Value>> {
	let mut m = Map::new();
	while match to_key(r)? {
		Some(k) => {
			m.insert(k, to_val(r)?);
			true
		}
		None => false,
	} {}
	Ok(m)
}
