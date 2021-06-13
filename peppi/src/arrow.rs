use arrow::{
	array::{
		ArrayBuilder,
		StructArray,
	},
	datatypes::DataType,
};

use super::{
	action_state,
	frame,
	game,
	primitives::{Direction, Port},
};

use peppi_arrow::{Arrow, Context, SlippiVersion};
use peppi_derive::Peppi;

#[derive(Clone, Copy, Debug)]
pub struct Opts {
	pub avro_compatible: bool,
}

macro_rules! arrow {
	( $( $type: ty : $arrow_type: ty ),* $(,)? ) => {
		$(
		impl Arrow for $type {
			type Builder = <$arrow_type as Arrow>::Builder;

			fn data_type<C: Context>(context: C) -> DataType {
				<$arrow_type>::data_type(context)
			}

			fn is_nullable() -> bool {
				<$arrow_type>::is_nullable()
			}

			fn builder<C: Context>(len: usize, context: C) -> Self::Builder {
				<$arrow_type>::builder(len, context)
			}

			fn append<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C) {
				<$arrow_type>::from(*self).append(builder, context)
			}

			fn append_null<C: Context>(builder: &mut dyn ArrayBuilder, context: C) {
				<$arrow_type>::append_null(builder, context)
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
	avro_compatible: bool,
}

impl Context for PeppiContext {
	fn slippi_version(&self) -> SlippiVersion {
		self.slippi_version
	}

	fn avro_compatible_field_names(&self) -> bool {
		self.avro_compatible
	}
}

fn _frames<const N: usize>(frames: &Vec<frame::Frame<N>>, context: PeppiContext) -> StructArray {
	let mut builder = frame::Frame::<N>::builder(frames.len(), context);
	for frame in frames {
		frame.append(&mut builder, context);
	}
	builder.finish()
}

fn context(game: &game::Game, opts: Option<Opts>) -> PeppiContext {
	let v = game.start.slippi.version;
	PeppiContext {
		slippi_version: SlippiVersion(v.0, v.1, v.2),
		avro_compatible: opts.map(|o| o.avro_compatible).unwrap_or(false),
	}
}

pub fn frames(game: &game::Game, opts: Option<Opts>) -> StructArray {
	use game::Frames::*;
	let c = context(game, opts);
	match &game.frames {
		P1(f) => _frames(f, c),
		P2(f) => _frames(f, c),
		P3(f) => _frames(f, c),
		P4(f) => _frames(f, c),
	}
}

#[derive(Peppi)]
struct FrameItem {
	frame_index: u32,
	item: frame::Item,
}

fn _items<const N: usize>(frames: &Vec<frame::Frame<N>>, context: PeppiContext) -> Option<StructArray> {
	if frames[0].items.is_some() {
		let len = frames.iter().map(|f| f.items.as_ref().unwrap().len()).sum();
		let mut builder = FrameItem::builder(len, context);
		for (idx, frame) in frames.iter().enumerate() {
			for item in frame.items.as_ref().unwrap() {
				FrameItem {
					frame_index: idx as u32,
					item: *item,
				}.append(&mut builder, context);
			}
		}
		Some(builder.finish())
	} else {
		None
	}
}

pub fn items(game: &game::Game, opts: Option<Opts>) -> Option<StructArray> {
	use game::Frames::*;
	let c = context(game, opts);
	match &game.frames {
		P1(f) => _items(f, c),
		P2(f) => _items(f, c),
		P3(f) => _items(f, c),
		P4(f) => _items(f, c),
	}
}
