mod find_and_replace;
pub mod gates;

use core::TargetQubits;
use std::{slice::Iter, vec::IntoIter};

use bits::Bits;
pub use find_and_replace::{FindAndReplace, FindAndReplaceError};

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

	pub fn fileter_len<F: Fn(&T) -> bool>(&self, f: F) -> usize {
		let mut len = 0;
		for gate in self.gates.iter() {
			if f(gate) {
				len += 1;
			}
		}

		len
	}

	pub fn is_empty(&self) -> bool {
		self.gates.is_empty()
	}

	pub fn push<G: Into<T>>(&mut self, gate: G) {
		self.gates.push(gate.into());
	}

	pub fn append<G: Into<T>>(&mut self, gates: &mut Vec<G>) {
		self.gates.extend(gates.drain(..).map(Into::into));
	}

	pub fn iter(&self) -> Iter<'_, T> {
		self.gates.iter()
	}

	pub fn reverse(&mut self) {
		self.gates.reverse();
	}
}

impl<T: TargetQubits> Circuit<T> {
	pub fn depth(&self) -> usize {
		self.fileter_len(|_| true)
	}

	/// Takes circuit depth while ignoring all gates for which the given filter
	/// returns false.
	pub fn filtered_depth<F: Fn(&T) -> bool>(&self, filter: F) -> usize {
		let mut layers: Vec<Bits> = Vec::new();

		for gate in self.gates.iter() {
			if !filter(gate) {
				continue;
			}

			let mut stop: Option<usize> = None;
			'layer: for (i, layer) in layers.iter().enumerate().rev() {
				for &qubit in gate.target_qubits() {
					if layer.get(qubit) {
						stop = Some(i);
						break 'layer;
					}
				}
			}

			let layer: usize = match stop {
				Some(i) => {
					if (i + 1) == layers.len() {
						layers.push(Bits::new());
					}
					i + 1
				}
				None => {
					if layers.is_empty() {
						layers.push(Bits::new());
					}
					0
				}
			};

			for &qubit in gate.target_qubits() {
				layers[layer].set(qubit, true);
			}
		}

		layers.len()
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
