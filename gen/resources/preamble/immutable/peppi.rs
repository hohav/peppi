#![allow(unused_variables)]

use arrow2::{
	array::{Array, ListArray, PrimitiveArray, StructArray},
	datatypes::{DataType, Field},
	offset::OffsetsBuffer,
};

use crate::{
	io::slippi::{Version, STAGE_EVENTS_VERSION},
	frame::{
		immutable::{Data, Frame, PortData},
		PortOccupancy,
	},
	game::{Port, NUM_PORTS},
};

trait StructArrayConvertible {
	fn data_type(version: Version) -> DataType;
	fn into_struct_array(self, version: Version) -> StructArray;
	fn from_struct_array(array: StructArray, version: Version) -> Self;
}

impl Data {
	fn data_type(version: Version) -> DataType {
		DataType::Struct(vec![
			Field::new("pre", Pre::data_type(version).clone(), false),
			Field::new("post", Post::data_type(version).clone(), false),
		])
	}

	fn into_struct_array(self, version: Version) -> StructArray {
		let values = vec![
			self.pre.into_struct_array(version).boxed(),
			self.post.into_struct_array(version).boxed(),
		];
		StructArray::new(Self::data_type(version), values, self.validity)
	}

	fn from_struct_array(array: StructArray, version: Version) -> Self {
		let (_, values, validity) = array.into_data();
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
			validity: validity,
		}
	}
}

impl PortData {
	fn data_type(version: Version, port: PortOccupancy) -> DataType {
		let mut fields = vec![Field::new(
			"leader",
			Data::data_type(version).clone(),
			false,
		)];
		if port.follower {
			fields.push(Field::new(
				"follower",
				Data::data_type(version).clone(),
				false,
			));
		}
		DataType::Struct(fields)
	}

	fn into_struct_array(self, version: Version, port: PortOccupancy) -> StructArray {
		let mut values = vec![self.leader.into_struct_array(version).boxed()];
		if let Some(follower) = self.follower {
			values.push(follower.into_struct_array(version).boxed());
		}
		StructArray::new(Self::data_type(version, port), values, None)
	}

	fn from_struct_array(array: StructArray, version: Version, port: Port) -> Self {
		let (fields, values, _) = array.into_data();
		assert_eq!("leader", fields[0].name);
		fields.get(1).map(|f| assert_eq!("follower", f.name));
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
		}
	}
}

impl Frame {
	fn port_data_type(version: Version, ports: &[PortOccupancy]) -> DataType {
		DataType::Struct(
			ports.iter().map(|p| {
				Field::new(
					format!("{}", p.port),
					PortData::data_type(version, *p).clone(),
					false,
				)
			})
			.collect(),
		)
	}

	fn item_data_type(version: Version) -> DataType {
		DataType::List(Box::new(Field::new(
			"item",
			Item::data_type(version),
			false,
		)))
	}

	fn fod_platform_data_type(version: Version) -> DataType {
		DataType::List(Box::new(Field::new(
			"fod_platform",
			FodPlatform::data_type(version),
			false,
		)))
	}

	fn dreamland_whispy_data_type(version: Version) -> DataType {
		DataType::List(Box::new(Field::new(
			"dreamland_whispy",
			DreamlandWhispy::data_type(version),
			false,
		)))
	}

	fn stadium_transformation_data_type(version: Version) -> DataType {
		DataType::List(Box::new(Field::new(
			"stadium_transformation",
			StadiumTransformation::data_type(version),
			false,
		)))
	}

	fn data_type(version: Version, ports: &[PortOccupancy]) -> DataType {
		let mut fields = vec![
			Field::new("id", DataType::Int32, false),
			Field::new("ports", Self::port_data_type(version, ports).clone(), false),
		];
		if version.gte(2, 2) {
			fields.push(Field::new("start", Start::data_type(version).clone(), false));
			if version.gte(3, 0) {
				fields.push(Field::new("end", End::data_type(version).clone(), false));
				fields.push(Field::new("item", Self::item_data_type(version).clone(), false));
				if version >= STAGE_EVENTS_VERSION {
					fields.push(Field::new("fod_platform", Self::fod_platform_data_type(version).clone(), false));
					fields.push(Field::new("dreamland_whispy", Self::dreamland_whispy_data_type(version).clone(), false));
					fields.push(Field::new("stadium_transformation", Self::stadium_transformation_data_type(version).clone(), false));
				}
			}
		}
		DataType::Struct(fields)
	}

	pub fn into_struct_array(self, version: Version, ports: &[PortOccupancy]) -> StructArray {
		let values: Vec<_> = std::iter::zip(ports, self.ports)
			.map(|(occupancy, data)| data.into_struct_array(version, *occupancy).boxed())
			.collect();

		let mut arrays = vec![
			self.id.boxed(),
			StructArray::new(Self::port_data_type(version, ports), values, None).boxed(),
		];

		if version.gte(2, 2) {
			arrays.push(self.start.unwrap().into_struct_array(version).boxed());
			if version.gte(3, 0) {
				arrays.push(self.end.unwrap().into_struct_array(version).boxed());
				let item_values = self.item.unwrap().into_struct_array(version).boxed();
				arrays.push(ListArray::new(
					Self::item_data_type(version),
					self.item_offset.unwrap(),
					item_values,
					None,
				).boxed());
				if version >= STAGE_EVENTS_VERSION {
					let fod_platform_values = self.fod_platform.unwrap().into_struct_array(version).boxed();
					arrays.push(ListArray::new(
						Self::fod_platform_data_type(version),
						self.fod_platform_offset.unwrap(),
						fod_platform_values,
						None,
					).boxed());
					let dreamland_whispy_values = self.dreamland_whispy.unwrap().into_struct_array(version).boxed();
					arrays.push(ListArray::new(
						Self::dreamland_whispy_data_type(version),
						self.dreamland_whispy_offset.unwrap(),
						dreamland_whispy_values,
						None,
					).boxed());
					let stadium_transformation_values = self.stadium_transformation.unwrap().into_struct_array(version).boxed();
					arrays.push(ListArray::new(
						Self::stadium_transformation_data_type(version),
						self.stadium_transformation_offset.unwrap(),
						stadium_transformation_values,
						None,
					).boxed());
				}
			}
		}

		StructArray::new(Self::data_type(version, ports), arrays, None)
	}

	fn port_data_from_struct_array(array: StructArray, version: Version) -> Vec<PortData> {
		let (fields, values, _) = array.into_data();
		let mut ports = vec![];
		for i in 0 .. NUM_PORTS {
			if let Some(a) = values.get(i as usize) {
				ports.push(PortData::from_struct_array(
					a.as_any().downcast_ref::<StructArray>().unwrap().clone(),
					version,
					Port::parse(&fields[i as usize].name).unwrap(),
				));
			}
		}
		ports
	}

	fn values_and_offsets<T: StructArrayConvertible>(arr: &Box<dyn Array>, version: Version) -> (Option<T>, Option<OffsetsBuffer<i32>>) {
		let arrays = arr.as_any()
			.downcast_ref::<ListArray<i32>>()
			.unwrap()
			.clone();
		let offsets = arrays.offsets().clone();
		let values = T::from_struct_array(
			arrays.values()
				.as_any()
				.downcast_ref::<StructArray>()
				.unwrap()
				.clone(),
			version,
		);
		(Some(values), Some(offsets))
	}

	pub fn from_struct_array(array: StructArray, version: Version) -> Self {
		let (fields, values, _) = array.into_data();
		assert_eq!("id", fields[0].name);
		assert_eq!("ports", fields[1].name);
		if version.gte(2, 2) {
			assert_eq!("start", fields[2].name);
			if version.gte(3, 0) {
				assert_eq!("end", fields[3].name);
				assert_eq!("item", fields[4].name);
				if version >= STAGE_EVENTS_VERSION {
					assert_eq!("fod_platform", fields[5].name);
					assert_eq!("dreamland_whispy", fields[6].name);
					assert_eq!("stadium_transformation", fields[7].name);
				}
			}
		}

		let (item, item_offset) = values.get(4).map_or((None, None), |arr| Frame::values_and_offsets(arr, version));
		let (fod_platform, fod_platform_offset) = values.get(5).map_or((None, None), |arr| Frame::values_and_offsets(arr, version));
		let (dreamland_whispy, dreamland_whispy_offset) = values.get(6).map_or((None, None), |arr| Frame::values_and_offsets(arr, version));
		let (stadium_transformation, stadium_transformation_offset) = values.get(7).map_or((None, None), |arr| Frame::values_and_offsets(arr, version));

		Self {
			id: values[0]
				.as_any()
				.downcast_ref::<PrimitiveArray<i32>>()
				.unwrap()
				.clone(),
			ports: Self::port_data_from_struct_array(
				values[1]
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			),
			start: values.get(2).map(|v|
				Start::from_struct_array(
					v.as_any()
						.downcast_ref::<StructArray>()
						.unwrap()
						.clone(),
						version,
				)
			),
			end: values.get(3).map(|v|
				End::from_struct_array(
					v.as_any()
						.downcast_ref::<StructArray>()
						.unwrap()
						.clone(),
						version,
				)
			),
			item,
			item_offset,
			fod_platform,
			fod_platform_offset,
			dreamland_whispy,
			dreamland_whispy_offset,
			stadium_transformation,
			stadium_transformation_offset,
		}
	}
}
