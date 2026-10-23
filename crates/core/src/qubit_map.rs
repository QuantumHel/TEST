use std::{
	collections::{BTreeMap, BTreeSet},
	fmt::Debug,
};

#[derive(Debug)]
pub struct QubitIsNotMapped {
	pub qubit: usize,
}

#[derive(Debug)]
pub struct ToQubitAlreadyMappedTo {
	pub from: usize,
	pub to: usize,
}

/// This maps some qubits to some other qubits.
///
/// The most common usecase is to map qubits at the start of the program, as
/// that is a free classical step.
#[derive(Default)]
pub struct PartialQubitMap {
	map: BTreeMap<usize, usize>,
	inverse: BTreeMap<usize, usize>,
}

impl PartialQubitMap {
	/// Sets that `from` maps to `to`. Overwrites old value of `from` if alread
	/// mapped.
	pub fn map_qubit(&mut self, from: usize, to: usize) -> Result<(), ToQubitAlreadyMappedTo> {
		if let Some(old_from) = self.inverse.get(&to) {
			if *old_from == from {
				return Ok(());
			}
			return Err(ToQubitAlreadyMappedTo {
				from: *old_from,
				to,
			});
		}
		if let Some(to) = self.map.insert(from, to) {
			self.inverse.remove(&to);
		}
		self.inverse.insert(to, from);
		Ok(())
	}

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
	pub fn map(&self, qubit: usize) -> Result<usize, QubitIsNotMapped> {
		if let Some(target) = self.map.get(&qubit) {
			return Ok(*target);
		}

		if self.inverse.contains_key(&qubit) {
			return Err(QubitIsNotMapped { qubit });
		}

		Ok(qubit)
	}

	/// Finds the source qubit of a mapped qubit
	pub fn source(&self, qubit: usize) -> Result<usize, QubitIsNotMapped> {
		if let Some(target) = self.inverse.get(&qubit) {
			return Ok(*target);
		}

		if self.map.contains_key(&qubit) {
			return Err(QubitIsNotMapped { qubit });
		}

		Ok(qubit)
	}

	pub fn non_trivial_mappings(&self) -> impl Iterator<Item = (usize, usize)> {
		self.map
			.iter()
			.map(|(&a, &b)| (a, b))
			.filter(|(a, b)| a != b)
	}

	/// Resets the mapping for the given qubit. This returns the qubit to either
	/// identity map, or unmapped if `qubit` is already mapped to.
	pub fn reset_qubit(&mut self, qubit: usize) {
		if let Some(to) = self.map.remove(&qubit) {
			self.inverse.remove(&to);
		}
	}
}

impl TryInto<QubitMap> for PartialQubitMap {
	type Error = QubitIsNotMapped;

	fn try_into(self) -> Result<QubitMap, Self::Error> {
		let mut removed: BTreeSet<usize> = BTreeSet::new();
		let mut filled: BTreeSet<usize> = BTreeSet::new();

		for (from, to) in self.map.iter() {
			removed.insert(*from);
			filled.insert(*to);
		}

		if removed != filled {
			for to in filled.iter() {
				removed.remove(to);
			}
			let not_mapped = removed.first().expect("hmm...");
			return Err(QubitIsNotMapped { qubit: *not_mapped });
		}

		Ok(QubitMap {
			map: self.map,
			inverse: self.inverse,
		})
	}
}

impl Debug for PartialQubitMap {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("QubitMap").field("map", &self.map).finish()
	}
}

/// This maps all qubits and is a bijection.
///
/// The most common usecase is to map qubits at the start of the program, as
/// that is a free classical step.
#[derive(Default, Clone)]
pub struct QubitMap {
	map: BTreeMap<usize, usize>,
	inverse: BTreeMap<usize, usize>,
}

impl Debug for QubitMap {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("TotalQubitMap")
			.field("map", &self.map)
			.finish()
	}
}

impl QubitMap {
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

	pub fn non_trivial_mappings(&self) -> impl Iterator<Item = (usize, usize)> {
		self.map
			.iter()
			.map(|(&a, &b)| (a, b))
			.filter(|(a, b)| a != b)
	}

	pub fn as_reversed(self) -> Self {
		Self {
			map: self.inverse,
			inverse: self.map,
		}
	}
}

impl From<QubitMap> for PartialQubitMap {
	fn from(value: QubitMap) -> Self {
		PartialQubitMap {
			map: value.map,
			inverse: value.inverse,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{PartialQubitMap, QubitMap};

	#[test]
	fn test1() {
		let mut mapping = QubitMap::default();
		mapping.swap_mapped(0, 1);
		mapping.swap_mapped(1, 2);
		for a in mapping.non_trivial_mappings() {
			println!("{a:?}");
		}
	}

	#[test]
	fn tmp() {
		let mut mapping = PartialQubitMap::default();
		mapping.map_qubit(1, 0).unwrap();
		assert!(mapping.map(0).is_err());
		assert!(mapping.source(1).is_err());
	}
}
