mod cnot;
mod rz;
use std::num::NonZero;

pub use cnot::{CNot, CNotOpaque};
use openqasm2::{OpaqueFunction, OpaqueFunctionDefinition};
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
