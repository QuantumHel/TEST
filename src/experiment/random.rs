use rand::prelude::*;

use crate::pauli::{PauliAngle, PauliExp, PauliLetter, PauliString};

pub fn random_exp<R: Rng>(max_exp_size: usize, rng: &mut R) -> PauliExp<PauliAngle> {
	let n_letters = (1_usize..=max_exp_size).choose(rng);
	let mut selection: Vec<usize> = (0..max_exp_size).collect();
	selection.shuffle(rng);
	let mut string = PauliString::default();
	for qubit in selection.into_iter().take(n_letters.unwrap()) {
		let pauli = match (0_usize..3_usize).choose(rng).unwrap() {
			0 => PauliLetter::X,
			1 => PauliLetter::Y,
			_ => PauliLetter::Z,
		};
		string.set(qubit, pauli);
	}

	PauliExp {
		string,
		angle: PauliAngle::MultipleOfPi(rng.random()),
	}
}
