/// The analog trigger value the game engine uses to calculate the next frame
pub type Logical = f32;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, peppi_derive::Arrow)]
/// The analog trigger values read by the console
pub struct Physical {
	pub l: f32,
	pub r: f32,
}
