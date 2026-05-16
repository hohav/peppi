use std::io::Read;

use byteorder::{BigEndian, ReadBytesExt};

use crate::io::{
	Result,
	ubjson::{Map, Value},
};

fn read_utf8<R: Read>(r: &mut R) -> Result<String> {
	let length = r.read_u8()?;
	let mut buf = vec![0; length as usize];
	r.read_exact(&mut buf)?;
	Ok(String::from_utf8(buf)?)
}

pub(crate) fn read<R: Read>(r: &mut R) -> Result<Value> {
	match r.read_u8()? {
		// "S": str
		0x53 => match r.read_u8()? {
			0x55 => Ok(Value::String(read_utf8(r)?)),
			c => Err(err!("Expected 0x55 for string length, but got: {}", c)),
		},
		// "i": i8
		0x69 => Ok(Value::Int8(r.read_i8()?)),
		// "U": u8
		0x55 => Ok(Value::UInt8(r.read_u8()?)),
		// "I": i16
		0x49 => Ok(Value::Int16(r.read_i16::<BigEndian>()?)),
		// "l": i32
		0x6c => Ok(Value::Int32(r.read_i32::<BigEndian>()?)),
		// "L": i64
		0x4c => Ok(Value::Int64(r.read_i64::<BigEndian>()?)),
		// "{": map
		0x7b => Ok(Value::Object(read_map(r)?)),
		c => Err(err!("unexpected UBJSON value type: {}", c)),
	}
}

fn read_key<R: Read>(r: &mut R) -> Result<Option<String>> {
	match r.read_u8()? {
		0x55 => Ok(Some(read_utf8(r)?)),
		0x7d => Ok(None),
		c => Err(err!("unexpected UBJSON key type: {}", c)),
	}
}

pub(crate) fn read_map<R: Read>(r: &mut R) -> Result<Map> {
	let mut m = Map::new();
	while match read_key(r)? {
		Some(k) => {
			m.insert(k, read(r)?);
			true
		}
		None => false,
	} {}
	Ok(m)
}
