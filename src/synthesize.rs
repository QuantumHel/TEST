use crate::{
	connectivity::Connectivity,
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
	connectivity: Option<&Connectivity>,
) -> SynthesizeResult<N> {
	#[cfg(feature = "return_ordered")]
	let mut clone: Vec<PauliExp<N, FreePauliAngle>> = exponentials.clone();
	#[cfg(feature = "return_ordered")]
	let mut ordered: Vec<PauliExp<N, FreePauliAngle>> = Vec::new();

	let n = gate_size.as_value();
	let mut circuit: Vec<PauliExp<N, FreePauliAngle>> = Vec::new();
	let mut clifford_part: Vec<PauliExp<N, CliffordPauliAngle>> = Vec::new();

	// move single (an no) qubit gates to circuit
	let remove_indexes = get_remove_indexes(&exponentials, |p| p.len() <= 1);
	for i in remove_indexes.into_iter() {
		assert!(exponentials.get(i).unwrap().len() == 1);
		circuit.push(exponentials.remove(i));
		#[cfg(feature = "return_ordered")]
		ordered.push(clone.remove(i));
	}

	// new loop
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
				angle: CliffordPauliAngle::NeqPiOver4,
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
	return (circuit, clifford_part, ordered);

	#[cfg(not(feature = "return_ordered"))]
	(circuit, clifford_part)
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
