macro_rules! pseudo_enum {
	($name:ident : $type:ty { $( $value:expr => $variant:ident ),* $(,)? }) => {
		#[derive(Copy, Clone, PartialEq, Eq, Hash)]
		pub struct $name(pub $type);

		impl $name {
			$( pub const $variant:$name = $name($value); )*
		}

		impl std::fmt::Debug for $name {
			fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match unsafe { super::SERIALIZE_ENUMS_WITH_NAMES } {
					true => match self.0 {
						$( $value => write!(f, "{}:{}", stringify!($value), stringify!($variant)), )*
						_ => write!(f, "{}", self.0),
					},
					_ => write!(f, "{}", self.0),
				}
			}
		}

		impl serde::Serialize for $name {
			fn serialize<S:serde::ser::Serializer>(&self, serializer:S) -> Result<S::Ok, S::Error> {
				match unsafe { super::SERIALIZE_ENUMS_WITH_NAMES } {
					true => format!("{:?}", self).serialize(serializer),
					_ => self.0.serialize(serializer),
				}
			}
		}
	}
}
