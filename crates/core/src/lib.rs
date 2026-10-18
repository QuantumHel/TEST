pub mod connectivity;
pub mod qubit_map;

pub mod prelude {
	pub use super::Compiler;
	pub use super::connectivity::Connectivity;
}

pub trait Compiler<Input, Output, Device = ()>: Sized {
	fn compile(&self, input: Input, device: &Device) -> Output;
}
