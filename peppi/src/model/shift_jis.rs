use encoding_rs::SHIFT_JIS;
use serde::Serialize;

use std::io::Result;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ShiftJis(pub String);

impl ShiftJis {
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}
	pub fn to_normalized(&self) -> String {
		self.0.clone().chars().map(fix_char).collect::<String>()
	}
}

impl TryFrom<&[u8]> for ShiftJis {
	type Error = std::io::Error;
	fn try_from(s: &[u8]) -> Result<ShiftJis> {
		let first_null = s.iter().position(|&x| x == 0).unwrap_or(s.len());
		match SHIFT_JIS.decode_without_bom_handling(&s[0..first_null]) {
			(_, true) => Err(err!("malformed shift_jis sequence")),
			(cow, _) => Ok(ShiftJis(cow.to_string())),
		}
	}
}

/// Performs the following fixes:
/// 1. Map full-width code points to their half-width equivalents
/// https://en.wikipedia.org/wiki/Halfwidth_and_Fullwidth_Forms_(Unicode_block)
/// 2. Change ideographic space (U+3000) to ascii space
/// 3. Change tilde (called WAVE DASH) (U+301C) to ascii tilde ~
/// 4. Change Right Single/Double Quotation Mark (U+2019/U+201D) to their
/// ascii equivalents
fn fix_char(c: char) -> char {
	let mut c: u32 = u32::from(c);

	if c > 0xff00 && c < 0xff5f {
		c = c + 0x0020 - 0xff00;
	} else if c == 0x3000 {
		c = 0x20;
	} else if c == 0x301c {
		c = 0x7e;
	} else if c == 0x2019 {
		c = 0x27;
	} else if c == 0x201d {
		c = 0x22;
	}

	char::try_from(c).unwrap()
}
