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
			fn arrow_buffers(path: &Vec<String>, len: usize, slippi: super::slippi::Slippi, opts: super::arrow::Opts) -> Vec<super::arrow::Buffer> {
				use ::arrow::datatypes::{DataType, Field};

				let mut buffers = vec![super::arrow::Buffer::new(path, 0, DataType::Null)];
				let mut fields = vec![];

				$( {
					let name = stringify!($field).trim_start_matches("r#");
					let bufs = <$type>::arrow_buffers(
						&super::arrow::clone_push(path, name),
						len,
						slippi,
						opts,
					);
					fields.push(Field::new(name, bufs[0].data_type.clone(), false));
					buffers.extend(bufs);
				} )*
				$( if slippi.version >= $slippi_ver {
					let name = stringify!($opt_field).trim_start_matches("r#");
					let bufs = <$opt_type>::arrow_buffers(
						&super::arrow::clone_push(path, name),
						len,
						slippi,
						opts,
					);
					fields.push(Field::new(name, bufs[0].data_type.clone(), false));
					buffers.extend(bufs);
				} )*

				buffers[0].data_type = DataType::Struct(fields);
				buffers
			}

			fn arrow_append(&self, buffers: &mut Vec<super::arrow::Buffer>, index: usize, slippi: super::slippi::Slippi, opts: super::arrow::Opts) -> usize {
				let mut offset = 1;
				$( offset += self.$field.arrow_append(buffers, index + offset, slippi, opts); )*
				$( if slippi.version >= $slippi_ver {
					offset += self.$opt_field.as_ref().unwrap().arrow_append(buffers, index + offset, slippi, opts);
				} )*
				offset
			}

			fn arrow_append_null(buffers: &mut Vec<super::arrow::Buffer>, index: usize, slippi: super::slippi::Slippi, opts: super::arrow::Opts) -> usize {
				let mut offset = 1;
				$( offset += <$type>::arrow_append_null(buffers, index + offset, slippi, opts); )*
				$( if slippi.version >= $slippi_ver {
					offset += <$opt_type>::arrow_append_null(buffers, index + offset, slippi, opts);
				} )*
				offset
			}
		}
	}
}
