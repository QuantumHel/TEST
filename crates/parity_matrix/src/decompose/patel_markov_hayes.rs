use circuit::{Circuit, gates::CNot};
use core::Compiler;

use crate::ParityMatrix;
use std::{num::NonZero, ops::Range};

enum MSource {
	Constant(NonZero<usize>),
	Equation(Box<dyn Fn(usize) -> NonZero<usize>>),
}

/// # Patel Markov Hayes
///
/// This is an implementation of the algorithm in https://arxiv.org/abs/quant-ph/0302002
pub struct PatelMarkovHayes {
	/// During the algorithm the matrix will be split into parts with no more
	/// than m columns.
	m: MSource,
}

impl Default for PatelMarkovHayes {
	fn default() -> Self {
		Self::new_with_equation(|n| {
			NonZero::new(((n as f64).log2() / 2.).round().max(1.) as usize).unwrap()
		})
	}
}

impl Compiler<ParityMatrix, Circuit<CNot>> for PatelMarkovHayes {
	fn compile(&self, mut input: ParityMatrix, _: &()) -> Circuit<CNot> {
		let mut circuit_l = self.lwr_cnot_synth(&mut input);
		let mut input = input.transpose();
		let circuit_u = self.lwr_cnot_synth(&mut input);
		assert!(input.is_identity());

		let mut circuit = circuit_u
			.into_iter()
			.map(|cnot| cnot.reverse())
			.rev()
			.collect::<Vec<CNot>>();
		circuit.append(&mut circuit_l);

		Circuit { gates: circuit }
	}
}

impl PatelMarkovHayes {
	/// During the algorithm the matrix will be split into parts with no more
	/// than m columns.
	pub fn new(m: NonZero<usize>) -> Self {
		Self {
			m: MSource::Constant(m),
		}
	}

	/// During the algorithm the matrix will be split into parts with no more
	/// than m columns.
	pub fn new_with_equation<F: Fn(usize) -> NonZero<usize> + 'static>(m_equation: F) -> Self {
		Self {
			m: MSource::Equation(Box::new(m_equation)),
		}
	}

	fn lwr_cnot_synth(&self, matrix: &mut ParityMatrix) -> Vec<CNot> {
		let n = matrix.size();
		let m = match &self.m {
			MSource::Constant(m) => *m,
			MSource::Equation(f) => f(n),
		}
		.get();
		let mut circuit = Vec::new();

		// Iterate over column sections
		for sec in 1..=n.div_ceil(m) {
			// remove duplicate sub-rows in sec
			// We will likely not have so large values that we fail the m as u32
			// conversion
			let mut patt = vec![-1; 2_usize.pow(m as u32)];
			for row_ind in ((sec - 1) * m)..n {
				let sub_row_patt =
					matrix.get_section::<Range<usize>>(row_ind, ((sec - 1) * m)..(sec * m));
				let sub_row_patt: usize = sub_row_patt.try_into().unwrap();

				// The pseudocode in the paper forgot this, but it is needed.
				if sub_row_patt == 0 {
					continue;
				}

				// if first copy of pattern save otherwise remove
				if patt[sub_row_patt] == -1 {
					patt[sub_row_patt] = row_ind as i32;
				} else {
					circuit.push(matrix.add_row(patt[sub_row_patt] as usize, row_ind)); // HERE
				}
			}

			// Use gaussian elimination for remaining entries in column section
			// The commented out "-1" is in the paper, but I think that it is a mistake.
			for col_ind in ((sec - 1) * m)..(sec * m/* -1 */) {
				// check for 1 on diagonal
				let mut diag_one = matrix.get(col_ind, col_ind);

				//removes ones in rows below col_ind
				for row_ind in (col_ind + 1)..n {
					if matrix.get(row_ind, col_ind) {
						if !diag_one {
							circuit.push(matrix.add_row(row_ind, col_ind));
							diag_one = true;
						}
						circuit.push(matrix.add_row(col_ind, row_ind)); // HERE
					}
				}
			}
		}

		circuit.into_iter().rev().collect()
	}
}

#[cfg(test)]
mod test {
	use circuit::{Circuit, RandomGate, gates::CNot};
	use core::Compiler;
	use rand::prelude::*;
	use rand_chacha::ChaCha8Rng;
	use std::num::NonZero;

	use super::PatelMarkovHayes;
	use crate::ParityMatrix;

	/// This is the example given in the original paper.
	///
	/// The idea here is not to only decompose the tableau, but also to get
	/// the exact same decomposition.
	#[test]
	fn paper_example() {
		let gates = vec![
			CNot::new(4, 3).unwrap(), // Leftmost
			CNot::new(1, 0).unwrap(),
			CNot::new(3, 1).unwrap(),
			CNot::new(5, 2).unwrap(),
			CNot::new(4, 2).unwrap(),
			CNot::new(4, 3).unwrap(),
			CNot::new(5, 4).unwrap(),
			CNot::new(2, 3).unwrap(), // Start of dashed box
			CNot::new(3, 2).unwrap(),
			CNot::new(3, 5).unwrap(),
			CNot::new(2, 4).unwrap(),
			CNot::new(1, 2).unwrap(),
			CNot::new(0, 1).unwrap(),
			CNot::new(0, 4).unwrap(),
			CNot::new(0, 3).unwrap(),
		];
		let answer = Circuit { gates };

		let mut partiy_matrix = ParityMatrix::default();
		for cnot in answer.iter() {
			partiy_matrix.insert_cnot(*cnot);
		}

		let cnot_synth = PatelMarkovHayes::new(NonZero::new(2).unwrap());
		let output = cnot_synth.compile(partiy_matrix.clone(), &());
		assert_eq!(answer, output);
	}

	#[test]
	fn random_test() {
		const TEST_COUNT: usize = 100;
		const QUBIT_COUNT: usize = 100;
		const CNOT_COUNT: usize = QUBIT_COUNT * 100;

		let m = ((QUBIT_COUNT as f64).log2() / 2.).round() as usize;
		let cnot_synth = PatelMarkovHayes::new(NonZero::new(m.max(1)).unwrap());
		let mut rng = ChaCha8Rng::seed_from_u64(2);

		for _ in 0..TEST_COUNT {
			let cnots: Vec<_> = (0..CNOT_COUNT)
				.map(|_| CNot::random(QUBIT_COUNT, &mut rng))
				.collect();

			let mut parity_matrix = ParityMatrix::default();
			for cnot in cnots {
				parity_matrix.insert_cnot(cnot);
			}

			let out = cnot_synth.compile(parity_matrix.clone(), &());

			for cnot in out.iter().rev() {
				parity_matrix.insert_cnot(*cnot);
			}

			assert!(parity_matrix.is_identity());
		}
	}

	#[test]
	fn same_circuit_different_basis_test() {
		const TEST_COUNT: usize = 100;
		const QUBIT_COUNT: usize = 100;
		const CNOT_COUNT: usize = QUBIT_COUNT * 100;

		let m = ((QUBIT_COUNT as f64).log2() / 2.).round() as usize;
		let cnot_synth = PatelMarkovHayes::new(NonZero::new(m.max(1)).unwrap());
		let mut rng = ChaCha8Rng::seed_from_u64(2);

		for _ in 0..TEST_COUNT {
			let cnots: Vec<_> = (0..CNOT_COUNT)
				.map(|_| CNot::random(QUBIT_COUNT, &mut rng))
				.collect();

			let (standard_out, mut standard_matrix) = {
				let mut parity_matrix = ParityMatrix::default();
				for cnot in cnots.iter() {
					parity_matrix.insert_cnot(*cnot);
				}

				// This will crash if it fails
				let out = cnot_synth.compile(parity_matrix.clone(), &());
				(out, parity_matrix)
			};

			let (hadamard_out, mut hadamard_matrix) = {
				let mut parity_matrix = ParityMatrix::hadamard_basis();
				for cnot in cnots.iter() {
					parity_matrix.insert_cnot(*cnot);
				}

				// This will crash if it fails
				let out = cnot_synth.compile(parity_matrix.clone(), &());
				(out, parity_matrix)
			};

			for cnot in standard_out.into_iter().rev() {
				hadamard_matrix.insert_cnot(cnot);
			}
			assert!(hadamard_matrix.is_identity());

			for cnot in hadamard_out.into_iter().rev() {
				standard_matrix.insert_cnot(cnot);
			}
			assert!(standard_matrix.is_identity());
		}
	}
}
