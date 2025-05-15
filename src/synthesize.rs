use std::{collections::btree_map::Entry, usize};

use crate::pauli::{PauliAngle, PauliExp, PauliMatrix, PauliString};

pub fn synthesize(
	mut exponentials: Vec<PauliExp>,
	gate_size: usize,
) -> (Vec<PauliExp>, Vec<PauliExp>) {
	let mut circuit: Vec<PauliExp> = Vec::new();
	let mut clifford_tableau: Vec<PauliExp> = Vec::new();

	while !exponentials.is_empty() {
		exponentials.sort_by(|a, b| a.len().cmp(&b.len()).reverse());

		while exponentials.last().is_some_and(|v| v.len() == 1) {
			circuit.push(exponentials.pop().unwrap())
		}

		// TODO: add gate_size-qubit gates to circuit?

		if exponentials.last().is_none() {
			break;
		}

		if exponentials.last().unwrap().len() <= gate_size {
			let removable = vec![exponentials.pop().unwrap()];
			// TODO: Make so that we remove all possible together

			// TODO:: Need to also change the way we select the string based on selecting many
			let mut push_string = PauliString::default();
			let mut iter = removable.first().unwrap().string.letters.iter();
			match iter.next().unwrap() {
				(i, PauliMatrix::X) => {
					push_string.letters.insert(*i, PauliMatrix::Y);
				}
				(i, PauliMatrix::Y) => {
					push_string.letters.insert(*i, PauliMatrix::Z);
				}
				(i, PauliMatrix::Z) => {
					push_string.letters.insert(*i, PauliMatrix::X);
				}
			}
			for (i, matrix) in iter {
				push_string.letters.insert(*i, *matrix);
			}
			let mut qubits_missing = gate_size - push_string.len();
			for i in 0..(gate_size as u32) {
				if qubits_missing < 1 {
					continue;
				}

				if let Entry::Vacant(entry) = push_string.letters.entry(i) {
					entry.insert(PauliMatrix::X);
					qubits_missing -= 1;
				}
			}

			circuit.push(PauliExp {
				string: push_string.clone(),
				angle: PauliAngle::PiOver4,
			});
			clifford_tableau.push(PauliExp {
				string: push_string.clone(),
				angle: PauliAngle::NeqPiOver4,
			});
			for mut exp in removable.into_iter() {
				exp.push_neq_pi_over_4(&push_string);
				assert_eq!(exp.len(), 1);
				circuit.push(exp);
			}
			for exp in exponentials.iter_mut() {
				exp.push_neq_pi_over_4(&push_string);
			}
		} else {
			let push_string = PauliString {
				letters: exponentials
					.first()
					.unwrap()
					.string
					.letters
					.iter()
					.take(gate_size)
					.map(|(i, m)| (*i, *m))
					.collect(),
			};

			circuit.push(PauliExp {
				string: push_string.clone(),
				angle: PauliAngle::PiOver4,
			});
			clifford_tableau.push(PauliExp {
				string: push_string.clone(),
				angle: PauliAngle::NeqPiOver4,
			});
			for exp in exponentials.iter_mut() {
				exp.push_neq_pi_over_4(&push_string);
			}
		}
	}

	(circuit, clifford_tableau)
}
