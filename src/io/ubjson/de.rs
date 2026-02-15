use std::io::Read;

use byteorder::{BigEndian, ReadBytesExt};
use serde_json::{Map, Value};

use crate::io::Result;

fn read_int<R: Read>(r: &mut R, marker: u8) -> Result<i64> {
	return match marker {
		0x55 => Ok(r.read_u8()? as i64),               // `U`
		0x69 => Ok(r.read_i8()? as i64),               // `i`
		0x49 => Ok(r.read_i16::<BigEndian>()? as i64), // `I`
		0x6c => Ok(r.read_i32::<BigEndian>()? as i64), // `l`
		0x4c => Ok(r.read_i64::<BigEndian>()?),        // `L`
		c => Err(err!("expected integer string length, got: {:#02x}", c)),
	};
}

fn read_utf8<R: Read>(r: &mut R, marker: u8) -> Result<String> {
	let length = read_int(r, marker)?;
	let mut buf = vec![0; length as usize];
	r.read_exact(&mut buf)?;
	Ok(String::from_utf8(buf)?)
}

fn read_array<R: Read>(r: &mut R) -> Result<Vec<Value>> {
	let mut values = vec![];
	while match r.read_u8()? {
		0x5d => false, // end of array (`]`)
		0x24 => {
			// optimized format (`$`)
			let type_marker = r.read_u8()?;
			let count_marker = r.read_u8()?;
			if count_marker != 0x23 {
				// `#`
				return Err(err!("expected count (`#`), got: {}", count_marker));
			}
			let count_type_marker = r.read_u8()?;
			let count = read_int(r, count_type_marker)?;
			for _ in 0..count {
				values.push(read_val(r, type_marker)?);
			}
			let end_marker = r.read_u8()?;
			if end_marker != 0x5d {
				// `]`
				return Err(err!("expected end of array (`]`), got: {}", end_marker));
			}
			false
		}
		c => {
			values.push(read_val(r, c)?);
			true // continue
		}
	} {}
	return Ok(values);
}

pub(crate) fn read_val<R: Read>(r: &mut R, marker: u8) -> Result<Value> {
	match marker {
		// "S": str
		0x53 => {
			let marker = r.read_u8()?;
			Ok(Value::String(read_utf8(r, marker)?))
		}
		// "{": map
		0x7b => Ok(Value::Object(read_map(r)?)),
		// number
		0x55 | 0x69 | 0x49 | 0x6c | 0x4c => Ok(Value::Number(serde_json::Number::from(read_int(
			r, marker,
		)?))),
		0x5b => Ok(Value::Array(read_array(r)?)),
		_ => Err(err!("unexpected value type: {:#02x}", marker)),
	}
}

pub(crate) fn read_key<R: Read>(r: &mut R) -> Result<Option<String>> {
	match r.read_u8()? {
		0x7d => Ok(None),
		c => Ok(Some(read_utf8(r, c)?)),
	}
}

pub(crate) fn read_map<R: Read>(r: &mut R) -> Result<Map<String, Value>> {
	let mut m = Map::new();
	while match read_key(r)? {
		Some(k) => {
			let marker = r.read_u8()?;
			m.insert(k, read_val(r, marker)?);
			true
		}
		None => false,
	} {}
	Ok(m)
}
