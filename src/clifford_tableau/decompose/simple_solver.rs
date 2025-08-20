use crate::{
	clifford_tableau::{CliffordTableau, decompose::QubitProtection},
	misc::NonZeroEvenUsize,
	pauli::{PauliLetter, PauliString},
};

/// Assumes that there are at least gate size many dirty qubits
pub fn simple_solver<const N: usize>(
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
		// check here?
		assert_eq!(protection, QubitProtection::None);
		let mut new_string: PauliString<N> = PauliString::id();
		new_string.set(end_qubit, target_letter.next());

		let mut n_remove = (string.len() + 1).saturating_sub(n);
		if n_remove % 2 == 1 {
			n_remove += 1;
		}
		let n_remove = n_remove.min(n - 2);
		let letters = string.letters();
		let mut iter = letters.iter();
		for _ in 0..n_remove {
			let (index, letter) = iter.next().unwrap();
			new_string.set(*index, *letter);
		}

		let mut rest: Vec<_> = iter.collect();
		let additional = if !rest.is_empty() && rest.len() % 2 == 0 {
			Some(rest.pop().unwrap())
		} else {
			None
		};

		for (index, letter) in rest {
			if new_string.len() == n {
				break;
			}
			new_string.set(*index, letter.next());
		}

		for qubit in dirty_qubits {
			if new_string.len() == n {
				break;
			}

			if new_string.get(*qubit) != PauliLetter::I {
				continue;
			}

			new_string.set(*qubit, PauliLetter::X);
		}

		// We may need to remove one more. This is because we need to anticommute on uneven amount
		if new_string.len() < n {
			assert_eq!(new_string.len(), n - 1);
			assert!(additional.is_some());
			let (index, letter) = additional.unwrap();
			new_string.set(*index, *letter);
		}

		assert_eq!(new_string.len(), n);
		string.pi_over_4_sandwitch(false, &new_string);
		pushing.push(new_string);
	}

	// if even and not 4, make it so that we are uneven and under 4
	if string.len().is_multiple_of(2) && string.len() != n {
		let mut new_string: PauliString<N> = PauliString::id();

		// anticummute on target to keep it
		match protection {
			QubitProtection::X => {
				new_string.set(end_qubit, PauliLetter::X);
			}
			QubitProtection::Z => {
				new_string.set(end_qubit, PauliLetter::Z);
			}
			QubitProtection::None => {
				new_string.set(end_qubit, string.get(end_qubit).next());
			}
		}

		// remove as many as possible
		let letters = string.letters();
		let iter = letters.iter().filter(|(i, _)| *i != end_qubit);
		for (index, letter) in iter {
			if new_string.len() == n {
				break;
			}
			new_string.set(*index, *letter);
		}

		// make sure that the len is correct
		for qubit in dirty_qubits {
			if new_string.len() == n {
				break;
			}

			if new_string.get(*qubit) == PauliLetter::I {
				new_string.set(*qubit, PauliLetter::X);
			}
		}

		string.pi_over_4_sandwitch(false, &new_string);
		pushing.push(new_string);
	}

	assert!(string.len() % 2 == 1 || string.len() == n);

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

pub fn fastest<const N: usize>(
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
				&& (string.len() - gate_size.as_value()).is_multiple_of(gate_size.as_value() - 1)
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
				&& (string.len() - gate_size.as_value()).is_multiple_of(gate_size.as_value() - 1)
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

#[cfg(test)]
mod test {
	use crate::{
		clifford_tableau::decompose::{QubitProtection, simple_solver::simple_solver},
		misc::NonZeroEvenUsize,
		pauli::{PauliLetter, PauliString},
		pauli_string,
	};

	#[test]
	fn asd() {
		let mut string = pauli_string!("IXXI");
		let gate_size = NonZeroEvenUsize::new(4).unwrap();
		let end_qubit = 0;
		let target_letter = PauliLetter::X;
		let dirty_qubits = vec![0, 1, 2, 3];
		let protection = QubitProtection::None;

		let strings = simple_solver(
			&string,
			gate_size,
			end_qubit,
			target_letter,
			&dirty_qubits,
			protection,
		);

		for push in strings {
			string.pi_over_4_sandwitch(false, &push);
		}

		assert_eq!(string, PauliString::x(0));
	}
}
