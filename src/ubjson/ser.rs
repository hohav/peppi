use std::io::{Result, Write};

use byteorder::{BigEndian, WriteBytesExt};
use serde_json::{Map, Value};

fn write_utf8<W: Write>(w: &mut W, s: &str) -> Result<()> {
	write!(w, "U")?;
	w.write_u8(s.len().try_into().unwrap())?;
	write!(w, "{}", s)?;
	Ok(())
}

pub fn from_map<W: Write>(w: &mut W, map: &Map<String, Value>) -> Result<()> {
	for (k, v) in map {
		write_utf8(w, k)?;
		match v {
			Value::String(s) => {
				write!(w, "S")?;
				write_utf8(w, s)?;
			}
			Value::Number(n) => {
				write!(w, "l")?;
				w.write_i32::<BigEndian>(n.as_i64().unwrap().try_into().unwrap())?;
			}
			Value::Object(o) => {
				write!(w, "{{")?;
				from_map(w, o)?;
				write!(w, "}}")?;
			}
			_ => unimplemented!(),
		}
	}
	Ok(())
}
