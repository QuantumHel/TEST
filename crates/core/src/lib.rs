use std::collections::BTreeMap;

pub mod connectivity;
mod disjoint_set_forest;

pub mod prelude {
	pub use super::Compiler;
	pub use super::connectivity::{
		Connectivity, Edge, Graph, GraphExt, Node, Subedge, Subgraph, steiner_tree,
	};
}

pub trait Compiler<Input, Output, Device = ()>: Sized {
	fn compile(&self, input: Input, device: &Device) -> Output;
}

/// This maps some qubits to other qubits.
///
/// The most common usecase is to map qubits at the start of the program, as
/// that is a free classical step.
pub struct QubitMapping {
	map: BTreeMap<usize, usize>,
	inverse: BTreeMap<usize, usize>,
}

impl QubitMapping {
	/// Swaps two mapped qubits.
	///
	/// Returs the updated physical qubit locations `(a, b)`, which are
	/// identical when the inputs are identical.
	pub fn swap_mapped(&mut self, a: usize, b: usize) -> (usize, usize) {
		let a_source = self.inverse.get(&a).copied().unwrap_or(a);
		let b_source = self.inverse.get(&b).copied().unwrap_or(b);
		self.map.insert(a_source, b);
		self.map.insert(b_source, a);

		self.inverse.insert(b, a_source);
		self.inverse.insert(a, b_source);

		(b_source, a_source)
	}

	/// Maps a qubit according to the mapping
	pub fn map(&self, qubit: usize) -> usize {
		self.map.get(&qubit).copied().unwrap_or(qubit)
	}

	/// Finds the source qubit of a mapped qubit
	pub fn source(&self, qubit: usize) -> usize {
		self.inverse.get(&qubit).copied().unwrap_or(qubit)
	}
}
