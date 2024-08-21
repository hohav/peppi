#![allow(unused_variables)]

use std::sync::Arc;

use arrow::{
	array::{
		types::{Float32Type, Int32Type, Int8Type, UInt16Type, UInt32Type, UInt8Type},
		Array, ArrayRef, ListArray, PrimitiveArray, StructArray,
	},
	datatypes::{DataType, Field, Fields},
};

use crate::{
	frame::{
		immutable::{Data, Frame, PortData},
		PortOccupancy,
	},
	game::{Port, NUM_PORTS},
	io::slippi::Version,
};

impl Data {
	fn fields(version: Version) -> Fields {
		Fields::from(vec![
			Field::new("pre", Pre::data_type(version).clone(), false),
			Field::new("post", Post::data_type(version).clone(), false),
		])
	}

	fn data_type(version: Version) -> DataType {
		DataType::Struct(Self::fields(version))
	}

	fn into_struct_array(self, version: Version) -> StructArray {
		let values = vec![
			Arc::new(self.pre.into_struct_array(version)) as ArrayRef,
			Arc::new(self.post.into_struct_array(version)) as ArrayRef,
		];
		StructArray::new(Self::fields(version), values, self.validity)
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
			validity: validity,
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
				false,
			));
		}
		Fields::from(fields)
	}

	fn data_type(version: Version, port: PortOccupancy) -> DataType {
		DataType::Struct(Self::fields(version, port))
	}

	fn into_struct_array(self, version: Version, port: PortOccupancy) -> StructArray {
		let mut values = vec![Arc::new(self.leader.into_struct_array(version)) as Arc<dyn Array>];
		if let Some(follower) = self.follower {
			values.push(Arc::new(follower.into_struct_array(version)) as Arc<dyn Array>);
		}
		StructArray::new(Self::fields(version, port), values, None)
	}

	fn from_struct_array(array: StructArray, version: Version, port: Port) -> Self {
		let (fields, values, _) = array.into_parts();
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
						false,
					)
				})
				.collect::<Vec<_>>(),
		)
	}

	fn port_data_type(version: Version, ports: &[PortOccupancy]) -> DataType {
		DataType::Struct(Self::port_fields(version, ports))
	}

	fn item_field(version: Version) -> Field {
		Field::new("item", Item::data_type(version), false)
	}

	fn item_data_type(version: Version) -> DataType {
		DataType::List(Arc::new(Self::item_field(version)))
	}

	fn fields(version: Version, ports: &[PortOccupancy]) -> Fields {
		let mut fields = vec![
			Field::new("id", DataType::Int32, false),
			Field::new("ports", Self::port_data_type(version, ports), false),
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
					Self::item_data_type(version).clone(),
					false,
				));
			}
		}
		Fields::from(fields)
	}

	pub fn into_struct_array(self, version: Version, ports: &[PortOccupancy]) -> StructArray {
		let values: Vec<_> = std::iter::zip(ports, self.ports)
			.map(|(occupancy, data)| {
				Arc::new(data.into_struct_array(version, *occupancy)) as Arc<dyn Array>
			})
			.collect();

		let mut arrays = vec![
			Arc::new(self.id) as Arc<dyn Array>,
			Arc::new(StructArray::new(
				Self::port_fields(version, ports),
				values,
				None,
			)) as Arc<dyn Array>,
		];

		if version.gte(2, 2) {
			arrays.push(Arc::new(self.start.unwrap().into_struct_array(version)) as Arc<dyn Array>);
			if version.gte(3, 0) {
				arrays
					.push(Arc::new(self.end.unwrap().into_struct_array(version)) as Arc<dyn Array>);
				let item_values = Arc::new(self.item.unwrap().into_struct_array(version));
				arrays.push(Arc::new(ListArray::new(
					Arc::new(Self::item_field(version)),
					self.item_offset.unwrap(),
					item_values,
					None,
				)) as Arc<dyn Array>);
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

	pub fn from_struct_array(array: StructArray, version: Version) -> Self {
		let (fields, values, _) = array.into_parts();
		assert_eq!("id", fields[0].name());
		assert_eq!("ports", fields[1].name());
		if version.gte(2, 2) {
			assert_eq!("start", fields[2].name());
			if version.gte(3, 0) {
				assert_eq!("end", fields[3].name());
				assert_eq!("item", fields[4].name());
			}
		}

		let (item, item_offset) = values.get(4).map_or((None, None), |v| {
			let arrays = v.as_any().downcast_ref::<ListArray>().unwrap().clone();
			let item_offset = arrays.offsets().clone();
			let item = Item::from_struct_array(
				arrays
					.values()
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			);
			(Some(item), Some(item_offset))
		});

		Self {
			id: values[0]
				.as_any()
				.downcast_ref::<PrimitiveArray<Int32Type>>()
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
			item_offset: item_offset,
			item: item,
		}
	}
}
