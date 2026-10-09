mod cnot;
mod rz;
use std::num::NonZero;

pub use cnot::{CNot, CNotOpaque};
use openqasm2::{OpaqueFunction, OpaqueFunctionDefinition};
use pauli::{Clifford, PauliLetter};
use rand::RngExt;
pub use rz::Rz;

use crate::{Circuit, RandomGate};

macro_rules! trivial_single_qubit_gate {
	($name:ident, $opaque:ident, $text:literal) => {
		#[derive(Debug, Clone, Copy, PartialEq)]
		pub struct $name {
			pub target: usize,
		}

		impl RandomGate for $name {
			fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
				Self {
					target: rng.random_range(..n_qubits),
				}
			}
		}

		pub struct $opaque;

		impl<T: From<$name>> OpaqueFunction<Circuit<T>> for $opaque {
			fn definition(&self) -> OpaqueFunctionDefinition {
				OpaqueFunctionDefinition {
					name: String::from($text),
					n_params: 0,
					n_qargs: NonZero::new(1).unwrap(),
				}
			}

			fn insert_gate(&self, qargs: Vec<usize>, ir: &mut Circuit<T>) {
				let gate = $name { target: qargs[0] };
				ir.push(gate);
			}

			fn type_check(&self, _: Vec<usize>) -> Result<(), &'static str> {
				Ok(())
			}
		}
	};
}

trivial_single_qubit_gate!(Z, ZOpaque, "z");
trivial_single_qubit_gate!(X, XOpaque, "x");
trivial_single_qubit_gate!(Y, YOpaque, "y");
trivial_single_qubit_gate!(H, HOpaque, "h");
trivial_single_qubit_gate!(S, SOpaque, "s");
trivial_single_qubit_gate!(V, VOpaque, "v");

impl Clifford for X {
	fn conjugate(&self, pauli_string: &pauli::PauliString) -> (bool, pauli::PauliString) {
		(
			pauli_string.get(self.target) != PauliLetter::X
				&& pauli_string.get(self.target) != PauliLetter::I,
			pauli_string.clone(),
		)
	}

	fn interacting_qubits(&self) -> impl Iterator<Item = &usize> {
		std::iter::once(&self.target)
	}
}

impl Clifford for Z {
	fn conjugate(&self, pauli_string: &pauli::PauliString) -> (bool, pauli::PauliString) {
		(
			pauli_string.get(self.target) != PauliLetter::Z
				&& pauli_string.get(self.target) != PauliLetter::I,
			pauli_string.clone(),
		)
	}

	fn interacting_qubits(&self) -> impl Iterator<Item = &usize> {
		std::iter::once(&self.target)
	}
}

impl Clifford for Y {
	fn conjugate(&self, pauli_string: &pauli::PauliString) -> (bool, pauli::PauliString) {
		(
			pauli_string.get(self.target) != PauliLetter::Y
				&& pauli_string.get(self.target) != PauliLetter::I,
			pauli_string.clone(),
		)
	}

	fn interacting_qubits(&self) -> impl Iterator<Item = &usize> {
		std::iter::once(&self.target)
	}
}

impl Clifford for H {
	fn conjugate(&self, pauli_string: &pauli::PauliString) -> (bool, pauli::PauliString) {
		let (sign, letter) = match pauli_string.get(self.target) {
			PauliLetter::X => (false, PauliLetter::Z),
			PauliLetter::Z => (false, PauliLetter::X),
			PauliLetter::Y => (true, PauliLetter::Y),
			PauliLetter::I => (false, PauliLetter::I),
		};
		let mut new = pauli_string.clone();
		new.set(self.target, letter);
		(sign, new)
	}

	fn interacting_qubits(&self) -> impl Iterator<Item = &usize> {
		std::iter::once(&self.target)
	}
}

impl Clifford for S {
	fn conjugate(&self, pauli_string: &pauli::PauliString) -> (bool, pauli::PauliString) {
		let (sign, letter) = match pauli_string.get(self.target) {
			PauliLetter::Z => (false, PauliLetter::Z),
			PauliLetter::X => (false, PauliLetter::Y),
			PauliLetter::Y => (true, PauliLetter::X),
			PauliLetter::I => (false, PauliLetter::I),
		};
		let mut new = pauli_string.clone();
		new.set(self.target, letter);
		(sign, new)
	}

	fn interacting_qubits(&self) -> impl Iterator<Item = &usize> {
		std::iter::once(&self.target)
	}
}

impl Clifford for V {
	fn conjugate(&self, pauli_string: &pauli::PauliString) -> (bool, pauli::PauliString) {
		let (sign, letter) = match pauli_string.get(self.target) {
			PauliLetter::Z => (true, PauliLetter::Y),
			PauliLetter::X => (false, PauliLetter::X),
			PauliLetter::Y => (false, PauliLetter::Z),
			PauliLetter::I => (false, PauliLetter::I),
		};
		let mut new = pauli_string.clone();
		new.set(self.target, letter);
		(sign, new)
	}

	fn interacting_qubits(&self) -> impl Iterator<Item = &usize> {
		std::iter::once(&self.target)
	}
}
