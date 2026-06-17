use std::{slice::Iter, vec::IntoIter};

pub mod gates;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Circuit<T> {
	pub gates: Vec<T>,
}

impl<T> Default for Circuit<T> {
	fn default() -> Self {
		Self {
			gates: Vec::default(),
		}
	}
}

impl<T> Circuit<T> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn len(&self) -> usize {
		self.gates.len()
	}

	pub fn is_empty(&self) -> bool {
		self.gates.is_empty()
	}

	pub fn push<G: Into<T>>(&mut self, gate: G) {
		self.gates.push(gate.into());
	}

	pub fn iter(&self) -> Iter<'_, T> {
		self.gates.iter()
	}
}

impl<T> IntoIterator for Circuit<T> {
	type IntoIter = IntoIter<T>;
	type Item = T;

	fn into_iter(self) -> Self::IntoIter {
		self.gates.into_iter()
	}
}

impl<T: RandomGate> Circuit<T> {
	pub fn random<R: rand::prelude::Rng>(n_gates: usize, n_qubits: usize, rng: &mut R) -> Self {
		let mut gates = Vec::with_capacity(n_gates);
		while gates.len() < n_gates {
			gates.push(T::random(n_qubits, rng));
		}

		Self { gates }
	}
}

pub trait RandomGate {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self;
}
