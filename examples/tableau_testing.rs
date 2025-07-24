use std::{
	sync::{Arc, Mutex},
	thread,
};

use rand::{Rng, seq::IteratorRandom};
use test_transpiler::{
	clifford_tableau::CliffordTableau,
	misc::NonZeroEvenUsize,
	pauli::{PauliLetter, PauliString},
};

fn random_string<const N: usize, R: Rng>(rng: &mut R) -> PauliString<N> {
	let mut string = PauliString::default();
	for qubit in 0..N {
		let pauli = match (0_usize..3_usize).choose(rng).unwrap() {
			0 => PauliLetter::X,
			1 => PauliLetter::Y,
			_ => PauliLetter::Z,
		};
		string.set(qubit, pauli);
	}

	string
}

const QUBITS: usize = 6;
const GATE_SIZE: usize = 4;
const N_STRIGNS: usize = 100;

fn main() {
	let mut tableau: CliffordTableau<QUBITS> = CliffordTableau::id();
	let mut rng = rand::rng();
	for _ in 0..N_STRIGNS {
		tableau.merge_pi_over_4_pauli(false, &random_string(&mut rng));
	}

	println!("Unsolved:");
	tableau.info_print(QUBITS);
	println!();
	println!("Solved (at least partially):");
	let decomposition = tableau.decompose(NonZeroEvenUsize::new(GATE_SIZE).unwrap());
	println!();
	println!("Decomposition len: {}", decomposition.len());
}
