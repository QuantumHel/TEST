use test_core::Compiler;

use crate::{CNot, ParityMatrix};
use std::{num::NonZero, ops::Range};

/// # Patel Markov Hayes
///
/// This is an implementation of the algorithm in https://arxiv.org/abs/quant-ph/0302002
pub struct PatelMarkovHayes {
	/// During the algorithm the matrix will be split into parts with no more
	/// than m columns.
	pub m: u32,
}

impl Compiler for PatelMarkovHayes {
	type Input = ParityMatrix;
	type Output = Vec<CNot>;

	fn compile(&self, mut input: Self::Input) -> Self::Output {
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

		circuit
	}
}

impl PatelMarkovHayes {
	pub fn new(m: NonZero<u32>) -> Self {
		Self { m: m.into() }
	}

	fn lwr_cnot_synth(&self, matrix: &mut ParityMatrix) -> Vec<CNot> {
		let n = matrix.size();
		let mut circuit = Vec::new();

		// Iterate over column sections
		for sec in 1..=n.div_ceil(self.m as usize) {
			// remove duplicate sub-rows in sec
			let mut patt = vec![-1; 2_usize.pow(self.m)];
			for row_ind in ((sec - 1) * self.m as usize)..n {
				let sub_row_patt = matrix.get_section::<Range<usize>>(
					row_ind,
					((sec - 1) * self.m as usize)..(sec * self.m as usize),
				);
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
			for col_ind in ((sec - 1) * self.m as usize)..(sec * self.m as usize/* -1 */) {
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

		return circuit.into_iter().rev().collect();
	}
}

#[cfg(test)]
mod test {
	use rand::prelude::*;
	use rand_chacha::ChaCha8Rng;
	use std::num::NonZeroU32;
	use test_core::Compiler;

	use crate::{CNot, ParityMatrix, PatelMarkovHayes};

	/// This is the example given in the original paper.
	///
	/// The idea here is not to only decompose the tableau, but also to get
	/// the exact same decomposition.
	#[test]
	fn paper_example() {
		let answer = vec![
			CNot::new(4, 3), // Leftmost
			CNot::new(1, 0),
			CNot::new(3, 1),
			CNot::new(5, 2),
			CNot::new(4, 2),
			CNot::new(4, 3),
			CNot::new(5, 4),
			CNot::new(2, 3), // Start of dashed box
			CNot::new(3, 2),
			CNot::new(3, 5),
			CNot::new(2, 4),
			CNot::new(1, 2),
			CNot::new(0, 1),
			CNot::new(0, 4),
			CNot::new(0, 3),
		];

		let mut partiy_matrix = ParityMatrix::default();
		for cnot in answer.iter() {
			partiy_matrix.add_row(cnot.control, cnot.target);
		}

		let cnot_synth = PatelMarkovHayes::new(NonZeroU32::new(2).unwrap());
		let output = cnot_synth.compile(partiy_matrix);
		assert_eq!(answer, output);
	}

	#[test]
	fn random_test() {
		const TEST_COUNT: usize = 100;
		const QUBIT_COUNT: usize = 100;
		const CNOT_COUNT: usize = QUBIT_COUNT * 10;

		let m = ((QUBIT_COUNT as f64).log2() / 2.).round() as u32;
		let cnot_synth = PatelMarkovHayes::new(NonZeroU32::new(m.max(1)).unwrap());
		let mut rng = ChaCha8Rng::seed_from_u64(2);

		for _ in 0..TEST_COUNT {
			let cnots: Vec<_> = (0..CNOT_COUNT)
				.map(|_| CNot::random(QUBIT_COUNT, &mut rng))
				.collect();

			let mut partiy_matrix = ParityMatrix::default();
			for cnot in cnots {
				partiy_matrix.add_row(cnot.control, cnot.target);
			}

			// This will crash if it fails
			cnot_synth.compile(partiy_matrix);
		}
	}
}
