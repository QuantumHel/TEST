use std::{
	sync::{Arc, Mutex},
	thread,
};

use nanorand::{Rng, WyRand};
use test_transpiler::{
	clifford_tableau::CliffordTableau,
	pauli::{PauliLetter, PauliString},
};

fn random_string<const N: usize>(rng: &mut WyRand) -> PauliString<N> {
	let n_letters = rng.generate_range(1_usize..=N);
	let mut selection: Vec<usize> = (0..N).collect();
	rng.shuffle(&mut selection);
	let mut string = PauliString::default();
	for qubit in selection.into_iter().take(n_letters) {
		let pauli = match rng.generate_range(0_usize..3_usize) {
			0 => PauliLetter::X,
			1 => PauliLetter::Y,
			_ => PauliLetter::Z,
		};
		string.set(qubit, pauli);
	}

	string
}

const QUBITS: usize = 4;
const N_THREADS: usize = 32;
const ITERATIONS: usize = 10000000;

fn check(tableau: &CliffordTableau<QUBITS>) -> bool {
	!(tableau.get_x_row(0).unwrap().0.len() == 1
		&& tableau.get_z_row(0).unwrap().0.len() == 1
		&& tableau.get_x_row(1).unwrap().0.len() == 12)
}

fn main() {
	let lock = Arc::new(Mutex::new(()));
	let mut threads = Vec::new();
	for t in 0..N_THREADS {
		let print_lock = lock.clone();
		threads.push(thread::spawn(move || {
			let mut rng = WyRand::new();
			let mut tableau = CliffordTableau::<QUBITS>::default();
			println!("Start check for thread {}: {}", t, check(&tableau));

			let mut percent = 0;
			for i in 1..=ITERATIONS {
				let string = random_string(&mut rng);
				tableau.merge_pi_over_4_pauli(false, &string);
				if !check(&tableau) {
					let lock = print_lock.lock();
					println!("Check failed on thread {}", t);
					tableau.info_print(QUBITS);
					println!();
					drop(lock);
					return;
				}

				let current_percent = i * 100 / ITERATIONS;
				if current_percent > percent {
					percent = current_percent;
					println!("Thread {} is {} percent done", t, percent)
				}
			}
		}));
	}

	for thread in threads {
		thread.join().unwrap();
	}
	println!("All {} checks passed", N_THREADS * ITERATIONS);
}
