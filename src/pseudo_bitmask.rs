macro_rules! pseudo_bitmask {
	($name: ident : $type: ty { $( $value: expr => $variant: ident ),* $(,)? }) => {
		#[derive(PartialEq, Eq, Copy, Clone, serde::Serialize)]
		pub struct $name(pub $type);

		impl $name {
			$( pub const $variant:$name = $name($value); )*
		}

		impl std::fmt::Debug for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match unsafe { super::CONFIG.enum_names } {
					true => {
						let mut named_values: Vec<&str> = Vec::new();
						$( if (self.0 & $value) > 0 {
							named_values.push(stringify!($variant));
						} )*
						write!(f, "{} {:?}", self.0, named_values)
					},
					_ => write!(f, "{}", self.0),
				}
			}
		}

		impl std::ops::BitOr for $name {
			type Output = Self;

			fn bitor(self, rhs: Self) -> Self {
				$name(self.0 | rhs.0)
			}
		}

		impl std::ops::BitAnd for $name {
			type Output = Self;

			fn bitand(self, rhs: Self) -> Self {
				$name(self.0 & rhs.0)
			}
		}
	}
}
