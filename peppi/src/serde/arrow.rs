use arrow::{
	array::{
		ArrayRef,
		ArrayBuilder,
		StructArray,
	},
	datatypes::DataType,
};

use crate::{
	model::{
		enums::action_state,
		frame,
		game,
		item,
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
			type Builder = <$arrow_type as Arrow>::Builder;

			fn default() -> Self {
				<Self as Default>::default()
			}

			fn data_type<C: Context>(context: C) -> DataType {
				<$arrow_type>::data_type(context)
			}

			fn is_nullable() -> bool {
				<$arrow_type>::is_nullable()
			}

			fn builder<C: Context>(len: usize, context: C) -> Self::Builder {
				<$arrow_type>::builder(len, context)
			}

			fn write<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C) {
				<$arrow_type>::from(*self).write(builder, context)
			}

			fn write_null<C: Context>(builder: &mut dyn ArrayBuilder, context: C) {
				<$arrow_type>::write_null(builder, context)
			}

			fn read(&mut self, array: ArrayRef, idx: usize) {
				let mut x = <$arrow_type as Arrow>::default();
				x.read(array, idx);
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

fn context(game: &game::Game, opts: Option<Opts>) -> PeppiContext {
	let v = game.start.slippi.version;
	PeppiContext {
		slippi_version: SlippiVersion(v.0, v.1, v.2),
		avro_compatible: opts.map(|o| o.avro_compatible).unwrap_or(false),
	}
}

fn _frames_to_arrow<const N: usize>(frames: &[frame::Frame<N>], context: PeppiContext) -> StructArray {
	let mut builder = frame::Frame::<N>::builder(frames.len(), context);
	for frame in frames {
		frame.write(&mut builder, context);
	}
	builder.finish()
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

#[derive(peppi_derive::Arrow)]
struct FrameItem {
	frame_index: u32,
	item: item::Item,
}

fn _items_to_arrow<const N: usize>(frames: &[frame::Frame<N>], context: PeppiContext) -> Option<StructArray> {
	if frames[0].items.is_some() {
		let len = frames.iter().map(|f| f.items.as_ref().unwrap().len()).sum();
		let mut builder = FrameItem::builder(len, context);
		for (idx, frame) in frames.iter().enumerate() {
			for item in frame.items.as_ref().unwrap() {
				FrameItem {
					frame_index: u32::try_from(idx).unwrap(),
					item: *item,
				}.write(&mut builder, context);
			}
		}
		Some(builder.finish())
	} else {
		None
	}
}

/// Workaround for bugs in Parquet's ListArray support.
/// Normally items would be part of the frame data.
pub fn items_to_arrow(game: &game::Game, opts: Option<Opts>) -> Option<StructArray> {
	use game::Frames::*;
	let c = context(game, opts);
	match &game.frames {
		P1(f) => _items_to_arrow(f, c),
		P2(f) => _items_to_arrow(f, c),
		P3(f) => _items_to_arrow(f, c),
		P4(f) => _items_to_arrow(f, c),
	}
}

/*
fn _frames_from_arrow<const N: usize>(arr: ArrayRef) -> Vec<frame::Frame<N>> {
	let mut frames = Vec::new();
	for i in 0 .. arr.len() {
		frames.push(frame::Frame::<N>::default());
		frames[i].read(arr.clone(), i);
	}
	frames
}

pub fn game_from_arrow(frames: ArrayRef) -> game::Game {
	let ports_data_type = frames.as_any().downcast_ref::<StructArray>().expect("expected a `StructArray`").column_by_name("ports").expect("expected `ports`").data_type();
	match ports_data_type {
		DataType::Struct(fields) => {
			let frames = match fields.len() {
				1 => game::Frames::P1(_frames_from_arrow(frames)),
				2 => game::Frames::P2(_frames_from_arrow(frames)),
				3 => game::Frames::P3(_frames_from_arrow(frames)),
				4 => game::Frames::P4(_frames_from_arrow(frames)),
				_ => unimplemented!(),
			};
			game::Game {
				start: game::Start::default(), //TODO
				end: game::End::default(), //TODO
				frames: frames,
				metadata: crate::model::metadata::Metadata::default(), //TODO
				metadata_raw: serde_json::Map::new(), //TODO
				gecko_codes: None, //TODO
			}
		},
		_ => unimplemented!(),
	}
}
*/
