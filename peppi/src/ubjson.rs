use std::io::{Read, Write, Result, Error, ErrorKind};
use std::convert::TryInto;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde_json::{Map, Value};

fn parse_utf8<R: Read>(r: &mut R) -> Result<String> {
	let length = r.read_u8()?;
	let mut buf = vec![0; length as usize];
	r.read_exact(&mut buf)?;
	String::from_utf8(buf).map_err(|e| Error::new(ErrorKind::InvalidData, e))
}

fn parse_val<R: Read>(r: &mut R) -> Result<Value> {
	match r.read_u8()? {
		// "S": str
		0x53 => match r.read_u8()? {
			0x55 => Ok(Value::String(parse_utf8(r)?)),
			c => Err(err!("Expected 0x55 for string length, but got: {}", c)),
		},
		// "l": i32
		0x6c => Ok(Value::Number(serde_json::Number::from(r.read_i32::<BigEndian>()?))),
		// "{": map
		0x7b => Ok(Value::Object(parse_map(r)?)),
		c => Err(err!("unexpected UBJSON value type: {}", c)),
	}
}

fn parse_key<R: Read>(r: &mut R) -> Result<Option<String>> {
	match r.read_u8()? {
		0x55 => Ok(Some(parse_utf8(r)?)),
		0x7d => Ok(None),
		c => Err(err!("unexpected UBJSON key type: {}", c)),
	}
}

pub fn parse_map<R: Read>(r: &mut R) -> Result<Map<String, Value>> {
	let mut m = Map::new();
	while match parse_key(r)? {
		Some(k) => {m.insert(k, parse_val(r)?); true},
		None => false,
	} {}
	Ok(m)
}

fn write_utf8<W: Write>(w: &mut W, s: &String) -> Result<()> {
	//FIXME: length in bytes?
	write!(w, "U")?;
	w.write_u8(s.len().try_into().unwrap())?;
	write!(w, "{}", s)?;
	Ok(())
}

pub fn unparse_map<W: Write>(w: &mut W, map: &Map<String, Value>) -> Result<()> {
	for (k, v) in map {
		write_utf8(w, k)?;
		match v {
			Value::String(s) => {
				write!(w, "S")?;
				write_utf8(w, s)?;
			},
			Value::Number(n) => {
				write!(w, "l")?;
				w.write_i32::<BigEndian>(n.as_i64().unwrap().try_into().unwrap())?;
			},
			Value::Object(o) => {
				write!(w, "{{")?;
				unparse_map(w, o)?;
				write!(w, "}}")?;
			}
			_ => unimplemented!(),
		}
	}
	Ok(())
}
