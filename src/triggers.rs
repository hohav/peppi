pub type Logical = f32;

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize)]
pub struct Physical {
	pub l: f32,
	pub r: f32,
}
