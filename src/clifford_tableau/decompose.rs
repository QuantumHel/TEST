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

fn fastest_delicate<const N: usize>(
	tableau: &CliffordTableau<N>,
	dirty_qubits: &[usize],
) -> Option<(usize, PauliLetter)> {
	let mut res: Option<(usize, usize, PauliLetter)> = None;
	for qubit in dirty_qubits {
		let x_steps: usize = {
			let string = tableau.get_x_row(*qubit).unwrap().0;
			if string.get(*qubit) != PauliLetter::I {
				if string.len() == 1 {
					return Some((*qubit, PauliLetter::X));
				}
				if string.len().is_multiple_of(2) { 5 } else { 3 }
			} else {
				6
			}
		};
		let z_steps: usize = {
			let string = tableau.get_z_row(*qubit).unwrap().0;
			if string.get(*qubit) != PauliLetter::I {
				if string.len() == 1 {
					return Some((*qubit, PauliLetter::Z));
				}
				if string.len().is_multiple_of(2) { 5 } else { 3 }
			} else {
				6
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

/// Does not edit rows for solved qubits, but is expensive
///
/// Target letter has to be X or Z
fn delicate_solver<const N: usize>(
	string: &PauliString<N>,
	gate_size: NonZeroEvenUsize,
	target_qubit: usize,
	target_letter: PauliLetter,
) -> Vec<PauliString<N>> {
	if !(target_letter == PauliLetter::X || target_letter == PauliLetter::Z) {
		panic!("delicate solver only solves for X and Z");
	}

	if string.len() == 1 && string.get(target_qubit) == target_letter {
		return Vec::new();
	}

	let n = gate_size.as_value();
	let mut pushing: Vec<PauliString<N>> = Vec::new();

	if string.len() == 1 && string.get(target_qubit) != PauliLetter::I {
		let letter = if target_letter.next() != string.get(target_qubit) {
			target_letter.next()
		} else {
			target_letter.next().next()
		};

		let mut push = PauliString::id();
		push.set(target_qubit, letter);
		pushing.push(push);
	} else if string.get(target_qubit) != PauliLetter::I {
		let mut string = string.clone();

		// use single qubit gate to make sure that target qubit does not have target letter
		if string.get(target_qubit) == target_letter {
			let mut push: PauliString<N> = PauliString::id();
			match target_letter {
				PauliLetter::X => {
					push.set(target_qubit, PauliLetter::Z);
				}
				PauliLetter::Z => {
					push.set(target_qubit, PauliLetter::X);
				}
				_ => unreachable!(),
			}

			string.pi_over_4_sandwitch(false, &push);
			pushing.push(push);
		}

		let old_target = string.get(target_qubit);
		let other: Vec<(usize, PauliLetter)> = string
			.letters()
			.into_iter()
			.filter(|(q, _)| *q != target_qubit)
			.collect();

		if string.len().is_multiple_of(2) {
			// All uninvolved qubits have X in first string and Z on second.
			// The non target involved qubits always have their letter in the string
			// Target qubit has target letter
			let mut outer1: PauliString<N> = PauliString::id();
			let mut outer2: PauliString<N> = PauliString::id();

			// middle string
			// All non involved qubits have Y
			// All non target involved qubits have original letter
			// Target qubit has the one that is not target and not the one in string
			let mut inner: PauliString<N> = PauliString::id();

			for (qubit, letter) in other.iter() {
				outer1.set(*qubit, *letter);
				outer2.set(*qubit, *letter);
				inner.set(*qubit, *letter);
			}

			outer1.set(target_qubit, target_letter);
			outer2.set(target_qubit, target_letter);

			if old_target.next() != target_letter {
				inner.set(target_qubit, old_target.next());
			} else {
				inner.set(target_qubit, old_target.next().next());
			}

			for i in 0..n {
				if outer1.len() == n {
					break;
				}

				if outer1.get(i) == PauliLetter::I {
					outer1.set(i, PauliLetter::X);
					outer2.set(i, PauliLetter::Z);
					inner.set(i, PauliLetter::Y);
				}
			}

			pushing.push(outer1.clone());
			pushing.push(outer2.clone());
			pushing.push(inner);
			pushing.push(outer2);
			pushing.push(outer1);
		} else {
			// outer (1)
			// All non involved qubits have Y
			// All non target involved qubits get involved.next
			// target qubit gets target letter
			let mut outer: PauliString<N> = PauliString::id();

			// inner (1)
			// All non involved qubits get Y
			// All non target involved qubits get the one that is not involved and not involved.next
			// target qubit gets the one in string
			let mut inner: PauliString<N> = PauliString::id();

			for (qubit, letter) in other.iter() {
				outer.set(*qubit, letter.next());
				inner.set(*qubit, letter.next().next());
			}

			outer.set(target_qubit, target_letter);
			inner.set(target_qubit, old_target);

			for i in 0..n {
				if outer.len() == n {
					break;
				}

				if outer.get(i) == PauliLetter::I {
					outer.set(i, PauliLetter::Y);
					inner.set(i, PauliLetter::Y);
				}
			}

			pushing.push(outer.clone());
			pushing.push(inner);
			pushing.push(outer);
		}
	} else {
		// else (if target qubit not free). This does not protect the target qubit, but needing to protect it should be impossible

		let other: Vec<(usize, PauliLetter)> = string.letters();

		if string.len().is_multiple_of(2) {
			// 2 outer 2 inner
			// outer 1: uninvolved Y, target: target, first_inv: inv.next() = A, other: next
			let mut outer1: PauliString<N> = PauliString::id();
			// outer 2: uninvolved X, target: other, first_inv: A.next(), other: next next
			let mut outer2: PauliString<N> = PauliString::id();

			// inner 1: uninvolved: Z, target: target, first_inv: involved, other: nextnext
			let mut inner1: PauliString<N> = PauliString::id();
			// inner 2: uninvolved: Y, target: other, first_inv: nextnext, other: involved
			let mut inner2: PauliString<N> = PauliString::id();

			let mut other = other.into_iter();
			let (first_qubit, first_letter) = other.next().unwrap();
			outer1.set(first_qubit, first_letter.next());
			outer2.set(first_qubit, first_letter.next().next());
			inner1.set(first_qubit, first_letter);
			inner2.set(first_qubit, first_letter.next().next());

			for (qubit, letter) in other {
				outer1.set(qubit, letter.next());
				outer2.set(qubit, letter.next().next());
				inner1.set(qubit, letter.next().next());
				inner2.set(qubit, letter);
			}

			outer1.set(target_qubit, target_letter);
			inner1.set(target_qubit, target_letter);

			if target_letter == PauliLetter::X {
				outer2.set(target_qubit, PauliLetter::Z);
				inner2.set(target_qubit, PauliLetter::Z);
			} else if target_letter == PauliLetter::Z {
				outer2.set(target_qubit, PauliLetter::X);
				inner2.set(target_qubit, PauliLetter::X);
			} else {
				unreachable!()
			}

			for i in 0..n {
				if outer1.len() == n {
					break;
				}

				if outer1.get(i) == PauliLetter::I {
					outer1.set(i, PauliLetter::Y);
					outer2.set(i, PauliLetter::X);
					inner1.set(i, PauliLetter::Z);
					inner2.set(i, PauliLetter::Y);
				}
			}

			pushing.push(outer1.clone());
			pushing.push(outer2.clone());
			pushing.push(inner1);
			pushing.push(inner2);
			pushing.push(outer2);
			pushing.push(outer1);
		} else {
			// 2 outer, 2 inner

			// outer 1: uninvolved X, target: Y, all other involved take involved.next() = A
			let mut outer1: PauliString<N> = PauliString::id();
			// outer 2: uninvolved Z, target: Y, other involved: A.next()
			let mut outer2: PauliString<N> = PauliString::id();

			// inner 1: uninvolved Y, taget: opposite of target letter (X->Z, Z-> X), other same as outer 2
			let mut inner1: PauliString<N> = PauliString::id();
			// inner 2: non target involved = outer1, all other (target+uninvolved) Y
			let mut inner2: PauliString<N> = PauliString::id();

			for (qubit, letter) in other.into_iter() {
				outer1.set(qubit, letter.next());
				outer2.set(qubit, letter.next().next());
				inner1.set(qubit, letter.next().next());
				inner2.set(qubit, letter.next());
			}

			outer1.set(target_qubit, PauliLetter::Y);
			outer2.set(target_qubit, PauliLetter::Y);
			if target_letter == PauliLetter::X {
				inner1.set(target_qubit, PauliLetter::Z);
			} else {
				inner1.set(target_qubit, PauliLetter::X);
			}
			inner2.set(target_qubit, PauliLetter::Y);

			for i in 0..n {
				if outer1.len() == n {
					break;
				}

				if outer1.get(i) == PauliLetter::I {
					outer1.set(i, PauliLetter::X);
					outer2.set(i, PauliLetter::Z);
					inner1.set(i, PauliLetter::Y);
					inner2.set(i, PauliLetter::Y);
				}
			}

			pushing.push(outer1.clone());
			pushing.push(outer2.clone());
			pushing.push(inner1);
			pushing.push(inner2);
			pushing.push(outer2);
			pushing.push(outer1);
		}
	}

	let mut string = string.clone();
	for push in pushing.iter() {
		assert!(push.len() == 1 || push.len() == n);
		string.pi_over_4_sandwitch(false, push);
	}

	assert_eq!(string.len(), 1);
	assert_eq!(string.get(target_qubit), target_letter);

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
		let n_remove = n_remove.min(n - 2);
		let letters = string.letters();
		let mut iter = letters.iter();
		for _ in 0..n_remove {
			let (index, letter) = iter.next().unwrap();
			new_string.set(*index, *letter);
		}

		for (index, letter) in iter {
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

impl<const N: usize> CliffordTableau<N> {
	/// # Decompose
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
		while !dirty_qubits.is_empty() {
			let (qubit, letter) = fastest_delicate(&self, &dirty_qubits).unwrap();

			match letter {
				PauliLetter::X => {
					let x_moves = delicate_solver(
						self.x.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::X,
					);
					for string in x_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
					let z_moves = delicate_solver(
						self.z.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::Z,
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
					let z_moves = delicate_solver(
						self.z.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::Z,
					);
					for string in z_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
					let x_moves = delicate_solver(
						self.x.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::X,
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

		for (i, (x, z)) in self
			.x_signs
			.clone()
			.into_iter()
			.zip(self.z_signs.clone().into_iter())
			.enumerate()
		{
			let string = match (x, z) {
				(true, true) => PauliString::y(i),
				(true, false) => PauliString::z(i),
				(false, true) => PauliString::x(i),
				_ => {
					continue;
				}
			};

			self.merge_pi_over_4_pauli(true, &string);
			self.merge_pi_over_4_pauli(true, &string);
			decomposition.push(PauliExp {
				string: string.clone(),
				angle: CliffordPauliAngle::PiOver4,
			});
			decomposition.push(PauliExp {
				string,
				angle: CliffordPauliAngle::PiOver4,
			});
		}

		assert_eq!(self, CliffordTableau::id());

		decomposition.into_iter().rev().collect()
	}
}
