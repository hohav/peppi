macro_rules! pseudo_bitmask {
	($name: ident : $type: ty { $( $value: expr => $variant: ident ),* $(,)? }) => {
		#[derive(Copy, Clone, Default, PartialEq, Eq, serde::Serialize)]
		pub struct $name(pub $type);

		impl $name {
			$( pub const $variant:$name = $name($value); )*
		}

		#[allow(clippy::bad_bit_mask)]
		impl std::fmt::Debug for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match unsafe { crate::SERIALIZATION_CONFIG.enum_names } {
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

		impl peppi_arrow::Arrow for $name {
			type ArrowArray = <$type as peppi_arrow::Arrow>::ArrowArray;

			fn arrow_default() -> Self {
				<Self as Default>::default()
			}

			fn data_type<C: peppi_arrow::Context>(context: C) -> ::arrow2::datatypes::DataType {
				<$type>::data_type(context)
			}

			fn arrow_array<C: peppi_arrow::Context>(context: C) -> Self::ArrowArray {
				<$type>::arrow_array(context)
			}

			fn arrow_push(&self, array: &mut dyn ::arrow2::array::MutableArray) {
				self.0.arrow_push(array)
			}

			fn arrow_push_null(array: &mut dyn ::arrow2::array::MutableArray) {
				<$type>::arrow_push_null(array)
			}

			fn arrow_read(&mut self, array: &dyn ::arrow2::array::Array, idx: usize) {
				self.0.arrow_read(array, idx);
			}
		}
	}
}
