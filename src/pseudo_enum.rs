macro_rules! pseudo_enum {
	($name:ident : $type:ty { $( $value:expr => $variant:ident ),* $(,)? }) => {
		#[derive(Copy, Clone, PartialEq, Eq, Hash)]
		pub struct $name(pub $type);

		impl $name {
			$( pub const $variant:$name = $name($value); )*
		}

		impl std::fmt::Debug for $name {
			fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match unsafe { super::CONFIG.enum_names } {
					true => match self.0 {
						$( $value => write!(f, "{}:{}", self.0, stringify!($variant)), )*
						_ => write!(f, "{}", self.0),
					},
					_ => write!(f, "{}", self.0),
				}
			}
		}

		impl super::query::Query for $name {
			fn query(&self, f:&mut dyn std::io::Write, config:&super::Config, _query:&[&str]) -> std::io::Result<()> {
				match config.json {
					true => serde_json::to_writer(f, self).map_err(|e| err!("JSON serialization error: {:?}", e)),
					_ => write!(f, "{:?}", self),
				}
			}
		}

		impl serde::Serialize for $name {
			fn serialize<S:serde::ser::Serializer>(&self, serializer:S) -> std::result::Result<S::Ok, S::Error> {
				match unsafe { super::CONFIG.enum_names } {
					true => format!("{:?}", self).serialize(serializer),
					_ => self.0.serialize(serializer),
				}
			}
		}
	}
}
