use std::mem::MaybeUninit;

use arrow::{
	array::{
		ArrayRef,
		ArrayBuilder,
		BooleanArray,
		BooleanBuilder,
		ListArray,
		ListBuilder,
		PrimitiveArray,
		PrimitiveBuilder,
		StructArray,
		StructBuilder,
	},
	datatypes::{
		ArrowPrimitiveType,
		DataType,
		Field,
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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct SlippiVersion (pub u8, pub u8, pub u8);

pub trait Context: Copy {
	fn slippi_version(&self) -> SlippiVersion;
	fn avro_compatible_field_names(&self) -> bool { false }
}

pub trait Arrow {
	type Builder: ArrayBuilder;
	fn default() -> Self; // workaround for Default not working with const generics yet
	fn fields<C: Context>(_context: C) -> Vec<Field> { unimplemented!() }
	fn data_type<C: Context>(context: C) -> DataType;
	fn is_nullable() -> bool { false }
	fn builder<C: Context>(len: usize, context: C) -> Self::Builder;
	fn write<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C);
	fn write_null<C: Context>(builder: &mut dyn ArrayBuilder, context: C);
	fn read(&mut self, array: ArrayRef, idx: usize);
}

macro_rules! primitives {
	( $($type: ty : $arrow_type: ty),* $(,)? ) => { $(
		impl Arrow for $type {
			type Builder = PrimitiveBuilder::<$arrow_type>;

			fn default() -> Self {
				0 as $type
			}

			fn data_type<C: Context>(_context: C) -> DataType {
				<$arrow_type>::DATA_TYPE
			}

			fn builder<C: Context>(len: usize, _context: C) -> Self::Builder {
				PrimitiveBuilder::<$arrow_type>::new(len)
			}

			fn write<C: Context>(&self, builder: &mut dyn ArrayBuilder, _context: C) {
				builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap().append_value(*self)
			}

			fn write_null<C: Context>(builder: &mut dyn ArrayBuilder, _context: C) {
				builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap().append_null()
			}

			fn read(&mut self, array: ArrayRef, idx: usize) {
				*self = array.as_any().downcast_ref::<PrimitiveArray::<$arrow_type>>().unwrap().value(idx);
			}
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

impl Arrow for bool {
	type Builder = BooleanBuilder;

	fn default() -> Self {
		false
	}

	fn data_type<C: Context>(_context: C) -> DataType {
		DataType::Boolean
	}

	fn builder<C: Context>(len: usize, _context: C) -> Self::Builder {
		BooleanBuilder::new(len)
	}

	fn write<C: Context>(&self, builder: &mut dyn ArrayBuilder, _context: C) {
		builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap().append_value(*self)
	}

	fn write_null<C: Context>(builder: &mut dyn ArrayBuilder, _context: C) {
		builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap().append_null()
	}

	fn read(&mut self, array: ArrayRef, idx: usize) {
		*self = array.as_any().downcast_ref::<BooleanArray>().unwrap().value(idx)
	}
}

impl<T> Arrow for Option<T> where T: Arrow {
	type Builder = T::Builder;

	fn default() -> Self {
		None
	}

	fn data_type<C: Context>(context: C) -> DataType {
		T::data_type(context)
	}

	fn is_nullable() -> bool {
		true
	}

	fn builder<C: Context>(len: usize, context: C) -> Self::Builder {
		T::builder(len, context)
	}

	fn write<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C) {
		if let Some(value) = self {
			value.write(builder, context)
		} else {
			T::write_null(builder, context)
		}
	}

	fn write_null<C: Context>(builder: &mut dyn ArrayBuilder, context: C) {
		T::write_null(builder, context)
	}

	fn read(&mut self, array: ArrayRef, idx: usize) {
		*self = match array.is_valid(idx) {
			true => {
				let mut value = T::default();
				value.read(array, idx);
				Some(value)
			},
			_ => None,
		};
	}
}

impl<T> Arrow for Box<T> where T: Arrow {
	type Builder = T::Builder;

	fn default() -> Self {
		Box::new(T::default())
	}

	fn data_type<C: Context>(context: C) -> DataType {
		T::data_type(context)
	}

	fn builder<C: Context>(len: usize, context: C) -> Self::Builder {
		T::builder(len, context)
	}

	fn write<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C) {
		(**self).write(builder, context)
	}

	fn write_null<C: Context>(builder: &mut dyn ArrayBuilder, context: C) {
		T::write_null(builder, context)
	}

	fn read(&mut self, array: ArrayRef, idx: usize) {
		(*self).read(array, idx);
	}
}

impl<T> Arrow for Vec<T> where T: Arrow {
	type Builder = ListBuilder<T::Builder>;

	fn default() -> Self {
		Vec::new()
	}

	fn data_type<C: Context>(context: C) -> DataType {
		DataType::List(Box::new(Field::new("list", T::data_type(context), T::is_nullable())))
	}

	fn builder<C: Context>(len: usize, context: C) -> Self::Builder {
		Self::Builder::new(T::builder(len, context))
	}

	fn write<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C) {
		let builder = builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap();
		for x in self {
			x.write(builder.values(), context);
		}
		builder.append(true);
	}

	fn write_null<C: Context>(builder: &mut dyn ArrayBuilder, _context: C) {
		let builder = builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap();
		builder.append(false);
	}

	fn read(&mut self, array: ArrayRef, idx: usize) {
		let array = array.as_any().downcast_ref::<ListArray>().unwrap();
		for i in 0 .. array.value_length(idx) {
			let mut value = T::default();
			value.read(array.value(idx), i as usize);
			self.push(value);
		}
	}
}

/// TODO: replace with FixedSizeListArray once Parquet supports those
impl<T, const N: usize> Arrow for [T; N] where T: Arrow {
	type Builder = StructBuilder;

	fn default() -> Self {
		let mut data: [MaybeUninit<T>; N] = unsafe {
			MaybeUninit::uninit().assume_init()
		};
		for elem in &mut data[..] {
			*elem = MaybeUninit::new(T::default())
		}
		//unsafe { mem::transmute::<_, [T; N]>(data) }
		unsafe { data.as_ptr().cast::<[T; N]>().read() }
	}

	fn fields<C: Context>(context: C) -> Vec<Field> {
		let mut fields = vec![];
		for i in 0 .. N {
			let name = match context.avro_compatible_field_names() {
				true => format!("_{}", i),
				_ => format!("{}", i),
			};
			fields.push(Field::new(&name, T::data_type(context), T::is_nullable()));
		}
		fields
	}

	fn data_type<C: Context>(context: C) -> DataType {
		DataType::Struct(Self::fields(context))
	}

	fn builder<C: Context>(len: usize, context: C) -> Self::Builder {
		let fields = Self::fields(context);
		let mut builders = vec![];
		for _ in 0 .. N {
			builders.push(Box::new(T::builder(len, context)) as Box<dyn ArrayBuilder>);
		}
		StructBuilder::new(fields, builders)
	}

	fn write<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C) {
		let builder = builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap();
		for (i, x) in self.iter().enumerate() {
			x.write(builder.field_builder::<T::Builder>(i).unwrap(), context);
		}
		builder.append(true);
	}

	fn write_null<C: Context>(builder: &mut dyn ArrayBuilder, context: C) {
		let builder = builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap();
		for i in 0 .. N {
			T::write_null(builder.field_builder::<T::Builder>(i).unwrap(), context);
		}
		builder.append(false);
	}

	fn read(&mut self, array: ArrayRef, idx: usize) {
		let struct_array = array.as_any().downcast_ref::<StructArray>().unwrap();
		for (i, x) in self.iter_mut().enumerate() {
			x.read(struct_array.column(i).clone(), idx);
		}
	}
}

/* For use when Parquet supports fixed-size lists
impl<T, const N: usize> Arrow for [T; N] where T: Arrow {
	type Builder = FixedSizeListBuilder<T::Builder>;

	fn default() -> Self {
		let mut data: [MaybeUninit<T>; N] = unsafe {
			MaybeUninit::uninit().assume_init()
		};
		for elem in &mut data[..] {
			*elem = MaybeUninit::new(T::default())
		}
		//unsafe { mem::transmute::<_, [T; N]>(data) }
		unsafe { data.as_ptr().cast::<[T; N]>().read() }
	}

	fn data_type<C: Context>(context: C) -> DataType {
		let field = Field::new("values", T::data_type(context), T::is_nullable());
		DataType::FixedSizeList(Box::new(field), i32::try_from(N).unwrap())
	}

	fn builder<C: Context>(len: usize, context: C) -> Self::Builder {
		Self::Builder::new(T::builder(len, context), i32::try_from(N).unwrap())
	}

	fn write<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C) {
		let builder = builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap();
		for i in 0 .. N {
			self[i].write(builder.values(), context);
		}
		builder.append(true);
	}

	fn write_null<C: Context>(builder: &mut dyn ArrayBuilder, context: C) {
		let builder = builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap();
		builder.append(false);
	}

	fn read(&mut self, array: ArrayRef, idx: usize) {
		let struct_array = array.as_any().downcast_ref::<StructArray>().unwrap();
		for i in 0 .. N {
			self[i].read(struct_array.column(i).clone(), idx);
		}
	}
}
*/
