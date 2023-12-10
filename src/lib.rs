macro_rules! err {
	($( $arg: expr ),*) => {
		std::io::Error::new(std::io::ErrorKind::InvalidData, format!($( $arg ),*))
	}
}

pub mod model {
	pub mod frame;
	pub mod game;
	pub mod shift_jis;
}

pub mod io;
