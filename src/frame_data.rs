macro_rules! frame_data {
	($name: ident {
		$( $field: ident : $type: ty ),* $(,)?
	}, {
		$( $opt_field: ident : $opt_type: ty : $slippi_ver: expr ),* $(,)?
	}) => {
		#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
		pub struct $name {
			$( pub $field: $type, )*
			$( #[serde(skip_serializing_if = "Option::is_none")] pub $opt_field: Option<$opt_type>, )*
		}

		impl super::arrow::Arrow for $name {
			fn arrow_buffers(name: &str, len: usize, slippi: super::slippi::Slippi) -> Vec<super::arrow::Buffer> {
				let mut buffers = Vec::new();
				$( {
					buffers.extend(<$type>::arrow_buffers(
						format!("{}.{}", name, stringify!($field).trim_start_matches("r#")).as_str(),
						len,
						slippi,
					));
				} )*
				$( if slippi.version >= $slippi_ver {
					buffers.extend(<$opt_type>::arrow_buffers(
						format!("{}.{}", name, stringify!($opt_field).trim_start_matches("r#")).as_str(),
						len,
						slippi,
					));
				} )*
				buffers
			}

			fn arrow_append(&self, buffers: &mut Vec<super::arrow::Buffer>, index: usize, slippi: super::slippi::Slippi) -> usize {
				let mut offset = 0;
				$( offset += self.$field.arrow_append(buffers, index + offset, slippi); )*
				$( if slippi.version >= $slippi_ver {
					offset += self.$opt_field.unwrap().arrow_append(buffers, index + offset, slippi);
				} )*
				offset
			}
		}
	}
}
