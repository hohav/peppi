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
/// 3. Change Right Single/Double Quotation Mark (U+2019/U+201D) to their
/// ascii equivalents
fn fix_char(c: char) -> char {
	let c = u32::from(c);
	let c = match c {
		0xff01..=0xff5e => c + 0x0020 - 0xff00,
		0x3000 => 0x20,
		0x2019 => 0x27,
		0x201d => 0x22,
		_ => c,
	};
	char::try_from(c).unwrap()
}
