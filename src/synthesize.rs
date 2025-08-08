use crate::{
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

/// Gives the smallest value k for witch the length of exp is smaller or equal to n + k(n-1) where k
/// is even if len exp is, and uneven if len exp is
fn get_path_len<const N: usize, A: PauliAngle>(exp: &PauliExp<N, A>, n: usize) -> usize {
	let len = exp.len();
	if len < n {
		return if len.is_multiple_of(2) { 0 } else { 1 };
	}

	let len_over = (len - n) as f64;
	let mut k = (len_over / (n - 1) as f64).ceil() as usize;

	// Make sure that k is even if and onfly if len is
	if k % 2 != len % 2 {
		k += 1
	}

	k
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
	mut exponentials: Vec<PauliExp<N, FreePauliAngle>>,
	gate_size: NonZeroEvenUsize,
) -> SynthesizeResult<N> {
	#[cfg(feature = "return_ordered")]
	let mut clone: Vec<PauliExp<N, FreePauliAngle>> = exponentials.clone();
	#[cfg(feature = "return_ordered")]
	let mut ordered: Vec<PauliExp<N, FreePauliAngle>> = Vec::new();

	let n = gate_size.as_value();
	let mut circuit: Vec<PauliExp<N, FreePauliAngle>> = Vec::new();
	let mut clifford_tableau: Vec<PauliExp<N, CliffordPauliAngle>> = Vec::new();

	// move single (an no) qubit gates to circuit
	let remove_indexes = get_remove_indexes(&exponentials, |p| p.len() <= 1);
	for i in remove_indexes.into_iter() {
		assert!(exponentials.get(i).unwrap().len() == 1);
		circuit.push(exponentials.remove(i));
		#[cfg(feature = "return_ordered")]
		ordered.push(clone.remove(i));
	}

	// This is the main synthesize loop.
	// - Select an exponential to pus.
	// - push trough exponentials
	// - add corresponding exponentials to circuit and clifford_tableau
	// - move single qubit exponentials to the circuit.
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
				// We need to make the exponential longer
				let mut push_str = exp.string.clone();
				// Because uneven this makes sure that we anticommute and that all letter places
				// stay.
				for (i, m) in push_str.letters() {
					push_str.set(i, m.next());
				}

				// These will be added on push
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
				// Because we can add at maximum n-1 letters, we can select any commuting string to be able to select the earlier
				// option next round.

				// We have one difference in order to anticommute
				let mut letters = exp.string.letters();
				letters.first_mut().unwrap().1 = letters.first_mut().unwrap().1.next();

				let mut push_str = PauliString::<N>::id();
				for (i, l) in letters {
					push_str.set(i, l);
				}

				// Fill letters to he a n long string
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
				// By removing n-1 qubit letters from the exp we can make it compatible with if let
				// above the if let that we took.
				let mut letters: Vec<(usize, PauliLetter)> =
					exp.string.letters().into_iter().take(n).collect();
				letters.first_mut().unwrap().1 = letters.first_mut().unwrap().1.next();

				let mut string = PauliString::<N>::id();
				for (i, l) in letters {
					string.set(i, l);
				}
				string
			}
		} else {
			// Else remove as many qubits as possible from the exponential that
			// can be converted to a n one in least amount of steps.

			// Select shortest path one
			let exp = exponentials
				.iter()
				.reduce(|acc, e| {
					if get_path_len(acc, n) <= get_path_len(e, n) {
						acc
					} else {
						e
					}
				})
				.unwrap();

			let mut letters: Vec<(usize, PauliLetter)> =
				exp.string.letters().into_iter().take(n).collect();
			letters.first_mut().unwrap().1 = letters.first_mut().unwrap().1.next();

			let mut string = PauliString::<N>::id();
			for (i, l) in letters {
				string.set(i, l);
			}
			string
		};

		assert!(push_str.len() == n);

		for exp in exponentials.iter_mut() {
			exp.push_pi_over_4(false, &push_str);
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
		let single_qubit_indexes = get_remove_indexes(&exponentials, |p| p.len() == 1);
		for i in single_qubit_indexes.into_iter() {
			assert!(exponentials.get(i).unwrap().len() == 1);
			circuit.push(exponentials.remove(i));
			#[cfg(feature = "return_ordered")]
			ordered.push(clone.remove(i));
		}
	}

	#[cfg(feature = "return_ordered")]
	assert!(clone.is_empty());

	let clifford_circuit: Vec<PauliExp<N, CliffordPauliAngle>> =
		clifford_tableau.into_iter().rev().collect();

	#[cfg(feature = "return_ordered")]
	return (circuit, clifford_circuit, ordered);

	#[cfg(not(feature = "return_ordered"))]
	(circuit, clifford_circuit)
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
			let (circuit, clifford) = synthesize(input, NonZeroEvenUsize::new(4).unwrap());

			#[cfg(feature = "return_ordered")]
			let (circuit, clifford, _) = synthesize(input, NonZeroEvenUsize::new(4).unwrap());

			for exp in circuit {
				assert!(exp.len() == 1 || exp.len() == 4);
			}

			for exp in clifford {
				assert!(exp.len() == 1 || exp.len() == 4);
			}
		}
	}
}
