use std::num::NonZero;

use openqasm2::{OpaqueFunction, OpaqueFunctionDefinition, OpenQasm2Cx};
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

	pub fn target(&self) -> usize {
		self.target
	}

	pub fn control(&self) -> usize {
		self.control
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
