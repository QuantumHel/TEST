use std::{
	sync::{Arc, Mutex},
	thread,
};

use rand::{Rng, seq::IteratorRandom};
use test_transpiler::{
	clifford_tableau::CliffordTableau,
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

const QUBITS: usize = 4;
const N_OUTER_STRINGS: usize = 2;
const N_INNER_STRINGS: usize = 2;

const N_THREADS: usize = 8;

fn check(tableau: &CliffordTableau<QUBITS>) -> bool {
	!(tableau.get_x_row(0).unwrap().0 == PauliString::x(0)
		&& tableau.get_z_row(0).unwrap().0 == PauliString::z(0)
		&& tableau.get_x_row(1).unwrap().0 == PauliString::x(1)
		&& tableau.get_z_row(1).unwrap().0.get(1) == PauliLetter::I
		&& tableau.get_z_row(1).unwrap().0.len() == 2)
}

fn main() {
	let mut handles = Vec::new();
	let lock = Arc::new(Mutex::new(()));
	for _ in 0..N_THREADS {
		let lock = lock.clone();
		handles.push(thread::spawn(move || {
			loop {
				let mut rng = rand::rng();
				let mut tableau = CliffordTableau::<QUBITS>::default();

				let mut strings = Vec::new();
				for _ in 0..(N_OUTER_STRINGS + N_INNER_STRINGS) {
					strings.push(random_string(&mut rng));
				}

				for i in (0..N_OUTER_STRINGS).rev() {
					strings.push(strings[i].clone());
				}

				for (i, string) in strings.iter().enumerate() {
					tableau.merge_pi_over_4_pauli(false, string);
					if !check(&tableau) {
						let lock = lock.lock();
						println!("Check failed on at round {i}");
						for (i, s) in strings.iter().enumerate() {
							println!("{}: {}", i, s.as_string());
						}
						tableau.info_print(QUBITS);
						println!();
						drop(lock);
						return;
					}
				}
			}
		}));
	}

	for handle in handles.into_iter() {
		let _ = handle.join();
	}
}
