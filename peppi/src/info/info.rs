macro_rules! info {
	($type: path => $info: ident {
		$($name: ident {
			$( $field: ident : $value: expr ),+ $(,)?
		});+ $(;)?
	}) => {
		impl $info {
			pub fn try_from(value: $type) -> Option<$info> {
				match value {
					$( <$type>::$name => Some($name), )*
					_ => None
				}
			}
		}

		$(pub const $name: $info = $info {
			$( $field: $value ),*
		};)*
	}
}
