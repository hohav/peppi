//! Melee's internal string encoding.

use encoding_rs::SHIFT_JIS;
use log::warn;
use serde::{Deserialize, Serialize};

use crate::io::{err, Error, Result};

/// Melee encodes strings using the Shift JIS character encoding for Japanese.
/// This type represents a Shift JIS string that has been decoded to utf-8.
/// While this decoded string is valid utf-8, many characters are represented
/// unusually or inconsistently. For that reason, most users should use
/// [to_normalized](crate::game::shift_jis::MeleeString::to_normalized) when
/// working with this type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeleeString(pub String);

impl MeleeString {
	/// Returns the decoded string as-is in utf-8.
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}

	/// Normalizes the decoded string and returns it.
	/// Performs the following fixes:
	/// 1. Map full-width code points to their half-width equivalents
	/// <https://en.wikipedia.org/wiki/Halfwidth_and_Fullwidth_Forms_(Unicode_block)>
	/// 2. Change ideographic space (U+3000) to ascii space
	/// 3. Change Right Single/Double Quotation Mark (U+2019/U+201D) to their
	/// ascii equivalents
	pub fn to_normalized(&self) -> String {
		self.0.clone().chars().map(fix_char).collect::<String>()
	}

	pub fn bytes(&self) -> std::borrow::Cow<[u8]> {
		let (result, encoding, unmapped) = SHIFT_JIS.encode(&self.0);
		assert_eq!(encoding, SHIFT_JIS);
		if unmapped {
			warn!("unsupported chars in MeleeString: {}", self.0);
		}
		result
	}
}

impl TryFrom<&[u8]> for MeleeString {
	type Error = Error;
	fn try_from(s: &[u8]) -> Result<MeleeString> {
		let first_null = s.iter().position(|&x| x == 0).unwrap_or(s.len());
		match SHIFT_JIS.decode_without_bom_handling_and_without_replacement(&s[0..first_null]) {
			Some(cow) => Ok(MeleeString(cow.to_string())),
			_ => Err(err!("invalid Shift JIS sequence")),
		}
	}
}

fn fix_char(c: char) -> char {
	let c = u32::from(c);
	let c = match c {
		0xff01..=0xff5e => c + 0x0020 - 0xff00, // full-width => half-width
		0x3000 => 0x20,                         // ideographic space => ascii space
		0x2019 => 0x27,                         // ’ => '
		0x201d => 0x22,                         // ” => "
		_ => c,
	};
	char::try_from(c).unwrap()
}
