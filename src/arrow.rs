use std::{
	convert::TryInto,
	mem::size_of,
	sync::Arc,
};

use arrow::{
	array,
	array::{ArrayData, ArrayRef, StructArray},
	buffer,
	datatypes::{
		self,
		DataType,
		Field,
		BooleanType,
		Int8Type,
		UInt8Type,
		Int16Type,
		UInt16Type,
		Int32Type,
		UInt32Type,
		Int64Type,
		UInt64Type,
		Float32Type,
	},
};

use super::{
	frame,
	game,
	slippi::{Slippi, Version},
	primitives::{Direction, Port}
};

#[derive(Clone, Copy, Debug)]
pub struct Opts {
	pub avro_compatible: bool,
	pub skip_items: bool,
}

pub(super) struct Buffer {
	pub arrow_buffer: buffer::MutableBuffer,
	pub validities: Option<array::BooleanBufferBuilder>,
	pub data_type: datatypes::DataType,
	pub path: Vec<String>,
}

impl Buffer {
	pub fn new(path: &Vec<String>, size: usize, data_type: DataType) -> Buffer {
		Buffer {
			arrow_buffer: buffer::MutableBuffer::new(size),
			validities: None,
			data_type: data_type,
			path: path.clone(),
		}
	}

	fn item_size(&self) -> usize {
		use datatypes::DataType::*;
		match self.data_type {
			Boolean => 1,
			Int8 | UInt8 => 1,
			Int16 | UInt16 => 2,
			Int32 | UInt32 => 4,
			Int64 | UInt64 => 8,
			Float32 => 4,
			_ => unimplemented!(),
		}
	}

	fn into_primitive_array(self) -> ArrayRef {
		let mut builder = ArrayData::builder(self.data_type.clone())
			.len(self.arrow_buffer.len() / self.item_size())
			.add_buffer(match self.data_type {
				DataType::Boolean => unsafe {
					buffer::MutableBuffer::from_trusted_len_iter_bool(
						self.arrow_buffer.into_iter().map(|x| *x != 0))
					.into()
				},
				_ => self.arrow_buffer.into(),
			});
		if let Some(mut validities) = self.validities {
			builder = builder.null_bit_buffer(validities.finish());
		}
		array::make_array(builder.build())
	}
}

pub fn clone_push<S: AsRef<str>>(path: &Vec<String>, s: S) -> Vec<String> {
	let mut path = path.clone();
	path.push(s.as_ref().to_string());
	path
}

pub trait ArrowPrimitive: Copy + Sized {
	type ArrowNativeType: datatypes::ArrowNativeType;
	type ArrowType;
	const ARROW_DATA_TYPE: DataType;
	fn into_arrow_native(self) -> Self::ArrowNativeType;
}

macro_rules! primitives {
	( $($t1:ty : $t2:ty),* $(,)? ) => { $(
		impl ArrowPrimitive for $t1 {
			type ArrowNativeType = $t1;
			type ArrowType = $t2;
			const ARROW_DATA_TYPE: DataType = <$t2 as datatypes::ArrowPrimitiveType>::DATA_TYPE;
			fn into_arrow_native(self) -> Self::ArrowNativeType { self }
		}
	)* }
}

primitives!(
	i8: Int8Type,
	u8: UInt8Type,
	i16: Int16Type,
	u16: UInt16Type,
	i32: Int32Type,
	u32: UInt32Type,
	i64: Int64Type,
	u64: UInt64Type,
	f32: Float32Type,
);

impl ArrowPrimitive for bool {
	type ArrowNativeType = u8;
	type ArrowType = BooleanType;
	const ARROW_DATA_TYPE: DataType = DataType::Boolean;

	fn into_arrow_native(self) -> Self::ArrowNativeType {
		match self {
			true => 1,
			_ => 0,
		}
	}
}

impl ArrowPrimitive for Port {
	type ArrowNativeType = u8;
	type ArrowType = UInt8Type;
	const ARROW_DATA_TYPE: DataType = DataType::UInt8;
	fn into_arrow_native(self) -> Self::ArrowNativeType { self as u8 }
}

impl ArrowPrimitive for Direction {
	type ArrowNativeType = u8;
	type ArrowType = BooleanType;
	const ARROW_DATA_TYPE: DataType = DataType::Boolean;

	fn into_arrow_native(self) -> Self::ArrowNativeType {
		match self {
			Self::Left => 0,
			_ => 1,
		}
	}
}

pub(super) trait Arrow {
	fn arrow_buffers(path: &Vec<String>, len: usize, slippi: Slippi, opts: Opts) -> Vec<Buffer>;
	fn arrow_append(&self, buffers: &mut Vec<Buffer>, index: usize, slippi: Slippi, opts: Opts) -> usize;
	fn arrow_append_null(buffers: &mut Vec<Buffer>, index: usize, slippi: Slippi, opts: Opts) -> usize;
}

impl<T> Arrow for T where T: ArrowPrimitive {
	fn arrow_buffers(path: &Vec<String>, len: usize, _slippi: Slippi, _opts: Opts) -> Vec<Buffer> {
		vec![Buffer::new(path, len * size_of::<T::ArrowNativeType>(), T::ARROW_DATA_TYPE)]
	}

	fn arrow_append(&self, buffers: &mut Vec<Buffer>, index: usize, _slippi: Slippi, _opts: Opts) -> usize {
		buffers[index].arrow_buffer.push(self.into_arrow_native());
		1
	}

	fn arrow_append_null(buffers: &mut Vec<Buffer>, index: usize, _slippi: Slippi, _opts: Opts) -> usize {
		buffers[index].arrow_buffer.extend_zeros(
			size_of::<T::ArrowNativeType>()
		);
		1
	}
}

impl<T> Arrow for Option<T> where T: Arrow {
	fn arrow_buffers(path: &Vec<String>, len: usize, slippi: Slippi, opts: Opts) -> Vec<Buffer> {
		let mut buffers = T::arrow_buffers(path, len, slippi, opts);
		buffers[0].validities = Some(array::BooleanBufferBuilder::new(len));
		buffers
	}

	fn arrow_append(&self, buffers: &mut Vec<Buffer>, index: usize, slippi: Slippi, opts: Opts) -> usize {
		if let Some(v) = self {
			buffers[index].validities.as_mut().unwrap().append(true);
			v.arrow_append(buffers, index, slippi, opts)
		} else {
			Self::arrow_append_null(buffers, index, slippi, opts)
		}
	}

	fn arrow_append_null(buffers: &mut Vec<Buffer>, index: usize, slippi: Slippi, opts: Opts) -> usize {
		buffers[index].validities.as_mut().unwrap().append(false);
		T::arrow_append_null(buffers, index, slippi, opts)
	}
}

impl<T, const N: usize> Arrow for [T; N] where T: Arrow {
	fn arrow_buffers(path: &Vec<String>, len: usize, slippi: Slippi, opts: Opts) -> Vec<Buffer> {
		let mut buffers = vec![Buffer::new(path, 0, DataType::Null)];
		let mut fields = vec![];

		for i in 0 .. N {
			let name = if opts.avro_compatible {
				format!("_{}", i)
			} else {
				format!("{}", i)
			};
			let bufs = T::arrow_buffers(
				&clone_push(path, name.clone()),
				len,
				slippi,
				opts,
			);
			fields.push(Field::new(&name, bufs[0].data_type.clone(), false));
			buffers.extend(bufs);
		}

		buffers[0].data_type = DataType::Struct(fields);
		buffers
	}

	fn arrow_append(&self, buffers: &mut Vec<Buffer>, index: usize, slippi: Slippi, opts: Opts) -> usize {
		let mut offset = 1;
		for i in 0 .. N {
			offset += self[i].arrow_append(buffers, index + offset, slippi, opts);
		}
		offset
	}

	fn arrow_append_null(buffers: &mut Vec<Buffer>, index: usize, slippi: Slippi, opts: Opts) -> usize {
		let mut offset = 1;
		for _ in 0 .. N {
			offset += T::arrow_append_null(buffers, index + offset, slippi, opts);
		}
		offset
	}
}

impl<T> Arrow for Vec<T> where T: Arrow {
	fn arrow_buffers(path: &Vec<String>, len: usize, slippi: Slippi, opts: Opts) -> Vec<Buffer> {
		let bufs = T::arrow_buffers(path, len, slippi, opts);
		let field = Field::new(
			bufs[0].path.last().unwrap(),
			bufs[0].data_type.clone(),
			bufs[0].validities.is_some(),
		);

		let mut buffers = vec![Buffer::new(path, 0, DataType::List(Box::new(field)))];
		buffers[0].arrow_buffer.push(0i32);
		buffers.extend(bufs);
		buffers
	}

	fn arrow_append(&self, buffers: &mut Vec<Buffer>, index: usize, slippi: Slippi, opts: Opts) -> usize {
		let buf = &mut buffers[index].arrow_buffer;
		let idx = buf.len() - size_of::<i32>();
		let last_offset = i32::from_le_bytes(buf[idx..].try_into().unwrap());
		buf.push(last_offset + self.len() as i32);

		let mut offset = 0;
		for x in self {
			let new_offset = x.arrow_append(buffers, index + 1, slippi, opts);
			if offset > 0 {
				assert_eq!(new_offset, offset);
			}
			offset = new_offset;
		}
		offset
	}

	fn arrow_append_null(_buffers: &mut Vec<Buffer>, _index: usize, _slippi: Slippi, _opts: Opts) -> usize {
		unimplemented!()
	}
}

impl Arrow for frame::PortData {
	fn arrow_buffers(path: &Vec<String>, len: usize, slippi: Slippi, opts: Opts) -> Vec<Buffer> {
		let mut buffers = vec![Buffer::new(path, 0, DataType::Null)];
		let mut fields = vec![];

		if true {
			let bufs = frame::Data::arrow_buffers(
				&clone_push(path, "leader"),
				len,
				slippi,
				opts,
			);
			fields.push(Field::new("leader", bufs[0].data_type.clone(), false));
			buffers.extend(bufs);
		}
		if true {
			let bufs = Option::<frame::Data>::arrow_buffers(
				&clone_push(path, "follower"),
				len,
				slippi,
				opts,
			);
			fields.push(Field::new("follower", bufs[0].data_type.clone(), true));
			buffers.extend(bufs);
		}

		buffers[0].data_type = DataType::Struct(fields);
		buffers
	}

	fn arrow_append(&self, buffers: &mut Vec<Buffer>, index: usize, slippi: Slippi, opts: Opts) -> usize {
		let mut offset = 1;
		offset += self.leader.arrow_append(buffers, index + offset, slippi, opts);
		if let Some(f) = &self.follower {
			offset += f.arrow_append(buffers, index + offset, slippi, opts);
		} else {
			offset += frame::Data::arrow_append_null(buffers, index + offset, slippi, opts);
		}
		offset
	}

	fn arrow_append_null(buffers: &mut Vec<Buffer>, index: usize, slippi: Slippi, opts: Opts) -> usize {
		let mut offset = 1;
		offset += frame::Data::arrow_append_null(buffers, index + offset, slippi, opts);
		offset += frame::Data::arrow_append_null(buffers, index + offset, slippi, opts);
		offset
	}
}

impl<const N: usize> Arrow for frame::Frame<N> {
	fn arrow_buffers(path: &Vec<String>, len: usize, slippi: Slippi, opts: Opts) -> Vec<Buffer> {
		let mut buffers = vec![Buffer::new(path, 0, DataType::Null)];
		let mut fields = vec![];

		if true {
			let bufs = <[frame::PortData; N]>::arrow_buffers(
				&clone_push(path, "ports"),
				len,
				slippi,
				opts,
			);
			fields.push(Field::new("ports", bufs[0].data_type.clone(), false));
			buffers.extend(bufs);
		}
		if slippi.version >= Version(2, 2, 0) {
			let bufs = frame::Start::arrow_buffers(
				&clone_push(path, "start"),
				len,
				slippi,
				opts,
			);
			fields.push(Field::new("start", bufs[0].data_type.clone(), false));
			buffers.extend(bufs);
		}
		if slippi.version >= Version(3, 0, 0) {
			let bufs = frame::End::arrow_buffers(
				&clone_push(path, "end"),
				len,
				slippi,
				opts,
			);
			fields.push(Field::new("end", bufs[0].data_type.clone(), false));
			buffers.extend(bufs);
		}
		if !opts.skip_items && slippi.version >= Version(3, 0, 0) {
			let bufs = Vec::<frame::Item>::arrow_buffers(
				&clone_push(path, "items"),
				len,
				slippi,
				opts,
			);
			fields.push(Field::new("items", bufs[0].data_type.clone(), false));
			buffers.extend(bufs);
		}

		buffers[0].data_type = DataType::Struct(fields);
		buffers
	}

	fn arrow_append(&self, buffers: &mut Vec<Buffer>, index: usize, slippi: Slippi, opts: Opts) -> usize {
		let mut offset = 1;
		offset += self.ports.arrow_append(buffers, index + offset, slippi, opts);
		if slippi.version >= Version(2, 2, 0) {
			offset += self.start.as_ref().unwrap().arrow_append(buffers, index + offset, slippi, opts);
		}
		if slippi.version >= Version(3, 0, 0) {
			offset += self.end.as_ref().unwrap().arrow_append(buffers, index + offset, slippi, opts);
		}
		if !opts.skip_items && slippi.version >= Version(3, 0, 0) {
			offset += self.items.as_ref().unwrap().arrow_append(buffers, index + offset, slippi, opts);
		}
		offset
	}

	fn arrow_append_null(_buffers: &mut Vec<Buffer>, _index: usize, _slippi: Slippi, _opts: Opts) -> usize {
		unimplemented!()
	}
}

fn pop<I: Iterator<Item = Buffer>>(buffers: &mut I) -> ArrayRef {
	use datatypes::DataType::*;
	let buf = buffers.next().unwrap();
	let array = match buf.data_type.clone() {
		Struct(fields) => {
			let mut children = vec![];
			for f in fields {
				let child = pop(buffers);
				children.push((f, child));
			}
			Arc::new(StructArray::from(children)) as ArrayRef
		},
		List(_) => {
			let child = pop(buffers);
			Arc::new(array::ListArray::from(
				ArrayData::builder(buf.data_type.clone())
					.len(buf.arrow_buffer.len() / size_of::<i32>() - 1)
					.add_buffer(buf.arrow_buffer.into())
					.add_child_data(child.data().clone())
					.build()
			)) as ArrayRef
		},
		Boolean | Int8 | UInt8 | Int16 | UInt16 | Int32 | UInt32 | Int64 | UInt64 | Float32 =>
			buf.into_primitive_array(),
		_ => unimplemented!(),
	};
	array
}

fn _frames<const N: usize>(frames: &Vec<frame::Frame<N>>, slippi: Slippi, opts: Opts) -> ArrayRef {
	let mut buffers = frame::Frame::<N>::arrow_buffers(
		&vec![String::new()], frames.len(), slippi, opts);
	for frame in frames {
		frame.arrow_append(&mut buffers, 0, slippi, opts);
	}
	pop(&mut buffers.into_iter())
}

pub fn frames(game: &game::Game, opts: Option<Opts>) -> ArrayRef {
	use game::Frames::*;
	let opts = opts.unwrap_or(Opts {
		avro_compatible: false,
		skip_items: false,
	});
	let slippi = game.start.slippi;
	match &game.frames {
		P1(f) => _frames(f, slippi, opts),
		P2(f) => _frames(f, slippi, opts),
		P3(f) => _frames(f, slippi, opts),
		P4(f) => _frames(f, slippi, opts),
	}
}

fn _items<const N: usize>(frames: &Vec<frame::Frame<N>>, slippi: Slippi, opts: Opts) -> Option<ArrayRef> {
	if slippi.version >= Version(3, 0, 0) {
		let len: usize = frames.iter().map(|f| f.items.as_ref().unwrap().len()).sum();
		let mut buffers = frame::Item::arrow_buffers(&vec![String::new()], len, slippi, opts);
		buffers.push(Buffer::new(
			&vec![String::new(), "frame_index".to_string()],
			len * size_of::<i32>(),
			DataType::UInt32,
		));
		for (idx, frame) in frames.iter().enumerate() {
			for item in frame.items.as_ref().unwrap() {
				let offset = item.arrow_append(&mut buffers, 0, slippi, opts);
				(idx as i32).arrow_append(&mut buffers, offset, slippi, opts);
			}
		}
		Some(pop(&mut buffers.into_iter()))
	} else {
		None
	}
}

pub fn items(game: &game::Game, opts: Option<Opts>) -> Option<ArrayRef> {
	use game::Frames::*;
	let opts = opts.unwrap_or(Opts {
		avro_compatible: false,
		skip_items: false,
	});
	let slippi = game.start.slippi;
	match &game.frames {
		P1(f) => _items(f, slippi, opts),
		P2(f) => _items(f, slippi, opts),
		P3(f) => _items(f, slippi, opts),
		P4(f) => _items(f, slippi, opts),
	}
}
