use crate::{
	connectivity::{Connectivity, RoutingInstruction, RoutingInstructionTarget},
	misc::NonZeroEvenUsize,
	pauli::{CliffordPauliAngle, FreePauliAngle, PauliAngle, PauliExp, PauliLetter, PauliString},
};

fn get_remove_indexes<F: Fn(&PauliExp<N, A>) -> bool, const N: usize, A: PauliAngle>(
	exponentials: &[PauliExp<N, A>],
	f: F,
) -> Vec<usize> {
	let mut indexes: Vec<usize> = Vec::new();

	let mut i: usize = 0;
	for exp in exponentials.iter() {
		if f(exp) {
			indexes.push(i);
		} else {
			i += 1;
		}
	}

	indexes
}

#[cfg(not(feature = "return_ordered"))]
type SynthesizeResult<const N: usize> = (
	Vec<PauliExp<N, FreePauliAngle>>,
	Vec<PauliExp<N, CliffordPauliAngle>>,
);

#[cfg(feature = "return_ordered")]
type SynthesizeResult<const N: usize> = (
	Vec<PauliExp<N, FreePauliAngle>>,
	Vec<PauliExp<N, CliffordPauliAngle>>,
	Vec<PauliExp<N, FreePauliAngle>>,
);

pub fn synthesize<const N: usize>(
	exponentials: Vec<PauliExp<N, FreePauliAngle>>,
	gate_size: NonZeroEvenUsize,
	connectivity: Option<&Connectivity>,
) -> SynthesizeResult<N> {
	match connectivity {
		Some(connectivity) => synthesize_with_connectivity(exponentials, gate_size, connectivity),
		_ => synthesize_full_connectivity(exponentials, gate_size),
	}
}

fn synthesize_with_connectivity<const N: usize>(
	mut exponentials: Vec<PauliExp<N, FreePauliAngle>>,
	gate_size: NonZeroEvenUsize,
	connectivity: &Connectivity,
) -> SynthesizeResult<N> {
	#[cfg(feature = "return_ordered")]
	let mut ordered: Vec<PauliExp<N, FreePauliAngle>> = Vec::new();
	#[cfg(feature = "return_ordered")]
	let mut ordered_clifford: Vec<PauliExp<N, FreePauliAngle>> = Vec::new();

	let mut circuit: Vec<PauliExp<N, FreePauliAngle>> = Vec::new();
	let mut clifford_part: Vec<PauliExp<N, CliffordPauliAngle>> = Vec::new();

	// move clifford gates to clifford part
	for clifford in exponentials.extract_if(.., |v| v.angle.is_clifford()) {
		if let FreePauliAngle::Clifford(angle) = clifford.angle {
			clifford_part.push(PauliExp {
				string: clifford.string.clone(),
				angle,
			});
			#[cfg(feature = "return_ordered")]
			ordered_clifford.push(clifford); // nope
		} else {
			unreachable!()
		}
	}

	#[cfg(feature = "return_ordered")]
	let mut clone: Vec<PauliExp<N, FreePauliAngle>> = exponentials.clone();

	// move single (an no) qubit gates to circuit
	let remove_indexes = get_remove_indexes(&exponentials, |p| p.len() <= 1);
	for i in remove_indexes.into_iter() {
		assert!(exponentials.get(i).unwrap().len() == 1);
		circuit.push(exponentials.remove(i));
		#[cfg(feature = "return_ordered")]
		ordered.push(clone.remove(i));
	}

	// main loop
	while !exponentials.is_empty() {
		// The fastest one to solve
		let (index, instructions) = {
			let mut iter = exponentials.iter();

			let first = iter.next().unwrap();
			let instructions = connectivity.get_routing_path(&first.string.targets());
			let steps = first
				.string
				.steps_to_solve_instructions(gate_size, &instructions);

			// (steps, index, instructions)
			let mut shortest = (steps, 0, instructions);
			for (i, exp) in iter.enumerate() {
				let instructions = connectivity.get_routing_path(&exp.string.targets());
				let steps = exp
					.string
					.steps_to_solve_instructions(gate_size, &instructions);
				if steps < shortest.0 {
					shortest = (steps, i + 1, instructions)
				}
			}

			(shortest.1, shortest.2)
		};

		let mut exp = exponentials.remove(index);
		for instruction in instructions {
			let push_strs = handle_instruction(exp.string.clone(), gate_size, instruction);

			for push_str in push_strs {
				exp.push_pi_over_4(false, &push_str);
				for exp in exponentials.iter_mut() {
					exp.push_pi_over_4(false, &push_str);
				}

				circuit.push(PauliExp {
					string: push_str.clone(),
					angle: FreePauliAngle::Clifford(CliffordPauliAngle::PiOver4),
				});
				clifford_part.push(PauliExp {
					string: push_str,
					angle: CliffordPauliAngle::NegPiOver4,
				});
			}
		}

		assert_eq!(exp.len(), 1);
		// add exp to circuit
		circuit.push(exp);
		#[cfg(feature = "return_ordered")]
		ordered.push(clone.remove(index));
	}

	#[cfg(feature = "return_ordered")]
	assert!(clone.is_empty());

	let clifford_part: Vec<PauliExp<N, CliffordPauliAngle>> =
		clifford_part.into_iter().rev().collect();

	#[cfg(feature = "return_ordered")]
	{
		ordered_clifford.reverse();
		ordered.append(&mut ordered_clifford);
	}

	#[cfg(feature = "return_ordered")]
	return (circuit, clifford_part, ordered);

	#[cfg(not(feature = "return_ordered"))]
	(circuit, clifford_part)
}

fn synthesize_full_connectivity<const N: usize>(
	mut exponentials: Vec<PauliExp<N, FreePauliAngle>>,
	gate_size: NonZeroEvenUsize,
) -> SynthesizeResult<N> {
	#[cfg(feature = "return_ordered")]
	let mut ordered: Vec<PauliExp<N, FreePauliAngle>> = Vec::new();
	#[cfg(feature = "return_ordered")]
	let mut ordered_clifford: Vec<PauliExp<N, FreePauliAngle>> = Vec::new();

	let n = gate_size.as_value();
	let mut circuit: Vec<PauliExp<N, FreePauliAngle>> = Vec::new();
	let mut clifford_part: Vec<PauliExp<N, CliffordPauliAngle>> = Vec::new();

	// move clifford gates to clifford part
	for clifford in exponentials.extract_if(.., |v| v.angle.is_clifford()) {
		if let FreePauliAngle::Clifford(angle) = clifford.angle {
			clifford_part.push(PauliExp {
				string: clifford.string.clone(),
				angle,
			});
			#[cfg(feature = "return_ordered")]
			ordered_clifford.push(clifford); // nope
		} else {
			unreachable!()
		}
	}

	#[cfg(feature = "return_ordered")]
	let mut clone: Vec<PauliExp<N, FreePauliAngle>> = exponentials.clone();

	// move single (an no) qubit gates to circuit
	let remove_indexes = get_remove_indexes(&exponentials, |p| p.len() <= 1);
	for i in remove_indexes.into_iter() {
		assert!(exponentials.get(i).unwrap().len() == 1);
		circuit.push(exponentials.remove(i));
		#[cfg(feature = "return_ordered")]
		ordered.push(clone.remove(i));
	}

	// main loop
	while !exponentials.is_empty() {
		// The fastest one to solve
		let index = {
			let mut iter = exponentials.iter();
			// (steps, index)
			let mut shortest = (iter.next().unwrap().string.steps_to_len_one(gate_size), 0);
			for (i, exp) in iter.enumerate() {
				let steps = exp.string.steps_to_len_one(gate_size);
				if steps < shortest.0 {
					shortest = (steps, i + 1)
				}
			}

			shortest.1
		};

		let mut exp = exponentials.remove(index);
		while exp.len() != 1 {
			let push_str = if exp.len() == n {
				// One commutes, the rest cancel each other out
				let mut push_str = exp.string.clone();
				let (i, l) = push_str.letters().into_iter().next().unwrap();
				push_str.set(i, l.next());

				push_str
			} else if exp.len() % 2 == 1 && exp.len() < (2 * n) {
				// Change the string into a n long one
				if exp.len() < n {
					// We need to increase the amount of letters
					// Because the exp has an uneven len, we can commute on all
					let mut push_str = exp.string.clone();
					for (i, m) in push_str.letters() {
						push_str.set(i, m.next());
					}

					// Then we just add letters to make it n long
					for i in 0..n {
						if push_str.get(i) == PauliLetter::I {
							push_str.set(i, PauliLetter::X);
							if push_str.len() == n {
								break;
							}
						}
					}

					push_str
				} else {
					// We need to decrease the amount of letters
					// Select n many letters
					let mut letters: Vec<(usize, PauliLetter)> =
						exp.string.letters().into_iter().take(n).collect();

					// We need to to end up with n letters. This means that we
					// need to remove exp.len() - n letters. This means that
					// from the n many letters, we only want to keep
					// n - (exp.len() -n ) = 2n - exp.len() many.
					// As these are the ones we keep, we make them anticommute
					// (the amount is always uneven because exp.len() is)
					for (_, l) in letters.iter_mut().take(2 * n - exp.len()) {
						*l = l.next();
					}

					// Then we just collect the letters to a string
					let mut push_str = PauliString::<N>::id();
					for (i, l) in letters {
						push_str.set(i, l);
					}
					push_str
				}
			} else {
				// Now either exp.len() is at least len 2n or even.
				// To reach a uneven len under 2n, we remove as much as possible
				// and if needed add some letters, because operations have to be
				// len n.

				// select at least n many letters
				let mut letters: Vec<(usize, PauliLetter)> =
					exp.string.letters().into_iter().take(n).collect();

				// edit first one in order to anticommute (and delete on others)
				letters.first_mut().unwrap().1 = letters.first_mut().unwrap().1.next();

				// collect as string
				let mut push_str = PauliString::<N>::id();
				for (i, l) in letters {
					push_str.set(i, l);
				}
				// add letters if needed to make operation n long

				if push_str.len() < n {
					for i in 0..n {
						if push_str.get(i) == PauliLetter::I {
							push_str.set(i, PauliLetter::X);
							if push_str.len() == n {
								break;
							}
						}
					}
				}

				push_str
			};

			assert_eq!(push_str.len(), n);

			// push string trough/into things

			exp.push_pi_over_4(false, &push_str);
			for exp in exponentials.iter_mut() {
				exp.push_pi_over_4(false, &push_str);
			}
			circuit.push(PauliExp {
				string: push_str.clone(),
				angle: FreePauliAngle::Clifford(CliffordPauliAngle::PiOver4),
			});
			clifford_part.push(PauliExp {
				string: push_str,
				angle: CliffordPauliAngle::NegPiOver4,
			});
		}

		assert_eq!(exp.len(), 1);
		// add exp to circuit
		circuit.push(exp);
		#[cfg(feature = "return_ordered")]
		ordered.push(clone.remove(index));
	}

	#[cfg(feature = "return_ordered")]
	assert!(clone.is_empty());

	let clifford_part: Vec<PauliExp<N, CliffordPauliAngle>> =
		clifford_part.into_iter().rev().collect();

	#[cfg(feature = "return_ordered")]
	{
		ordered_clifford.reverse();
		ordered.append(&mut ordered_clifford);
	}

	#[cfg(feature = "return_ordered")]
	return (circuit, clifford_part, ordered);

	#[cfg(not(feature = "return_ordered"))]
	(circuit, clifford_part)
}

pub(crate) fn handle_instruction<const N: usize>(
	mut string: PauliString<N>,
	gate_size: NonZeroEvenUsize,
	instruction: RoutingInstruction,
) -> Vec<PauliString<N>> {
	let n = gate_size.as_value();
	let mut push_strs: Vec<PauliString<N>> = Vec::new();

	// Target qubit
	let target = match instruction.target {
		RoutingInstructionTarget::Single(target) => target,
		RoutingInstructionTarget::Multiple(targets) => {
			let mut target = *targets.first().unwrap();
			for option in targets {
				if string.get(option) != PauliLetter::I {
					target = option;
					break;
				}
			}

			target
		}
		RoutingInstructionTarget::Any => *instruction.qubits.first().unwrap(),
	};

	let mut removable = Vec::new();
	for qubit in instruction.qubits {
		if string.get(*qubit) != PauliLetter::I && *qubit != target {
			removable.push(*qubit);
		}
	}

	// make shorter than 2n
	while removable.len() > 2 * n - 2 {
		let mut push_str = PauliString::id();

		// anticommute on one
		let anticommute_on = *removable.first().unwrap();
		push_str.set(anticommute_on, string.get(anticommute_on).next());

		// remove on rest
		for _ in 0..(n - 1) {
			let i = removable.pop().unwrap();
			push_str.set(i, string.get(i));
		}

		assert_eq!(push_str.len(), n);

		string.pi_over_4_sandwitch(false, &push_str);
		push_strs.push(push_str);
	}

	// make sure we have target
	if string.get(target) == PauliLetter::I {
		let mut push_str: PauliString<N> = PauliString::id();

		// Adding the target qubit
		push_str.set(target, PauliLetter::X);

		// We add one, and then want to have n left
		let mut n_remove = (removable.len() + 2).saturating_sub(n);
		// Because we need to add one qubit and anticommute on one, we can only remove an
		// even amount.
		if n_remove % 2 == 1 {
			n_remove += 1;
		}
		// The amount of removals is limited by the gate size
		let n_remove = n_remove.min(n - 2);

		for _ in 0..n_remove {
			let i = removable.pop().unwrap();
			push_str.set(i, string.get(i));
		}

		// We anticommute on as many as possible (keeping the amount uneven)
		for qubit in removable.iter().rev().take(if removable.len() % 2 == 0 {
			removable.len() - 1
		} else {
			removable.len()
		}) {
			if push_str.len() == n {
				break;
			}
			push_str.set(*qubit, string.get(*qubit).next());
		}

		// If we do not have the option to anticommute on more, we need to add letters
		for qubit in instruction.qubits.iter().take(n + 1) {
			if push_str.len() == n {
				break;
			}
			if push_str.get(*qubit) != PauliLetter::I {
				continue;
			}
			if removable.contains(qubit) {
				continue;
			}

			removable.push(*qubit);
			push_str.set(*qubit, PauliLetter::X);
		}

		// We may need to remove one more. This is because we need to anticommute on uneven amount
		if push_str.len() < n {
			assert_eq!(push_str.len(), n - 1);
			let remove = removable.remove(0);
			push_str.set(remove, string.get(remove));
		}

		assert_eq!(push_str.len(), n);
		string.pi_over_4_sandwitch(false, &push_str);
		push_strs.push(push_str);
	}

	// if even and not 4 make so that we are at uneven
	if removable.len() != n - 1 && (removable.len() + 1).is_multiple_of(2) {
		let mut push_str: PauliString<N> = PauliString::id();

		// delete as many as can
		let n_remove = removable.len().min(n - 1);
		for qubit in removable.drain(0..n_remove) {
			push_str.set(qubit, string.get(qubit));
		}

		// anticommute on target
		push_str.set(target, string.get(target).next());

		// Fill in if needed
		for qubit in instruction.qubits.iter().take(n) {
			if push_str.len() == n {
				break;
			}
			if push_str.get(*qubit) != PauliLetter::I || *qubit == target {
				continue;
			}

			removable.push(*qubit);
			push_str.set(*qubit, PauliLetter::X);
		}

		assert_eq!(push_str.len(), n);
		string.pi_over_4_sandwitch(false, &push_str);
		push_strs.push(push_str);
	}

	// if uneven make so that len n
	if !removable.is_empty() && removable.len() % 2 == 0 {
		let len = removable.len() + 1;
		if len < n {
			// We need to add qubits
			let mut push_str = PauliString::id();

			// Commute on current letters
			for i in removable.iter() {
				push_str.set(*i, string.get(*i).next())
			}

			// Commute on target
			push_str.set(target, string.get(target).next());

			// these are added
			for qubit in instruction.qubits.iter() {
				if push_str.get(*qubit) == PauliLetter::I {
					push_str.set(*qubit, PauliLetter::X);
					removable.push(*qubit);
					if push_str.len() == n {
						break;
					}
				}
			}

			assert_eq!(push_str.len(), n);
			string.pi_over_4_sandwitch(false, &push_str);
			push_strs.push(push_str);
		} else {
			// We need to remove qubits
			let mut letters: Vec<(usize, PauliLetter)> =
				removable.drain(0..n).map(|i| (i, string.get(i))).collect();

			// These are the ones we keep (we make them anticommute)
			for (i, l) in letters.iter_mut().take(2 * n - len) {
				// We need to add these back to the removable ones
				removable.push(*i);
				*l = l.next();
			}

			// The rest of the qubits are removed
			let mut push_str = PauliString::<N>::id();
			for (i, l) in letters {
				push_str.set(i, l);
			}

			assert_eq!(push_str.len(), n);
			string.pi_over_4_sandwitch(false, &push_str);
			push_strs.push(push_str);
		}
	}

	assert!(removable.is_empty() || removable.len() == n - 1);

	// convert to single qubit if len == n
	if removable.len() == n - 1 {
		let mut push_str: PauliString<N> = PauliString::id();
		// commute on target
		push_str.set(target, string.get(target).next());

		// remove the rest
		for qubit in removable.iter() {
			push_str.set(*qubit, string.get(*qubit));
		}

		assert_eq!(push_str.len(), n);
		string.pi_over_4_sandwitch(false, &push_str);
		push_strs.push(push_str);
	}

	for string in push_strs.iter() {
		assert_eq!(string.len(), n)
	}

	push_strs
}

#[cfg(test)]
mod tests {
	use super::*;
	use rand::prelude::*;

	fn random_exp<const N: usize, R: Rng>(rng: &mut R) -> PauliExp<N, FreePauliAngle> {
		let n_letters = (1_usize..=N).choose(rng);
		let mut selection: Vec<usize> = (0..N).collect();
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
			angle: FreePauliAngle::MultipleOfPi(rng.random()),
		}
	}

	#[test]
	fn synthesize_result_has_suitable_operators() {
		for _ in 0..10 {
			let mut rng = rand::rng();
			let input: Vec<PauliExp<30, FreePauliAngle>> = (0..30)
				.map(move |_| random_exp::<30, _>(&mut rng))
				.collect();

			#[cfg(not(feature = "return_ordered"))]
			let (circuit, clifford) = synthesize(input, NonZeroEvenUsize::new(4).unwrap(), None);

			#[cfg(feature = "return_ordered")]
			let (circuit, clifford, _) = synthesize(input, NonZeroEvenUsize::new(4).unwrap(), None);

			for exp in circuit {
				assert!(exp.len() == 1 || exp.len() == 4);
			}

			for exp in clifford {
				assert!(exp.len() == 1 || exp.len() == 4);
			}
		}
	}
}
