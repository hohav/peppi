use arrow::{
	array::{
		ArrayBuilder,
		BooleanBuilder,
		ListBuilder,
		PrimitiveBuilder,
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
	fn fields<C: Context>(_context: C) -> Vec<Field> { unimplemented!() }
	fn data_type<C: Context>(context: C) -> DataType;
	fn is_nullable() -> bool { false }
	fn builder<C: Context>(len: usize, context: C) -> Self::Builder;
	fn append<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C);
	fn append_null<C: Context>(builder: &mut dyn ArrayBuilder, context: C);
}

macro_rules! primitives {
	( $($type: ty : $arrow_type: ty),* $(,)? ) => { $(
		impl Arrow for $type {
			type Builder = PrimitiveBuilder::<$arrow_type>;

			fn data_type<C: Context>(_context: C) -> DataType {
				<$arrow_type>::DATA_TYPE
			}

			fn builder<C: Context>(len: usize, _context: C) -> Self::Builder {
				PrimitiveBuilder::<$arrow_type>::new(len)
			}

			fn append<C: Context>(&self, builder: &mut dyn ArrayBuilder, _context: C) {
				builder.as_any_mut().downcast_mut::<PrimitiveBuilder::<$arrow_type>>().unwrap().append_value(*self).unwrap()
			}

			fn append_null<C: Context>(builder: &mut dyn ArrayBuilder, _context: C) {
				builder.as_any_mut().downcast_mut::<PrimitiveBuilder::<$arrow_type>>().unwrap().append_null().unwrap()
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

	fn data_type<C: Context>(_context: C) -> DataType {
		DataType::Boolean
	}

	fn builder<C: Context>(len: usize, _context: C) -> Self::Builder {
		BooleanBuilder::new(len)
	}

	fn append<C: Context>(&self, builder: &mut dyn ArrayBuilder, _context: C) {
		builder.as_any_mut().downcast_mut::<BooleanBuilder>().unwrap().append_value(*self).unwrap()
	}

	fn append_null<C: Context>(builder: &mut dyn ArrayBuilder, _context: C) {
		builder.as_any_mut().downcast_mut::<BooleanBuilder>().unwrap().append_null().unwrap()
	}
}

impl<T> Arrow for Option<T> where T: Arrow {
	type Builder = T::Builder;

	fn data_type<C: Context>(context: C) -> DataType {
		T::data_type(context)
	}

	fn is_nullable() -> bool {
		true
	}

	fn builder<C: Context>(len: usize, context: C) -> Self::Builder {
		T::builder(len, context)
	}

	fn append<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C) {
		if let Some(value) = self {
			value.append(builder, context)
		} else {
			T::append_null(builder, context)
		}
	}

	fn append_null<C: Context>(builder: &mut dyn ArrayBuilder, context: C) {
		T::append_null(builder, context)
	}
}

impl<T> Arrow for Box<T> where T: Arrow {
	type Builder = T::Builder;

	fn data_type<C: Context>(context: C) -> DataType {
		T::data_type(context)
	}

	fn builder<C: Context>(len: usize, context: C) -> Self::Builder {
		T::builder(len, context)
	}

	fn append<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C) {
		(**self).append(builder, context)
	}

	fn append_null<C: Context>(builder: &mut dyn ArrayBuilder, context: C) {
		T::append_null(builder, context)
	}
}

impl<T, const N: usize> Arrow for [T; N] where T: Arrow {
	type Builder = StructBuilder;

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
		::arrow::array::StructBuilder::new(fields, builders)
	}

	fn append<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C) {
		let builder = builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap();
		for i in 0 .. N {
			self[i].append(builder.field_builder::<T::Builder>(i).unwrap(), context);
		}
		builder.append(true).unwrap();
	}

	fn append_null<C: Context>(builder: &mut dyn ArrayBuilder, context: C) {
		let builder = builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap();
		for i in 0 .. N {
			T::append_null(builder.field_builder::<T::Builder>(i).unwrap(), context);
		}
		builder.append(false).unwrap();
	}
}

impl<T> Arrow for Vec<T> where T: Arrow {
	type Builder = ListBuilder<T::Builder>;

	fn data_type<C: Context>(context: C) -> DataType {
		DataType::List(Box::new(Field::new("list", T::data_type(context), T::is_nullable())))
	}

	fn builder<C: Context>(len: usize, context: C) -> Self::Builder {
		Self::Builder::new(T::builder(len, context))
	}

	fn append<C: Context>(&self, builder: &mut dyn ArrayBuilder, context: C) {
		let builder = builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap();
		for x in self {
			x.append(builder.values(), context);
		}
		builder.append(true).unwrap();
	}

	fn append_null<C: Context>(builder: &mut dyn ArrayBuilder, _context: C) {
		let builder = builder.as_any_mut().downcast_mut::<Self::Builder>().unwrap();
		builder.append(false).unwrap();
	}
}
