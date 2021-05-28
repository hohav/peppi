macro_rules! frame_data {
	($name: ident {
		$( $field: ident : $type: ty ),* $(,)?
	}, {
		$( $opt_field: ident : $opt_type: ty ),* $(,)?
	}) => {
		#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
		pub struct $name {
			$( pub $field: $type, )*
			$( #[serde(skip_serializing_if = "Option::is_none")] pub $opt_field: Option<$opt_type>, )*
		}

		impl super::arrow::Arrow for $name {
			fn arrow_buffers(&self, name: &str, len: usize) -> Vec<super::arrow::Buffer> {
				let mut buffers = Vec::new();
				$( {
					buffers.extend(self.$field.arrow_buffers(
						format!("{}.{}", name, stringify!($field).trim_start_matches("r#")).as_str(),
						len));
				} )*
				$( if let Some(f) = self.$opt_field {
					buffers.extend(f.arrow_buffers(
						format!("{}.{}", name, stringify!($opt_field).trim_start_matches("r#")).as_str(),
						len));
				} )*
				buffers
			}

			fn arrow_append(&self, buffers: &mut Vec<super::arrow::Buffer>, index: usize) -> usize {
				let mut offset = 0;
				$( offset += self.$field.arrow_append(buffers, index + offset); )*
				$( if let Some(f) = self.$opt_field {
					offset += f.arrow_append(buffers, index + offset);
				} )*
				offset
			}
		}
	}
}
