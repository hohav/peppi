#[macro_export]
macro_rules! pseudo_enum {
	($name:ident : $type:ty { $( $value:expr => $variant:ident ),* $(,)? }) => {
		#[derive(PartialEq, Eq, Copy, Clone)]
		pub struct $name { pub value:$type }

		impl $name {
			$( pub const $variant:$name = $name { value:$value }; )*
		}

		impl std::fmt::Debug for $name {
			fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match self.value {
					$( $value => write!(f, "{}:{}", stringify!($value), stringify!($variant)), )*
					_ => write!(f, "{}", self.value),
				}
			}
		}
	}
}
