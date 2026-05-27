#![allow(unused_variables)]

use crate::{
	frame::{Data, Frame, PortData, PortOccupancy, Validity},
	game::{NUM_PORTS, Port},
	io::slippi::Version,
};

use arrow::{
	array::{Array, ArrayRef, ListArray, PrimitiveArray, StructArray, downcast_array},
	buffer::{NullBuffer, OffsetBuffer, ScalarBuffer},
	datatypes::{
		DataType, Field, Fields, Float32Type, Int32Type, Int8Type, UInt16Type, UInt32Type,
		UInt8Type,
	},
};

use std::sync::Arc;

trait StructArrayConvertible {
	fn fields(version: Version) -> Fields;
	fn into_struct_array(self, version: Version) -> StructArray;
	fn from_struct_array(array: StructArray, version: Version) -> Self;

	fn data_type(version: Version) -> DataType {
		DataType::Struct(Self::fields(version))
	}
}

impl From<Option<NullBuffer>> for Validity {
	fn from(nullbuf: Option<NullBuffer>) -> Validity {
		let mut validity = Validity::with_capacity(nullbuf.as_ref().map_or(0, |v| v.len()));
		if let Some(nullbuf) = nullbuf.as_ref() {
			for value in nullbuf.iter() {
				validity.push(value);
			}
		}
		validity
	}
}

impl From<Validity> for Option<NullBuffer> {
	fn from(validity: Validity) -> Option<NullBuffer> {
		if validity.values.capacity() > 0 {
			Some(NullBuffer::from(validity.values))
		} else {
			None
		}
	}
}

impl StructArrayConvertible for Data {
	fn fields(version: Version) -> Fields {
		Fields::from(vec![
			Field::new("pre", Pre::data_type(version).clone(), false),
			Field::new("post", Post::data_type(version).clone(), false),
		])
	}

	fn into_struct_array(self, version: Version) -> StructArray {
		let values = vec![
			Arc::new(self.pre.into_struct_array(version)) as ArrayRef,
			Arc::new(self.post.into_struct_array(version)) as ArrayRef,
		];
		StructArray::new(
			Self::fields(version),
			values,
			self.validity.into(),
		)
	}

	fn from_struct_array(array: StructArray, version: Version) -> Self {
		let (_, values, validity) = array.into_parts();
		Self {
			pre: Pre::from_struct_array(
				values[0]
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			),
			post: Post::from_struct_array(
				values[1]
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			),
			validity: validity.into(),
		}
	}
}

impl PortData {
	fn fields(version: Version, port: PortOccupancy) -> Fields {
		let mut fields = vec![Field::new(
			"leader",
			Data::data_type(version).clone(),
			false,
		)];
		if port.follower {
			fields.push(Field::new(
				"follower",
				Data::data_type(version).clone(),
				true,
			));
		}
		Fields::from(fields)
	}

	fn data_type(version: Version, port: PortOccupancy) -> DataType {
		DataType::Struct(Self::fields(version, port))
	}

	fn into_struct_array(self, version: Version, port: PortOccupancy) -> StructArray {
		let mut values = vec![Arc::new(self.leader.into_struct_array(version)) as ArrayRef];
		if let Some(follower) = self.follower {
			values.push(Arc::new(follower.into_struct_array(version)) as ArrayRef);
		}
		StructArray::new(Self::fields(version, port), values, self.validity.into())
	}

	fn from_struct_array(array: StructArray, version: Version, port: Port) -> Self {
		let (fields, values, validity) = array.into_parts();
		assert_eq!("leader", fields[0].name());
		fields.get(1).map(|f| assert_eq!("follower", f.name()));
		Self {
			port: port,
			leader: Data::from_struct_array(
				values[0]
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			),
			follower: values.get(1).map(|x| {
				Data::from_struct_array(
					x.as_any().downcast_ref::<StructArray>().unwrap().clone(),
					version,
				)
			}),
			validity: validity.into(),
		}
	}
}

impl Frame {
	fn port_fields(version: Version, ports: &[PortOccupancy]) -> Fields {
		Fields::from(
			ports
				.iter()
				.map(|p| {
					Field::new(
						format!("{}", p.port),
						PortData::data_type(version, *p).clone(),
						true,
					)
				})
				.collect::<Vec<_>>(),
		)
	}

	fn item_field(version: Version) -> Arc<Field> {
		Arc::new(Field::new("item", Item::data_type(version), false))
	}

	fn fod_platform_field(version: Version) -> Arc<Field> {
		Arc::new(Field::new(
			"fod_platform",
			FodPlatform::data_type(version),
			false,
		))
	}

	fn dreamland_whispy_field(version: Version) -> Arc<Field> {
		Arc::new(Field::new(
			"dreamland_whispy",
			DreamlandWhispy::data_type(version),
			false,
		))
	}

	fn stadium_transformation_field(version: Version) -> Arc<Field> {
		Arc::new(Field::new(
			"stadium_transformation",
			StadiumTransformation::data_type(version),
			false,
		))
	}

	fn fields(version: Version, ports: &[PortOccupancy]) -> Fields {
		let mut fields = vec![
			Field::new("id", DataType::Int32, false),
			Field::new(
				"ports",
				DataType::Struct(Self::port_fields(version, ports)),
				false,
			),
		];
		if version.gte(2, 2) {
			fields.push(Field::new(
				"start",
				Start::data_type(version).clone(),
				false,
			));
			if version.gte(3, 0) {
				fields.push(Field::new("end", End::data_type(version).clone(), false));
				fields.push(Field::new(
					"item",
					DataType::List(Self::item_field(version)),
					false,
				));
				if version.gte(3, 18) {
					fields.push(Field::new(
						"fod_platform",
						DataType::List(Self::fod_platform_field(version)),
						false,
					));
					fields.push(Field::new(
						"dreamland_whispy",
						DataType::List(Self::dreamland_whispy_field(version)),
						false,
					));
					fields.push(Field::new(
						"stadium_transformation",
						DataType::List(Self::stadium_transformation_field(version)),
						false,
					));
				}
			}
		}
		Fields::from(fields)
	}

	pub fn into_struct_array(self, version: Version, ports: &[PortOccupancy]) -> StructArray {
		let values: Vec<_> = std::iter::zip(ports, self.ports)
			.map(|(occupancy, data)| {
				Arc::new(data.into_struct_array(version, *occupancy)) as ArrayRef
			})
			.collect();

		let mut arrays = vec![
			Arc::new(PrimitiveArray::<Int32Type>::from(self.id)) as ArrayRef,
			Arc::new(StructArray::new(
				Self::port_fields(version, ports),
				values,
				None,
			)) as ArrayRef,
		];

		if version.gte(2, 2) {
			arrays.push(Arc::new(self.start.unwrap().into_struct_array(version)));
			if version.gte(3, 0) {
				arrays.push(Arc::new(self.end.unwrap().into_struct_array(version)));
				let item_values = Arc::new(self.item.unwrap().into_struct_array(version));
				arrays.push(Arc::new(ListArray::new(
					Self::item_field(version),
					OffsetBuffer::new(ScalarBuffer::from(self.item_offset.unwrap())),
					item_values,
					None,
				)) as ArrayRef);
				if version.gte(3, 18) {
					let fod_platform_values =
						Arc::new(self.fod_platform.unwrap().into_struct_array(version));
					arrays.push(Arc::new(ListArray::new(
						Self::fod_platform_field(version),
						OffsetBuffer::new(ScalarBuffer::from(self.fod_platform_offset.unwrap())),
						fod_platform_values,
						None,
					)) as ArrayRef);
					let dreamland_whispy_values =
						Arc::new(self.dreamland_whispy.unwrap().into_struct_array(version));
					arrays.push(Arc::new(ListArray::new(
						Self::dreamland_whispy_field(version),
						OffsetBuffer::new(ScalarBuffer::from(self.dreamland_whispy_offset.unwrap())),
						dreamland_whispy_values,
						None,
					)) as ArrayRef);
					let stadium_transformation_values = Arc::new(
						self.stadium_transformation
							.unwrap()
							.into_struct_array(version),
					);
					arrays.push(Arc::new(ListArray::new(
						Self::stadium_transformation_field(version),
						OffsetBuffer::new(ScalarBuffer::from(self.stadium_transformation_offset.unwrap())),
						stadium_transformation_values,
						None,
					)) as ArrayRef);
				}
			}
		}

		StructArray::new(Self::fields(version, ports), arrays, None)
	}

	fn port_data_from_struct_array(array: StructArray, version: Version) -> Vec<PortData> {
		let (fields, values, _) = array.into_parts();
		let mut ports = vec![];
		for i in 0..NUM_PORTS {
			if let Some(a) = values.get(i as usize) {
				ports.push(PortData::from_struct_array(
					a.as_any().downcast_ref::<StructArray>().unwrap().clone(),
					version,
					Port::parse(&fields[i as usize].name()).unwrap(),
				));
			}
		}
		ports
	}

	fn values_and_offsets<T: StructArrayConvertible>(
		arr: &Arc<dyn Array>,
		version: Version,
	) -> (Option<T>, Option<OffsetBuffer<i32>>) {
		let list_array = downcast_array::<ListArray>(arr);
		let (_, offsets, values, _) = list_array.into_parts();
		let values = T::from_struct_array(downcast_array::<StructArray>(&values), version);
		(Some(values), Some(offsets))
	}

	pub fn from_struct_array(array: StructArray, version: Version) -> Self {
		// TODO: check that we're not doing any unnecessary copying
		let (fields, values, _) = array.into_parts();
		assert_eq!("id", fields[0].name());
		assert_eq!("ports", fields[1].name());
		if version.gte(2, 2) {
			assert_eq!("start", fields[2].name());
			if version.gte(3, 0) {
				assert_eq!("end", fields[3].name());
				assert_eq!("item", fields[4].name());
				if version.gte(3, 18) {
					assert_eq!("fod_platform", fields[5].name());
					assert_eq!("dreamland_whispy", fields[6].name());
					assert_eq!("stadium_transformation", fields[7].name());
				}
			}
		}

		let (item, item_offset) = values
			.get(4)
			.map_or((None, None), |arr| Frame::values_and_offsets(arr, version));
		let (fod_platform, fod_platform_offset) = values
			.get(5)
			.map_or((None, None), |arr| Frame::values_and_offsets(arr, version));
		let (dreamland_whispy, dreamland_whispy_offset) = values
			.get(6)
			.map_or((None, None), |arr| Frame::values_and_offsets(arr, version));
		let (stadium_transformation, stadium_transformation_offset) = values
			.get(7)
			.map_or((None, None), |arr| Frame::values_and_offsets(arr, version));

		Self {
			id: downcast_array::<PrimitiveArray<Int32Type>>(values[0].as_ref())
				.into_parts()
				.1
				.into(),
			ports: Self::port_data_from_struct_array(
				values[1]
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			),
			start: values.get(2).map(|v| {
				Start::from_struct_array(
					v.as_any().downcast_ref::<StructArray>().unwrap().clone(),
					version,
				)
			}),
			end: values.get(3).map(|v| {
				End::from_struct_array(
					v.as_any().downcast_ref::<StructArray>().unwrap().clone(),
					version,
				)
			}),
			item,
			item_offset: item_offset
				.map(|buf| buf.into_inner().into_inner().typed_data().into()),
			fod_platform,
			fod_platform_offset: fod_platform_offset
				.map(|buf| buf.into_inner().into_inner().typed_data().into()),
			dreamland_whispy,
			dreamland_whispy_offset: dreamland_whispy_offset
				.map(|buf| buf.into_inner().into_inner().typed_data().into()),
			stadium_transformation,
			stadium_transformation_offset: stadium_transformation_offset
				.map(|buf| buf.into_inner().into_inner().typed_data().into()),
		}
	}
}
