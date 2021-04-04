pub trait TransposeFrameRow<T> {
	fn transpose(&self, cols: &mut T);
}

pub trait TransposeFrameCol<T> {
	fn transpose(&self, n: usize) -> T;
	fn new(dim: usize, x: T) -> Self;
}

macro_rules! frame_data {
	($name: ident, $name_col: ident {
		$( $field: ident : $type: ty ),* $(,)? }
	$(, $ver: ident : $ver_type: ty, $ver_col: ty )? ) => {
		#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
		pub struct $name {
			$( pub $field: $type, )*
			$( #[serde(flatten)] pub $ver: Option<$ver_type> )?
		}

		impl super::frame_data::TransposeFrameRow<$name_col> for $name {
			fn transpose(&self, cols: &mut $name_col) {
				$( cols.$field.push(self.$field); )*
				$( if let Some(cols) = &mut cols.$ver {
					self.$ver.unwrap().transpose(cols);
				} )?
			}
		}

		#[derive(Debug)]
		pub struct $name_col {
			$( pub $field: Vec<$type>, )*
			$( pub $ver: Option<$ver_col> )?
		}

		impl super::frame_data::TransposeFrameCol<$name> for $name_col {
			fn transpose(&self, n: usize) -> $name {
				$name {
					$( $field: self.$field[n], )*
					$( $ver: self.$ver.as_ref().map(|ver| ver.transpose(n)) )?
				}
			}

			fn new(dim: usize, _x: $name) -> Self {
				Self {
					$( $field: Vec::with_capacity(dim), )*
					$( $ver: _x.$ver.map(|x| <$ver_col>::new(dim, x)) )?
				}
			}
		}
	}
}

pub trait TransposePortRow<T> {
	fn transpose(&self, cols: &mut T, port: usize);
}

pub trait TransposePortCol<T> {
	fn transpose(&self, port: usize, n: usize) -> T;
	fn new(dim: (usize, usize), x: T) -> Self;
}

macro_rules! port_data {
	($name: ident, $name_col: ident {
		$( $field: ident : $type: ty ),* $(,)?
	} $(, $ver: ident : $ver_type: ty, $ver_col: ty )? ) => {
		#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
		pub struct $name {
			$( pub $field: $type, )*
			$( pub $ver: Option<$ver_type> )?
		}

		impl super::frame_data::TransposePortRow<$name_col> for $name {
			fn transpose(&self, cols: &mut $name_col, port: usize) {
				$( cols.$field[port].push(self.$field); )*
				$( if let Some(cols) = &mut cols.$ver {
					self.$ver.unwrap().transpose(cols, port);
				} )?
			}
		}

		#[derive(Debug)]
		pub struct $name_col {
			$( pub $field: Vec<Vec<$type>>, )*
			$( pub $ver: Option<$ver_col> )?
		}

		impl super::frame_data::TransposePortCol<$name> for $name_col {
			fn transpose(&self, port: usize, n: usize) -> $name {
				$name {
					$( $field: self.$field[port][n], )*
					$( $ver: self.$ver.as_ref().map(|ver| ver.transpose(port, n)) )?
				}
			}

			fn new(dim: (usize, usize), _x: $name) -> Self {
				Self {
					$( $field: vec![Vec::with_capacity(dim.1); dim.0], )*
					$( $ver: _x.$ver.map(|x| <$ver_col>::new(dim, x)) )?
				}
			}
		}
	}
}

pub trait TransposeItemRow<T> {
	fn transpose(&self, cols: &mut T, index: i32);
}

pub trait TransposeItemCol<T> {
	fn transpose(&self, n: usize) -> T;
	fn new(dim: usize, x: T) -> Self;
}

macro_rules! item_data {
	($name: ident, $name_col: ident {
		$( $field: ident : $type: ty ),* $(,)? }
	$(, $ver: ident : $ver_type: ty, $ver_col: ty )? ) => {
		#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
		pub struct $name {
			$( pub $field: $type, )*
			$( #[serde(flatten)] pub $ver: Option<$ver_type> )?
		}

		impl super::frame_data::TransposeItemRow<$name_col> for $name {
			fn transpose(&self, cols: &mut $name_col, index: i32) {
				cols.index.push(index);
				$( cols.$field.push(self.$field); )*
				$( if let Some(cols) = &mut cols.$ver {
					self.$ver.unwrap().transpose(cols, index);
				} )?
			}
		}

		#[derive(Debug)]
		pub struct $name_col {
			pub index: Vec<i32>,
			$( pub $field: Vec<$type>, )*
			$( pub $ver: Option<$ver_col> )?
		}

		impl super::frame_data::TransposeItemCol<$name> for $name_col {
			fn transpose(&self, n: usize) -> $name {
				$name {
					$( $field: self.$field[n], )*
					$( $ver: self.$ver.as_ref().map(|ver| ver.transpose(n)) )?
				}
			}

			fn new(dim: usize, _x: $name) -> Self {
				Self {
					index: Vec::with_capacity(dim),
					$( $field: Vec::with_capacity(dim), )*
					$( $ver: _x.$ver.map(|x| <$ver_col>::new(dim, x)) )?
				}
			}
		}
	}
}
