pub mod choose;
pub mod decompose;
mod gray_star_synth;
mod parity_matrix;
mod t_par;
mod xor_span;

use core::Compiler;

use circuit::{Circuit, gates::CNot};
pub use parity_matrix::ParityMatrix;

// These need to be moved to own crate
pub use gray_star_synth::{GrayStarSynth, GrayStarTerminationCriteria};
pub use t_par::{CNotRzXYH, TPar};

pub use self::parity_matrix::Basis;

#[derive(Default)]
pub struct CollectParityMatrix {
	pub basis: Basis,
}

impl<D> Compiler<Circuit<CNot>, ParityMatrix, D> for CollectParityMatrix {
	fn compile(&self, input: Circuit<CNot>, _: &D) -> ParityMatrix {
		let mut parity_matrix = match self.basis {
			Basis::Standard => ParityMatrix::standard_basis(),
			Basis::Hadamard => ParityMatrix::hadamard_basis(),
		};

		for cnot in input {
			parity_matrix.insert_cnot(cnot);
		}

		parity_matrix
	}
}
