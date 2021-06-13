pub type Logical = f32;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, peppi_derive::Peppi)]
pub struct Physical {
	pub l: f32,
	pub r: f32,
}
