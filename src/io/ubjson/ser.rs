use std::io::{Result, Write};

use byteorder::{BigEndian, WriteBytesExt};

use crate::io::ubjson::{Map, Value};

fn write_utf8_value<W: Write>(w: &mut W, s: &str) -> Result<()> {
	write!(w, "U")?;
	w.write_u8(s.len().try_into().unwrap())?;
	write!(w, "{}", s)
}

fn write_utf8<W: Write>(w: &mut W, s: &str) -> Result<()> {
	write!(w, "S")?;
	write_utf8_value(w, s)
}

pub(crate) fn write_map<W: Write>(w: &mut W, map: &Map) -> Result<()> {
	write!(w, "{{")?;
	for (k, v) in map.iter() {
		write_utf8_value(w, &k)?;
		match v {
			Value::String(s) => {
				write_utf8(w, s)?;
			}
			Value::Int8(n) => {
				write!(w, "i")?;
				w.write_i8(*n)?;
			}
			Value::UInt8(n) => {
				write!(w, "U")?;
				w.write_u8(*n)?;
			}
			Value::Int16(n) => {
				write!(w, "I")?;
				w.write_i16::<BigEndian>(*n)?;
			}
			Value::Int32(n) => {
				write!(w, "l")?;
				w.write_i32::<BigEndian>(*n)?;
			}
			Value::Int64(n) => {
				write!(w, "L")?;
				w.write_i64::<BigEndian>(*n)?;
			}
			Value::Float32(n) => {
				write!(w, "d")?;
				w.write_f32::<BigEndian>(*n)?;
			}
			Value::Float64(n) => {
				write!(w, "D")?;
				w.write_f64::<BigEndian>(*n)?;
			}
			Value::Object(o) => {
				write_map(w, o)?;
			}
			_ => unimplemented!(),
		}
	}
	write!(w, "}}")
}
