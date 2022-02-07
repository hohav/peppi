use arrow2::{
	array::{
		Array,
		BooleanArray,
		ListArray,
		FixedSizeListArray,
		MutableArray,
		MutableBooleanArray,
		MutableFixedSizeListArray,
		MutableListArray,
		MutablePrimitiveArray,
		PrimitiveArray,
	},
	datatypes::{DataType, Field},
};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct SlippiVersion (pub u8, pub u8, pub u8);

pub trait Context: Copy {
	fn slippi_version(&self) -> SlippiVersion;
}

pub trait Arrow {
	type ArrowArray: MutableArray + 'static;

	fn data_type<C: Context>(context: C) -> DataType;
	fn is_nullable() -> bool { false }
	fn arrow_array<C: Context>(context: C) -> Self::ArrowArray;

	fn arrow_push(&self, array: &mut dyn MutableArray);
	fn arrow_push_null(array: &mut dyn MutableArray);

	fn arrow_read(&mut self, array: &dyn Array, idx: usize);

	fn arrow_default() -> Self; // workaround for Default not working with const generics
}

macro_rules! primitives {
	( $($type: ty : $arrow_type: expr),* $(,)? ) => { $(
		impl Arrow for $type {
			type ArrowArray = MutablePrimitiveArray<$type>;

			fn data_type<C: Context>(_context: C) -> DataType {
				$arrow_type
			}

			fn arrow_array<C: Context>(_context: C) -> Self::ArrowArray {
				Self::ArrowArray::new()
			}

			fn arrow_push(&self, array: &mut dyn MutableArray) {
				array.as_mut_any().downcast_mut::<Self::ArrowArray>().unwrap().push(Some(*self));
			}

			fn arrow_push_null(array: &mut dyn MutableArray) {
				array.push_null();
			}

			fn arrow_read(&mut self, array: &dyn Array, idx: usize) {
				*self = array.as_any().downcast_ref::<PrimitiveArray<$type>>().unwrap().value(idx);
			}

			fn arrow_default() -> Self {
				<$type>::default()
			}
		}
	)* }
}

primitives!(
	i8: DataType::Int8,
	u8: DataType::UInt8,
	i16: DataType::Int16,
	u16: DataType::UInt16,
	i32: DataType::Int32,
	u32: DataType::UInt32,
	i64: DataType::Int64,
	u64: DataType::UInt64,
	f32: DataType::Float32,
);

impl Arrow for bool {
	type ArrowArray = MutableBooleanArray;

	fn data_type<C: Context>(_context: C) -> DataType {
		DataType::Boolean
	}

	fn arrow_array<C: Context>(_context: C) -> Self::ArrowArray {
		Self::ArrowArray::new()
	}

	fn arrow_push(&self, array: &mut dyn MutableArray) {
		array.as_mut_any().downcast_mut::<Self::ArrowArray>().unwrap().push(Some(*self));
	}

	fn arrow_push_null(array: &mut dyn MutableArray) {
		array.push_null();
	}

	fn arrow_read(&mut self, array: &dyn Array, idx: usize) {
		*self = array.as_any().downcast_ref::<BooleanArray>().unwrap().value(idx);
	}

	fn arrow_default() -> Self {
		Self::default()
	}
}

impl<T> Arrow for Option<T> where T: Arrow {
	type ArrowArray = T::ArrowArray;

	fn data_type<C: Context>(context: C) -> DataType {
		T::data_type(context)
	}

	fn arrow_array<C: Context>(context: C) -> Self::ArrowArray {
		T::arrow_array(context)
	}

	fn is_nullable() -> bool {
		true
	}

	fn arrow_push(&self, array: &mut dyn MutableArray) {
		if let Some(value) = self {
			value.arrow_push(array);
		} else {
			T::arrow_push_null(array);
		}
	}

	fn arrow_push_null(array: &mut dyn MutableArray) {
		T::arrow_push_null(array);
	}

	fn arrow_read(&mut self, array: &dyn Array, idx: usize) {
		if array.is_valid(idx) {
			let mut value = T::arrow_default();
			value.arrow_read(array, idx);
			*self = Some(value);
		} // no `else` b/c we assume `*self` was initialized to None
	}

	fn arrow_default() -> Self {
		Self::default()
	}
}

impl<T> Arrow for Box<T> where T: Arrow {
	type ArrowArray = T::ArrowArray;

	fn data_type<C: Context>(context: C) -> DataType {
		T::data_type(context)
	}

	fn arrow_array<C: Context>(context: C) -> Self::ArrowArray {
		T::arrow_array(context)
	}

	fn arrow_push(&self, array: &mut dyn MutableArray) {
		(**self).arrow_push(array);
	}

	fn arrow_push_null(array: &mut dyn MutableArray) {
		T::arrow_push_null(array);
	}

	fn arrow_read(&mut self, array: &dyn Array, idx: usize) {
		(**self).arrow_read(array, idx);
	}

	fn arrow_default() -> Self {
		Box::new(T::arrow_default())
	}
}

impl<T> Arrow for Vec<T> where T: Arrow {
	type ArrowArray = MutableListArray<i32, T::ArrowArray>;

	fn data_type<C: Context>(context: C) -> DataType {
		DataType::List(Box::new(Field::new("item", T::data_type(context), false)))
	}

	fn arrow_array<C: Context>(context: C) -> Self::ArrowArray {
		Self::ArrowArray::new_from(T::arrow_array(context), Self::data_type(context), 0)
	}

	fn arrow_push(&self, array: &mut dyn MutableArray) {
		let array = array.as_mut_any().downcast_mut::<Self::ArrowArray>().unwrap();
		for x in self {
			x.arrow_push(array.mut_values());
		}
		array.try_push_valid().unwrap();
	}

	fn arrow_push_null(array: &mut dyn MutableArray) {
		array.push_null();
	}

	fn arrow_read(&mut self, array: &dyn Array, idx: usize) {
		let slice = array.as_any().downcast_ref::<ListArray<i32>>().unwrap().value(idx);
		for i in 0 .. slice.len() {
			let mut value = T::arrow_default();
			value.arrow_read(&*slice, i);
			self.push(value);
		}
	}

	fn arrow_default() -> Self {
		Self::default()
	}
}

impl<T, const N: usize> Arrow for [T; N] where T: Arrow {
	type ArrowArray = MutableFixedSizeListArray<T::ArrowArray>;

	fn data_type<C: Context>(context: C) -> DataType {
		let field = Field::new("item", T::data_type(context), T::is_nullable());
		DataType::FixedSizeList(Box::new(field), usize::try_from(N).unwrap())
	}

	fn arrow_array<C: Context>(context: C) -> Self::ArrowArray {
		Self::ArrowArray::new_from(T::arrow_array(context), Self::data_type(context), N)
	}

	fn arrow_push(&self, array: &mut dyn MutableArray) {
		let array = array.as_mut_any().downcast_mut::<Self::ArrowArray>().unwrap();
		for x in self {
			x.arrow_push(array.mut_values());
		}
		array.try_push_valid().unwrap();
	}

	fn arrow_push_null(array: &mut dyn MutableArray) {
		array.push_null();
	}

	fn arrow_read(&mut self, array: &dyn Array, idx: usize) {
		let slice = array.as_any().downcast_ref::<FixedSizeListArray>().unwrap().value(idx);
		for i in 0 .. slice.len() {
			self[i].arrow_read(slice.as_ref(), i);
		}
	}

	fn arrow_default() -> Self {
		[(); N].map(|_| {
			T::arrow_default()
		})
	}
}
