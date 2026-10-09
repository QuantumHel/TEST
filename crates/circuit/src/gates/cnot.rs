use std::num::NonZero;

use openqasm2::{OpaqueFunction, OpaqueFunctionDefinition, OpenQasm2Cx};
use pauli::{
	Clifford,
	PauliLetter::{I, X, Y, Z},
};
use rand::RngExt;

use crate::{Circuit, RandomGate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CNot {
	control: usize,
	target: usize,
}

impl CNot {
	/// Fails if control == target
	pub fn new(control: usize, target: usize) -> Option<Self> {
		if control != target {
			Some(CNot { control, target })
		} else {
			None
		}
	}

	pub fn target(&self) -> &usize {
		&self.target
	}

	pub fn control(&self) -> &usize {
		&self.control
	}

	pub fn reverse(&self) -> Self {
		CNot {
			control: self.target,
			target: self.control,
		}
	}
}

impl RandomGate for CNot {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		let control = rng.random_range(..n_qubits);
		let mut target = rng.random_range(..(n_qubits - 1));
		if target >= control {
			target += 1;
		}

		Self { control, target }
	}
}

pub struct CNotOpaque;

impl<T: From<CNot>> OpaqueFunction<Circuit<T>> for CNotOpaque {
	fn definition(&self) -> OpaqueFunctionDefinition {
		OpaqueFunctionDefinition {
			name: String::from("cx"),
			n_params: 0,
			n_qargs: NonZero::new(2).unwrap(),
		}
	}

	fn insert_gate(&self, qargs: Vec<usize>, ir: &mut Circuit<T>) {
		let cnot = CNot::new(qargs[0], qargs[1]).unwrap();
		ir.push(cnot);
	}

	fn type_check(&self, qargs: Vec<usize>) -> Result<(), &'static str> {
		if qargs[0] == qargs[1] {
			Err("cx needs unique control and target")
		} else {
			Ok(())
		}
	}
}

impl<T: From<CNot>> OpenQasm2Cx<Circuit<T>> for CNotOpaque {
	fn insert_cnot(&self, control: usize, target: usize, ir: &mut Circuit<T>) {
		let cnot = CNot::new(control, target).unwrap();
		ir.push(cnot);
	}
}

impl Clifford for CNot {
	fn conjugate(&self, pauli_string: &pauli::PauliString) -> (bool, pauli::PauliString) {
		let (sign, control, target) = match (
			pauli_string.get(self.control),
			pauli_string.get(self.target),
		) {
			(Z, Z) => (false, I, Z),
			(I, Z) => (false, Z, Z),
			(Z, Y) => (false, I, Y),
			(I, Y) => (false, Z, Y),
			(Y, I) => (false, Y, X),
			(Y, X) => (false, Y, I),
			(X, I) => (false, X, X),
			(X, X) => (false, X, I),
			(X, Z) => (true, Y, Y),
			(Y, Y) => (true, X, Z),
			(X, Y) => (true, Y, Z),
			(Y, Z) => (true, X, Y),
			(c, t) => (false, c, t),
		};

		let mut new = pauli_string.clone();
		new.set(self.control, control);
		new.set(self.target, target);
		(sign, new)
	}

	fn interacting_qubits(&self) -> impl Iterator<Item = &usize> {
		[&self.target, &self.control].into_iter()
	}
}
