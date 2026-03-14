use std::num::NonZeroU32;

use cnot_parity_matrix::{CNot, ParityMatrix, PatelMarkovHayes};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use test_core::Compiler;

const TEST_COUNT: usize = 100;
const QUBIT_COUNT: usize = 100;

fn main() {
	let m = ((QUBIT_COUNT as f64).log2() / 2.).round() as u32;

	for cnot_count in 1..40 {
		let cnot_count = cnot_count * 10;
		let mut bonus = 0;
		let mut bonus_percentage = 0.;

		let cnot_synth = PatelMarkovHayes::new(NonZeroU32::new(m.max(1)).unwrap());
		let mut rng = ChaCha8Rng::seed_from_u64(2);

		for _ in 0..TEST_COUNT {
			let cnots: Vec<_> = (0..cnot_count)
				.map(|_| CNot::random(QUBIT_COUNT, &mut rng))
				.collect();

			let standard = {
				let mut parity_matrix = ParityMatrix::default();
				for cnot in cnots.iter() {
					parity_matrix.insert_cnot(*cnot);
				}

				cnot_synth.compile(parity_matrix)
			};

			let hadamard = {
				let mut parity_matrix = ParityMatrix::hadamard_basis();
				for cnot in cnots {
					parity_matrix.insert_cnot(cnot);
				}

				cnot_synth.compile(parity_matrix)
			};

			let diff = standard.len().max(hadamard.len()) - standard.len().min(hadamard.len());
			bonus += diff;
			bonus_percentage += 100. * diff as f64 / standard.len() as f64;
		}

		let average_bonus = bonus as f64 / TEST_COUNT as f64;
		let average_percentage = bonus_percentage / TEST_COUNT as f64;

		println!("CNOT count: {cnot_count}");
		println!("Average diff; {average_bonus} ({average_percentage}%)");
	}
}
