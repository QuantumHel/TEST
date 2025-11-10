use crate::{
	clifford_tableau::CliffordTableau,
	misc::NonZeroEvenUsize,
	pauli::{PauliLetter, PauliString},
};

pub fn fastest_delicate(
	tableau: &CliffordTableau,
	dirty_qubits: &[usize],
) -> Option<(usize, PauliLetter)> {
	let mut res: Option<(usize, usize, PauliLetter)> = None;
	for qubit in dirty_qubits {
		let x_steps: usize = {
			let string = tableau.get_x_row(*qubit).0;
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
			let string = tableau.get_z_row(*qubit).0;
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
pub fn delicate_solver(
	string: &PauliString,
	gate_size: NonZeroEvenUsize,
	target_qubit: usize,
	target_letter: PauliLetter,
	usable_qubits: Option<&[usize]>,
) -> Vec<PauliString> {
	if !(target_letter == PauliLetter::X || target_letter == PauliLetter::Z) {
		panic!("delicate solver only solves for X and Z");
	}

	if string.len() == 1 && string.get(target_qubit) == target_letter {
		return Vec::new();
	}

	let n = gate_size.as_value();
	let range: Vec<_> = (0..n).collect();
	let usable_qubits = match usable_qubits {
		Some(usable_qubits) => usable_qubits,
		_ => &range,
	};

	let mut pushing: Vec<PauliString> = Vec::new();

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
			let mut push = PauliString::id();
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
			.filter(|(q, _)| *q != target_qubit)
			.collect();

		if string.len().is_multiple_of(2) {
			// All uninvolved qubits have X in first string and Z on second.
			// The non target involved qubits always have their letter in the string
			// Target qubit has target letter
			let mut outer1 = PauliString::id();
			let mut outer2 = PauliString::id();

			// middle string
			// All non involved qubits have Y
			// All non target involved qubits have original letter
			// Target qubit has the one that is not target and not the one in string
			let mut inner = PauliString::id();

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

			for i in usable_qubits.iter() {
				if outer1.len() == n {
					break;
				}

				if outer1.get(*i) == PauliLetter::I {
					outer1.set(*i, PauliLetter::X);
					outer2.set(*i, PauliLetter::Z);
					inner.set(*i, PauliLetter::Y);
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
			let mut outer = PauliString::id();

			// inner (1)
			// All non involved qubits get Y
			// All non target involved qubits get the one that is not involved and not involved.next
			// target qubit gets the one in string
			let mut inner = PauliString::id();

			for (qubit, letter) in other.iter() {
				outer.set(*qubit, letter.next());
				inner.set(*qubit, letter.next().next());
			}

			outer.set(target_qubit, target_letter);
			inner.set(target_qubit, old_target);

			for i in usable_qubits.iter() {
				if outer.len() == n {
					break;
				}

				if outer.get(*i) == PauliLetter::I {
					outer.set(*i, PauliLetter::Y);
					inner.set(*i, PauliLetter::Y);
				}
			}

			pushing.push(outer.clone());
			pushing.push(inner);
			pushing.push(outer);
		}
	} else {
		// else (if target qubit not free). This does not protect the target qubit, but needing to protect it should be impossible

		let other: Vec<(usize, PauliLetter)> = string.letters().collect();

		if string.len().is_multiple_of(2) {
			// 2 outer 2 inner
			// outer 1: uninvolved Y, target: target, first_inv: inv.next() = A, other: next
			let mut outer1 = PauliString::id();
			// outer 2: uninvolved X, target: other, first_inv: A.next(), other: next next
			let mut outer2 = PauliString::id();

			// inner 1: uninvolved: Z, target: target, first_inv: involved, other: nextnext
			let mut inner1 = PauliString::id();
			// inner 2: uninvolved: Y, target: other, first_inv: nextnext, other: involved
			let mut inner2 = PauliString::id();

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

			for i in usable_qubits.iter() {
				if outer1.len() == n {
					break;
				}

				if outer1.get(*i) == PauliLetter::I {
					outer1.set(*i, PauliLetter::Y);
					outer2.set(*i, PauliLetter::X);
					inner1.set(*i, PauliLetter::Z);
					inner2.set(*i, PauliLetter::Y);
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
			let mut outer1 = PauliString::id();
			// outer 2: uninvolved Z, target: Y, other involved: A.next()
			let mut outer2 = PauliString::id();

			// inner 1: uninvolved Y, taget: opposite of target letter (X->Z, Z-> X), other same as outer 2
			let mut inner1 = PauliString::id();
			// inner 2: non target involved = outer1, all other (target+uninvolved) Y
			let mut inner2 = PauliString::id();

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

			for i in usable_qubits.iter() {
				if outer1.len() == n {
					break;
				}

				if outer1.get(*i) == PauliLetter::I {
					outer1.set(*i, PauliLetter::X);
					outer2.set(*i, PauliLetter::Z);
					inner1.set(*i, PauliLetter::Y);
					inner2.set(*i, PauliLetter::Y);
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
