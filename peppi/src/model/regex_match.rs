macro_rules! regex_match {
	($module_name: ident : $type: ident {
		$($name: ident : $regex: expr),+ $(,)?
	}) => {
		#[cfg(feature = "regex_match")]
		mod $module_name {
			use lazy_static::lazy_static;
			use regex::Regex;
			lazy_static! {
				$( pub static ref $name: Regex = Regex::new($regex).unwrap(); )*
			}
		}

		#[cfg(feature = "regex_match")]
		impl $type {
			pub fn try_match(s: &str) -> Option<$type> {
				if !s.is_ascii() || s.len() > 50 {
					return None
				}

				match s {
					$( s if $module_name::$name.is_match(s) => Some($type::$name), )*
					_ => None,
				}
			}
		}
	}
}

