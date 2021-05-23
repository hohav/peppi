macro_rules! pseudo_bitmask {
	($name: ident : $type: ty { $( $value: expr => $variant: ident ),* $(,)? }) => {
		#[derive(PartialEq, Eq, Copy, Clone, serde::Serialize)]
		pub struct $name(pub $type);

		impl $name {
			$( pub const $variant:$name = $name($value); )*
		}

		impl std::fmt::Debug for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match unsafe { super::SERIALIZATION_CONFIG.enum_names } {
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

		impl super::arrow::ArrowPrimitive for $name {
			type ArrowNativeType = $type;
			type ArrowType = <$type as super::arrow::ArrowPrimitive>::ArrowType;
			const ARROW_DATA_TYPE: ::arrow::datatypes::DataType = <$type>::ARROW_DATA_TYPE;
			fn into_arrow_native(self) -> Self::ArrowNativeType { self.0 as $type }
		}
	}
}
