use std::collections::btree_map::Entry;

use crate::{
	misc::NonZeroEvenUsize,
	pauli::{PauliAngle, PauliExp, PauliMatrix, PauliString},
};

fn get_all<F: Fn(&PauliExp) -> bool>(exponentials: &mut Vec<PauliExp>, f: F) -> Vec<PauliExp> {
	let mut indexes: Vec<usize> = Vec::new();

	let mut i: usize = 0;
	for exp in exponentials.iter() {
		if f(exp) {
			indexes.push(i);
		} else {
			i += 1;
		}
	}

	let mut res = Vec::new();
	for i in indexes {
		res.push(exponentials.remove(i));
	}

	res
}

pub fn synthesize(
	mut exponentials: Vec<PauliExp>,
	gate_size: NonZeroEvenUsize,
) -> (Vec<PauliExp>, Vec<PauliExp>) {
	// TODO: merge equal pauli strings
	let n = gate_size.as_value();
	let mut circuit: Vec<PauliExp> = Vec::new();
	let mut clifford_tableau: Vec<PauliExp> = Vec::new();

	// move single (an no) qubit gates to circuit
	let single_qubit = get_all(&mut exponentials, |p| p.len() <= 1);
	for gate in single_qubit.into_iter() {
		assert!(gate.len() == 1);
		circuit.push(gate);
	}

	// This is the main synthesize loop.
	// - Select an exponential to pus.
	// - push trough exponentials
	// - add corresponding exponentials to circuit and clifford_tableau
	// - mode single qubit exponentials to the circuit.
	while !exponentials.is_empty() {
		// Select the pauli string to push
		let push_str = if let Some(exp) = exponentials.iter().find(|p| p.len() == n) {
			// Turn n long exponential to 1 long.
			let mut push_str = exp.string.clone();
			let (i, m) = push_str.letters.iter().next().unwrap();
			push_str.letters.insert(*i, m.change());

			push_str
		} else if let Some(exp) = exponentials
			.iter()
			.find(|p| (p.string.len() % 2 == 1) && (p.string.len() < (2 * n)))
		{
			// Turn uneven long exponential that is shorter than 2n into a n long one;
			if exp.len() < n {
				// Add some if not enough
				let mut push_str = exp.string.clone();
				// Because uneven this makes sure that we anticommute and that all letter places
				// stay.
				for (_, m) in push_str.letters.iter_mut() {
					*m = m.change();
				}

				// These will be added on push
				for i in 0..(n as u32) {
					if let Entry::Vacant(entry) = push_str.letters.entry(i) {
						entry.insert(PauliMatrix::X);
						if push_str.len() == n {
							break;
						}
					}
				}

				push_str
			} else {
				// remove some if too many
				let mut push_str = PauliString {
					letters: exp
						.string
						.letters
						.iter()
						.take(n)
						.map(|(i, m)| (*i, *m))
						.collect(),
				};

				// These are the ones we keep
				for (_, m) in push_str.letters.iter_mut().take(2 * n - exp.len()) {
					*m = m.change();
				}

				push_str
			}
		} else if let Some(exp) = exponentials
			.iter()
			.find(|p| (p.string.len() % 2 == 0) && (p.string.len() < (3 * n - 1)))
		{
			// This makes the selected exp compatible with the case above. This means that we need
			// two steps to get this exp into a single qubit exp.
			if exp.len() < n {
				// By adding n-1 qubit letters to the exp we can make it compatible with if let
				// above the if let that we took.

				let mut push_str = PauliString {
					letters: exp
						.string
						.letters
						.iter()
						.take(1)
						.map(|(i, m)| (*i, m.change()))
						.collect(),
				};

				// These will be added on push
				for i in 0..(2 * n as u32) {
					if !exp.string.letters.contains_key(&i) {
						push_str.letters.insert(i, PauliMatrix::X);
						if push_str.len() == n {
							break;
						}
					}
				}

				push_str
			} else {
				// By removing n-1 qubit letters from the exp we can make it compatible with if let
				// above the if let that we took.
				let mut push_str = PauliString {
					letters: exp
						.string
						.letters
						.iter()
						.take(n)
						.map(|(i, m)| (*i, *m))
						.collect(),
				};

				let m = push_str.letters.iter_mut().next().unwrap().1;
				*m = m.change();

				push_str
			}
		} else {
			// Else remove as many qubits as possible from the first exponential so that we can
			// access some of the cases above. This can take multiple rounds.
			let exp = exponentials.first().unwrap();

			let mut push_str = PauliString {
				letters: exp
					.string
					.letters
					.iter()
					.take(n)
					.map(|(i, m)| (*i, *m))
					.collect(),
			};

			let m = push_str.letters.iter_mut().next().unwrap().1;
			*m = m.change();

			push_str
		};

		assert!(push_str.len() == n);

		for exp in exponentials.iter_mut() {
			exp.push_neq_pi_over_4(&push_str);
		}
		circuit.push(PauliExp {
			string: push_str.clone(),
			angle: PauliAngle::PiOver4,
		});
		clifford_tableau.push(PauliExp {
			string: push_str,
			angle: PauliAngle::NeqPiOver4,
		});
		// move all created single qubit gates to circuit
		let single_qubit = get_all(&mut exponentials, |p| p.len() == 1);
		for gate in single_qubit.into_iter() {
			assert!(gate.len() == 1);
			circuit.push(gate);
		}
	}

	(circuit, clifford_tableau)
}

#[cfg(test)]
mod tests {
	use super::*;
	use nanorand::{Rng, WyRand};

	fn random_exp(qubits: usize, rng: &mut WyRand) -> PauliExp {
		let n_letters = rng.generate_range(1_usize..=qubits);
		let mut selection: Vec<u32> = (0..(qubits as u32)).collect();
		rng.shuffle(&mut selection);
		let mut string = PauliString::default();
		for qubit in selection.into_iter().take(n_letters) {
			let pauli = match rng.generate_range(0_usize..3_usize) {
				0 => PauliMatrix::X,
				1 => PauliMatrix::Y,
				_ => PauliMatrix::Z,
			};
			string.letters.insert(qubit, pauli);
		}

		PauliExp {
			string,
			angle: PauliAngle::Free(rng.generate()),
		}
	}

	#[test]
	fn synthesize_result_has_suitable_operators() {
		for _ in 0..1 {
			let mut rng = WyRand::new();
			let input: Vec<PauliExp> = (0..30).map(move |_| random_exp(30, &mut rng)).collect();

			let (circuit, clifford) = synthesize(input, NonZeroEvenUsize::new(4).unwrap());

			for exp in circuit {
				assert!(exp.len() == 1 || exp.len() == 4);
			}

			for exp in clifford {
				assert!(exp.angle.is_clifford())
			}
		}
	}
}
