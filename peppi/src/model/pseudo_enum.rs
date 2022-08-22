#[derive(std::fmt::Debug)]
pub struct ConversionError {
	pub r#type: String,
	pub value: String,
}

impl std::fmt::Display for ConversionError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "no such {}: {}", self.r#type, self.value)
	}
}

impl std::error::Error for ConversionError { }

// An open "enum" that supports named and unnamed values.
// Used when not all possible values are known.
macro_rules! pseudo_enum {
	($name: ident : $type: ty { $( $value: expr => $variant: ident ),* $(,)? }) => {
		#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct $name(pub $type);

		impl $name {
			$( pub const $variant:$name = $name($value); )*
		}

		impl std::convert::TryFrom<$name> for String {
			type Error = crate::model::pseudo_enum::ConversionError;
			/// Returns the stringified name for this enum value, if any.
			fn try_from(e: $name) -> std::result::Result<Self, Self::Error> {
				match e.0 {
					$( $value => Ok(stringify!($variant).to_string()), )*
					_ => Err(Self::Error {
						r#type: format!("{}::{}", module_path!(), stringify!($name)),
						value: format!("{}", e.0),
					}),
				}
			}
		}

		impl std::fmt::Debug for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match unsafe { crate::SERIALIZATION_CONFIG.enum_names } {
					true => match String::try_from(*self) {
						Ok(s) => write!(f, "{}:{}", self.0, s),
						_ => write!(f, "{}", self.0),
					},
					_ => write!(f, "{}", self.0),
				}
			}
		}

		impl serde::Serialize for $name {
			fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
				match unsafe { crate::SERIALIZATION_CONFIG.enum_names } {
					true => format!("{:?}", self).serialize(serializer),
					_ => self.0.serialize(serializer),
				}
			}
		}

		impl peppi_arrow::Arrow for $name {
			type ArrowArray = <$type as peppi_arrow::Arrow>::ArrowArray;

			fn arrow_default() -> Self {
				<Self as Default>::default()
			}

			fn data_type<C: ::peppi_arrow::Context>(context: C) -> ::arrow2::datatypes::DataType {
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
