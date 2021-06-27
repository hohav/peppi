macro_rules! pseudo_bitmask {
	($name: ident : $type: ty { $( $value: expr => $variant: ident ),* $(,)? }) => {
		#[derive(Copy, Clone, Default, PartialEq, Eq, serde::Serialize)]
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

		impl peppi_arrow::Arrow for $name {
			type Builder = <$type as peppi_arrow::Arrow>::Builder;

			fn default() -> Self {
				<Self as Default>::default()
			}

			fn data_type<C: ::peppi_arrow::Context>(context: C) -> arrow::datatypes::DataType {
				<$type>::data_type(context)
			}

			fn builder<C: ::peppi_arrow::Context>(len: usize, context: C) -> Self::Builder {
				<$type>::builder(len, context)
			}

			fn write<C: ::peppi_arrow::Context>(&self, builder: &mut dyn ::arrow::array::ArrayBuilder, context: C) {
				self.0.write(builder, context)
			}

			fn write_null<C: ::peppi_arrow::Context>(builder: &mut dyn ::arrow::array::ArrayBuilder, context: C) {
				<$type>::write_null(builder, context)
			}

			fn read(&mut self, array: arrow::array::ArrayRef, idx: usize) {
				self.0.read(array, idx);
			}
		}
	}
}
