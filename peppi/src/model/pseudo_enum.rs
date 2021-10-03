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

macro_rules! pseudo_enum {
	($name: ident : $type: ty { $( $value: expr => $variant: ident ),* $(,)? }) => {
		#[derive(Copy, Clone, Default, PartialEq, Eq, Hash, serde::Deserialize)]
		#[serde(transparent)]
		pub struct $name(pub $type);

		impl $name {
			$( pub const $variant:$name = $name($value); )*
		}

		impl std::convert::TryFrom<&str> for $name {
			type Error = crate::model::pseudo_enum::ConversionError;
			fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
				match s {
					$( stringify!($variant) => Ok($name::$variant), )*
					_ => Err(Self::Error {
						r#type: format!("{}::{}", module_path!(), stringify!($name)),
						value: s.to_string(),
					}),
				}
			}
		}

		impl std::fmt::Debug for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match unsafe { crate::SERIALIZATION_CONFIG.enum_names } {
					true => match self.0 {
						$( $value => write!(f, "{}:{}", self.0, stringify!($variant)), )*
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
