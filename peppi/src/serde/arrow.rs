use arrow2::{
	array::{Array, MutableArray, StructArray},
	datatypes::DataType,
};

use crate::{
	model::{
		enums::action_state,
		frame,
		game,
		primitives::{Direction, Port},
	},
};

use peppi_arrow::{Arrow, Context, SlippiVersion};

#[derive(Clone, Copy, Debug)]
pub struct Opts {
	pub avro_compatible: bool,
}

macro_rules! arrow {
	( $( $type: ty : $arrow_type: ty ),* $(,)? ) => {
		$(
		impl Arrow for $type {
			type ArrowArray = <$arrow_type as Arrow>::ArrowArray;

			fn arrow_default() -> Self {
				<Self as Default>::default()
			}

			fn data_type<C: Context>(context: C) -> DataType {
				<$arrow_type>::data_type(context)
			}

			fn arrow_array<C: Context>(context: C) -> Self::ArrowArray {
				<$arrow_type>::arrow_array(context)
			}

			fn is_nullable() -> bool {
				<$arrow_type>::is_nullable()
			}

			fn arrow_push(&self, array: &mut dyn MutableArray) {
				<$arrow_type>::from(*self).arrow_push(array)
			}

			fn arrow_push_null(builder: &mut dyn MutableArray) {
				<$arrow_type>::arrow_push_null(builder)
			}

			fn arrow_read(&mut self, array: &dyn Array, idx: usize) {
				let mut x = <$arrow_type as Arrow>::arrow_default();
				x.arrow_read(array, idx);
				*self = <$type>::try_from(x).unwrap();
			}
		}
		)*
	}
}

arrow!(
	Port: u8,
	Direction: u8,
	action_state::State: u16,
);

#[derive(Clone, Copy, Debug)]
struct PeppiContext {
	slippi_version: SlippiVersion,
}

impl Context for PeppiContext {
	fn slippi_version(&self) -> SlippiVersion {
		self.slippi_version
	}
}

fn context(game: &game::Game, _opts: Option<Opts>) -> PeppiContext {
	let v = game.start.slippi.version;
	PeppiContext {
		slippi_version: SlippiVersion(v.0, v.1, v.2),
	}
}

fn _frames_to_arrow<const N: usize>(frames: &[frame::Frame<N>], context: PeppiContext) -> StructArray {
	let mut array = frame::Frame::<N>::arrow_array(context);
	for frame in frames {
		frame.arrow_push(&mut array);
	}
	array.into()
}

/// Convert a game's frame data to an Arrow StructArray
pub fn frames_to_arrow(game: &game::Game, opts: Option<Opts>) -> StructArray {
	use game::Frames::*;
	let c = context(game, opts);
	match &game.frames {
		P1(f) => _frames_to_arrow(f, c),
		P2(f) => _frames_to_arrow(f, c),
		P3(f) => _frames_to_arrow(f, c),
		P4(f) => _frames_to_arrow(f, c),
	}
}

fn _frames_from_arrow<const N: usize>(array: &dyn Array) -> Vec<frame::Frame<N>> {
	let mut frames = Vec::new();
	for i in 0 .. array.len() {
		frames.push(frame::Frame::<N>::arrow_default());
		frames[i].arrow_read(array, i);
	}
	frames
}

pub fn frames_from_arrow(frames: &dyn Array) -> game::Frames {
	let frames = frames.as_any().downcast_ref::<StructArray>().expect("expected a `StructArray`");
	let mut ports_data_type = Option::<DataType>::None;
	for f in frames.fields() {
		if f.name == "ports" {
			ports_data_type = Some(f.data_type.clone());
			break;
		}
	}
	match ports_data_type.expect("expected `ports`") {
		DataType::FixedSizeList(_, len) => match len {
			1 => game::Frames::P1(_frames_from_arrow(frames)),
			2 => game::Frames::P2(_frames_from_arrow(frames)),
			3 => game::Frames::P3(_frames_from_arrow(frames)),
			4 => game::Frames::P4(_frames_from_arrow(frames)),
			n => panic!("unsupported number of ports: {}", n),
		},
		x => panic!("expected `FixedSizeList`, got: {:?}", x),
	}
}
