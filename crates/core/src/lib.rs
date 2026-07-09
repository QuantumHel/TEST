pub mod connectivity;
mod disjoint_set_forest;

pub mod prelude {
	pub use super::Compiler;
	pub use super::connectivity::{
		Connectivity, Edge, Graph, Node, Subedge, Subgraph, steiner_tree,
	};
}

pub trait Compiler<Input, Output, Device = ()>: Sized {
	fn compile(&self, input: Input, device: &Device) -> Output;
}
