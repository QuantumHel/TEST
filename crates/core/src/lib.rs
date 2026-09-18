use std::{
	collections::{BTreeMap, BTreeSet},
	fmt::Debug,
};

pub mod connectivity;

pub mod prelude {
	pub use super::Compiler;
	pub use super::connectivity::Connectivity;
}

pub trait Compiler<Input, Output, Device = ()>: Sized {
	fn compile(&self, input: Input, device: &Device) -> Output;
}

#[derive(Default, Debug)]
pub struct QubitMappingBuilder {
	map: BTreeMap<usize, usize>,
}

impl QubitMappingBuilder {
	/// Sets that `from` maps to `to`. Overwrites old value of `from` if alread
	/// mapped.
	pub fn map_qubit(&mut self, from: usize, to: usize) {
		self.map.insert(from, to);
	}

	/// Tries to create a [QubitMapping]. If ther resulting mapping maps two
	/// qubits on top of each other, the [QubitMappingBuilder] is returned as
	/// [Err].
	pub fn finish(self) -> Result<QubitMapping, Self> {
		let mut removed: BTreeSet<usize> = BTreeSet::new();
		let mut filled: BTreeSet<usize> = BTreeSet::new();

		let mut map: BTreeMap<usize, usize> = BTreeMap::new();
		let mut inverse: BTreeMap<usize, usize> = BTreeMap::new();
		for (from, to) in self.map.iter() {
			removed.insert(*from);
			if !filled.insert(*to) {
				return Err(self);
			}

			map.insert(*from, *to);
			inverse.insert(*to, *from);
		}

		if removed != filled {
			return Err(self);
		}

		Ok(QubitMapping { map, inverse })
	}
}

/// This maps some qubits to other qubits.
///
/// The most common usecase is to map qubits at the start of the program, as
/// that is a free classical step.
#[derive(Default)]
pub struct QubitMapping {
	map: BTreeMap<usize, usize>,
	inverse: BTreeMap<usize, usize>,
}

impl Debug for QubitMapping {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("QubitMapping")
			.field("map", &self.map)
			.finish()
	}
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

	pub fn map_non_trivial(&self) -> impl Iterator<Item = (usize, usize)> {
		self.map.iter().map(|(&a, &b)| (a, b))
	}

	pub fn as_reversed(self) -> Self {
		Self {
			map: self.inverse,
			inverse: self.map,
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::QubitMapping;

	#[test]
	fn test1() {
		let mut mapping = QubitMapping::default();
		mapping.swap_mapped(0, 1);
		mapping.swap_mapped(1, 2);
		for a in mapping.map_non_trivial() {
			println!("{a:?}");
		}
	}
}
