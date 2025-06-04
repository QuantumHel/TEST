use crate::{
	misc::NonZeroEvenUsize,
	pauli::{CliffordPauliAngle, FreePauliAngle, PauliAngle, PauliExp, PauliLetter, PauliString},
};

fn get_all<F: Fn(&PauliExp<N, A>) -> bool, const N: usize, A: PauliAngle>(
	exponentials: &mut Vec<PauliExp<N, A>>,
	f: F,
) -> Vec<PauliExp<N, A>> {
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

pub fn synthesize<const N: usize>(
	mut exponentials: Vec<PauliExp<N, FreePauliAngle>>,
	gate_size: NonZeroEvenUsize,
) -> (
	Vec<PauliExp<N, FreePauliAngle>>,
	Vec<PauliExp<N, CliffordPauliAngle>>,
) {
	// TODO: merge equal pauli strings
	let n = gate_size.as_value();
	let mut circuit: Vec<PauliExp<N, FreePauliAngle>> = Vec::new();
	let mut clifford_tableau: Vec<PauliExp<N, CliffordPauliAngle>> = Vec::new();

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
			let (i, l) = push_str.letters().into_iter().next().unwrap();
			push_str.set(i, l.next());

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
				for (i, m) in push_str.letters() {
					push_str.set(i, m.next());
				}

				// These will be added on push
				for i in 0..(n) {
					if push_str.get(i) == PauliLetter::I {
						push_str.set(i, PauliLetter::X);
						if push_str.len() == n {
							break;
						}
					}
				}

				push_str
			} else {
				// remove some if too many
				let mut letters: Vec<(usize, PauliLetter)> =
					exp.string.letters().into_iter().take(n).collect();

				// These are the ones we keep (we make them anticommute)
				for (_, l) in letters.iter_mut().take(2 * n - exp.len()) {
					*l = l.next();
				}

				let mut string = PauliString::<N>::id();
				for (i, l) in letters {
					string.set(i, l);
				}
				string
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

				let mut push_str = PauliString::<N>::id();
				let (i, l) = exp.string.letters().into_iter().next().unwrap();
				push_str.set(i, l);

				// These will be added on push
				for i in 0..(2 * n) {
					if push_str.get(i) == PauliLetter::I {
						push_str.set(i, PauliLetter::X);
						if push_str.len() == n {
							break;
						}
					}
				}

				push_str
			} else {
				// By removing n-1 qubit letters from the exp we can make it compatible with if let
				// above the if let that we took.
				let mut letters: Vec<(usize, PauliLetter)> =
					exp.string.letters().into_iter().take(n).collect();
				let difference = letters.pop().unwrap();
				letters.push((difference.0, difference.1.next()));

				let mut string = PauliString::<N>::id();
				for (i, l) in letters {
					string.set(i, l);
				}
				string
			}
		} else {
			// Else remove as many qubits as possible from the shortest exponential so that we can
			// access some of the cases above. This can take multiple rounds.

			// Select shortest
			let exp = exponentials
				.iter()
				.reduce(|acc, e| if acc.len() <= e.len() { acc } else { e })
				.unwrap();

			let mut letters: Vec<(usize, PauliLetter)> =
				exp.string.letters().into_iter().take(n).collect();
			let difference = letters.pop().unwrap();
			letters.push((difference.0, difference.1.next()));

			let mut string = PauliString::<N>::id();
			for (i, l) in letters {
				string.set(i, l);
			}
			string
		};

		assert!(push_str.len() == n);

		for exp in exponentials.iter_mut() {
			exp.push_pi_over_4(true, &push_str);
		}
		circuit.push(PauliExp {
			string: push_str.clone(),
			angle: FreePauliAngle::Clifford(CliffordPauliAngle::PiOver4),
		});
		clifford_tableau.push(PauliExp {
			string: push_str,
			angle: CliffordPauliAngle::NeqPiOver4,
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

	fn random_exp<const N: usize>(rng: &mut WyRand) -> PauliExp<N, FreePauliAngle> {
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

		PauliExp {
			string,
			angle: FreePauliAngle::Free(rng.generate()),
		}
	}

	#[test]
	fn synthesize_result_has_suitable_operators() {
		for _ in 0..1 {
			let mut rng = WyRand::new();
			let input: Vec<PauliExp<30, FreePauliAngle>> =
				(0..30).map(move |_| random_exp::<30>(&mut rng)).collect();

			let (circuit, clifford) = synthesize(input, NonZeroEvenUsize::new(4).unwrap());

			for exp in circuit {
				assert!(exp.len() == 1 || exp.len() == 4);
			}

			for exp in clifford {
				assert!(exp.len() == 1 || exp.len() == 4);
			}
		}
	}
}
