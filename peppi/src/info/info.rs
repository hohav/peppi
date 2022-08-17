macro_rules! info {
	($type: path => $info: ident {
		$($name: ident {
			$( $field: ident : $value: expr ),+ $(,)?
		});+ $(;)?
	}) => {
		impl $info {
			pub fn try_from(value: $type) -> Option<$info> {
				match value {
					$( <$type>::$name => Some($name), )*
					_ => None
				}
			}
		}

		$(pub const $name: $info = $info {
			$( $field: $value ),*
		};)*
	}
}

macro_rules! info_regex {
	($info: ident {
        $($name: ident : $regex: expr),+ $(,)?
    }) => {
		mod regexes {
			use lazy_static::lazy_static;
			use regex::Regex;
			lazy_static! {
				$( pub static ref $name: Regex = Regex::new($regex).unwrap(); )*
			}
		}

		impl $info {
			pub fn try_match(s: &str) -> Option<$info> {
				match s {
					$( s if regexes::$name.is_match(s) => Some($name), )*
					_ => None,
				}
			}
		}
	}
}
