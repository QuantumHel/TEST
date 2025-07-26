use std::ops::Index;

use crate::{
	clifford_tableau::CliffordTableau,
	misc::NonZeroEvenUsize,
	pauli::{CliffordPauliAngle, PauliExp, PauliLetter, PauliString},
};

#[derive(Debug, PartialEq, Eq)]
enum QubitProtection {
	X,
	Z,
	None,
}

enum XZLetter {
	X,
	Z,
}

/// Does not edit rows for solved qubits, but is expensive
fn delicate_solver<const N: usize>(
	string: &PauliString<N>,
	target_qubit: usize,
	target_letter: XZLetter,
) -> Vec<PauliString<N>> {
	let mut pushing: Vec<PauliString<N>> = Vec::new();
	// TODO: Handle if correct qubit with len 1(maybe wrong letter)
	// and return

	// use single qubit gate to make sure that target qubit does not have target letter

	// Add outer ones to start
	if string.len() % 2 == 0 {
		// All uninvolved qubits have X in first string and Z on second.
		// The non target qubits always have their letter in the string
		// Target qubit has target letter

		// middle string
		// All non involved qubits have Y
		// All non target involved qubits have original letter
		// Target qubit has the one that is not target and not the one in string
	} else {
		// outer (1)
		// All non involved qubits have Y
		// All non target involved qubits get involved.next
		// target qubit gets target letter

		// inner (1)
		// All non involved qubits get Y
		// All non target involved qubits get the one that is not involved and not involved.next
		// target qubit gets the one in string
	}

	// Not sure if here or after middle row
	if string.get(target_qubit) == PauliLetter::I {
		// This does not protect on target_qubit. On the other hand this is unreachable if we need
		// to protect it.

		todo!()
	}

	// add row
	// All non involved qubits have Y

	// Add the outer ones to end
	if string.len() % 2 == 0 {
		pushing.push(pushing[1].clone());
	}
	pushing.push(pushing[0].clone());

	pushing
}

/// Assumes that there are at least gate size many dirty qubits
fn simple_solver<const N: usize>(
	string: &PauliString<N>,
	gate_size: NonZeroEvenUsize,
	end_qubit: usize,
	target_letter: PauliLetter,
	dirty_qubits: &[usize],
	protection: QubitProtection,
) -> Vec<PauliString<N>> {
	let mut pushing: Vec<PauliString<N>> = Vec::new();
	let n = gate_size.as_value();
	let mut string = string.clone();

	while string.len() > 2 * n - 2 {
		let mut new_string: PauliString<N> = PauliString::id();
		for (i, letter) in string.letters() {
			if i != end_qubit {
				new_string.set(i, letter);
				if new_string.len() == n {
					break;
				}
			}
		}

		// makes it so that we anticommute
		let a = new_string.letters();
		let (index, letter) = a.first().unwrap();
		new_string.set(*index, letter.next());
		string.pi_over_4_sandwitch(false, &new_string);
		pushing.push(new_string);
	}

	// Make sure that the end_qubit has a letter
	if string.get(end_qubit) == PauliLetter::I {
		assert_eq!(protection, QubitProtection::None);
		let mut new_string: PauliString<N> = PauliString::id();
		new_string.set(end_qubit, target_letter.next());

		let mut n_remove = (string.len() + 1).saturating_sub(n);
		if n_remove % 2 == 1 {
			n_remove += 1;
		}
		let n_remove = n_remove.max(n - 2);
		let letters = string.letters();
		let mut iter = letters.iter();
		for _ in 0..n_remove {
			let (index, letter) = iter.next().unwrap();
			new_string.set(*index, *letter);
		}

		while new_string.len() < n {
			let (index, letter) = iter.next().unwrap();
			new_string.set(*index, letter.next());
		}

		string.pi_over_4_sandwitch(false, &new_string);
		pushing.push(new_string);
	}

	// if even and not 4, make it so that we are uneven and under 4
	if string.len() % 2 == 0 && string.len() != n {
		let letters = string.letters();
		let mut iter = letters.iter().filter(|(i, _)| *i != end_qubit);

		// anticummuting letter
		let mut new_string: PauliString<N> = PauliString::id();
		let (index, letter) = iter.next().unwrap();
		new_string.set(*index, letter.next());

		// removals
		while new_string.len() < n {
			let (index, letter) = iter.next().unwrap();
			new_string.set(*index, *letter);
		}

		// we should not be able to come here with less than n-2 qubits
		assert_eq!(new_string.len(), n);

		string.pi_over_4_sandwitch(false, &new_string);
		pushing.push(new_string);
	}

	// if uneven make it so that we get to 4
	if string.len() % 2 == 1 {
		if string.len() < n {
			// add qubits
			let mut new_string = PauliString::id();
			for (i, letter) in string.letters() {
				if i == end_qubit && protection != QubitProtection::None {
					let letter = match protection {
						QubitProtection::X => {
							assert_ne!(string.get(i), PauliLetter::X);
							PauliLetter::X
						}
						// This has to be Z
						_ => {
							assert_ne!(string.get(i), PauliLetter::Z);
							PauliLetter::Z
						}
					};
					new_string.set(i, letter);
				} else {
					new_string.set(i, letter.next());
				}
			}

			// these are added
			for qubit in dirty_qubits {
				if new_string.get(*qubit) == PauliLetter::I {
					new_string.set(*qubit, PauliLetter::X);
					if new_string.len() == n {
						break;
					}
				}
			}

			string.pi_over_4_sandwitch(false, &new_string);
			pushing.push(new_string);
		} else {
			// remove qubits
			let mut letters: Vec<(usize, PauliLetter)> = string
				.letters()
				.into_iter()
				.filter(|(i, _)| *i != end_qubit)
				.take(n)
				.collect();

			// These are the ones we keep (we make them anticommute)
			for (_, l) in letters.iter_mut().take(2 * n - string.len()) {
				*l = l.next();
			}

			let mut new_string = PauliString::<N>::id();
			for (i, l) in letters {
				new_string.set(i, l);
			}

			string.pi_over_4_sandwitch(false, &new_string);
			pushing.push(new_string);
		}
	}

	assert_eq!(string.len(), n);

	// convert to single qubit one
	let mut to_single = string.clone();
	match protection {
		QubitProtection::X => {
			assert_ne!(string.get(end_qubit), PauliLetter::X);
			assert_ne!(target_letter, PauliLetter::X);
			to_single.set(end_qubit, PauliLetter::X);
		}
		QubitProtection::Z => {
			assert_ne!(string.get(end_qubit), PauliLetter::Z);
			assert_ne!(target_letter, PauliLetter::Z);
			to_single.set(end_qubit, PauliLetter::Z);
		}
		QubitProtection::None => {
			if target_letter != string.get(end_qubit) {
				let mut letter = string.get(end_qubit).next();
				if letter == target_letter {
					letter = letter.next();
				}
				to_single.set(end_qubit, letter);
			} else {
				to_single.set(end_qubit, string.get(end_qubit).next());
			}
		}
	}
	string.pi_over_4_sandwitch(false, &to_single);
	pushing.push(to_single);

	assert_eq!(string.len(), 1);
	assert_ne!(string.get(end_qubit), PauliLetter::I);

	// add single qubit gate if needed to set target correct
	if string.get(end_qubit) != target_letter {
		match protection {
			QubitProtection::X => {
				string.pi_over_4_sandwitch(false, &PauliString::x(end_qubit));
				pushing.push(PauliString::x(end_qubit));
			}
			QubitProtection::Z => {
				string.pi_over_4_sandwitch(false, &PauliString::z(end_qubit));
				pushing.push(PauliString::z(end_qubit));
			}
			QubitProtection::None => {
				let mut letter = target_letter.next();
				if letter == string.get(end_qubit) {
					letter = letter.next();
				}
				let mut new_string = PauliString::id();
				new_string.set(end_qubit, letter);
				string.pi_over_4_sandwitch(false, &new_string);
				pushing.push(new_string);
			}
		}
	}

	assert_eq!(string.len(), 1);
	assert_eq!(string.get(end_qubit), target_letter);

	pushing
}

fn fastest<const N: usize>(
	tableau: &CliffordTableau<N>,
	dirty_qubits: &[usize],
	gate_size: NonZeroEvenUsize,
) -> Option<(usize, PauliLetter)> {
	let mut res: Option<(usize, usize, PauliLetter)> = None;
	for qubit in dirty_qubits {
		let x_steps = {
			let string = tableau.x.get(*qubit).unwrap();
			let basic_steps = string.steps_to_len_one(gate_size);
			if string.get(*qubit) == PauliLetter::I
				&& string.len() >= gate_size.as_value()
				&& (string.len() - gate_size.as_value()) % (gate_size.as_value() - 1) == 0
			{
				// These steps are needed to move to the correct qubit
				basic_steps + 2
			} else {
				basic_steps
			}
		};

		let z_steps = {
			let string = tableau.z.get(*qubit).unwrap();
			let basic_steps = string.steps_to_len_one(gate_size);
			if string.get(*qubit) == PauliLetter::I
				&& string.len() >= gate_size.as_value()
				&& (string.len() - gate_size.as_value()) % (gate_size.as_value() - 1) == 0
			{
				// These steps are needed to move to the correct qubit
				basic_steps + 2
			} else {
				basic_steps
			}
		};
		let letter = if x_steps <= z_steps {
			PauliLetter::X
		} else {
			PauliLetter::Z
		};

		let n_steps = x_steps.min(z_steps);

		match res {
			Some(v) => {
				if n_steps < v.0 {
					res = Some((n_steps, *qubit, letter));
				}
			}
			None => {
				res = Some((n_steps, *qubit, letter));
			}
		}
	}

	res.map(|(_, qubit, letter)| (qubit, letter))
}

impl<const N: usize> CliffordTableau<N> {
	/// # THIS FUNCTION DOES NOT WORK YET
	///
	/// Decomposes the tableau into clifford gates.
	pub fn decompose(
		mut self,
		gate_size: NonZeroEvenUsize,
	) -> Vec<PauliExp<N, CliffordPauliAngle>> {
		let mut decomposition: Vec<PauliExp<N, CliffordPauliAngle>> = Vec::new();
		let mut dirty_qubits: Vec<usize> = (0..N).collect();

		while dirty_qubits.len() >= gate_size.as_value() {
			let (qubit, letter) = fastest(&self, &dirty_qubits, gate_size).unwrap();
			match letter {
				PauliLetter::X => {
					let x_moves = simple_solver(
						self.x.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::X,
						&dirty_qubits,
						QubitProtection::None,
					);
					for string in x_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
					let z_moves = simple_solver(
						self.z.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::Z,
						&dirty_qubits,
						QubitProtection::X,
					);
					for string in z_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
				}
				PauliLetter::Z => {
					let z_moves = simple_solver(
						self.z.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::Z,
						&dirty_qubits,
						QubitProtection::None,
					);
					for string in z_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
					let x_moves = simple_solver(
						self.x.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::X,
						&dirty_qubits,
						QubitProtection::Z,
					);
					for string in x_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
				}
				_ => unreachable!(),
			}

			// Now the qubit is not dirty anymore
			dirty_qubits.retain(|q| *q != qubit);
		}

		// then for remaining use delicate solver
		//     solve x
		//     solve y

		// Turn signs into correct ones

		self.info_print(N);

		decomposition
	}
}
