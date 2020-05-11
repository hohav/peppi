macro_rules! pseudo_enum {
	($name:ident : $type:ty { $( $value:expr => $variant:ident ),* $(,)? }) => {
		#[derive(Copy, Clone, PartialEq, Eq, Hash, serde::Serialize)]
		pub struct $name(pub $type);

		impl $name {
			$( pub const $variant:$name = $name($value); )*
		}

		impl std::fmt::Debug for $name {
			fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match self.0 {
					$( $value => write!(f, "{}:{}", stringify!($value), stringify!($variant)), )*
					_ => write!(f, "{}", self.0),
				}
			}
		}
	}
}
