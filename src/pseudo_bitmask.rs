#[macro_export]
macro_rules! pseudo_bitmask {
	($name:ident : $type:ty { $( $value:expr => $variant:ident ),* $(,)? }) => {
		#[derive(PartialEq, Eq, Copy, Clone)]
		pub struct $name { pub value:$type }

		impl $name {
			$( pub const $variant:$name = $name { value:$value }; )*
		}

		impl std::fmt::Debug for $name {
			fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				let mut foo:Vec<&str> = Vec::new();
				$( if (self.value & $value) > 0 {
					foo.push(stringify!($variant));
				} )*
				write!(f, "{} => {:?}", self.value, foo)
			}
		}
	}
}
