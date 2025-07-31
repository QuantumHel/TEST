use bitvec::vec::BitVec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use test_transpiler::{
	clifford_tableau::CliffordTableau,
	misc::NonZeroEvenUsize,
	pauli::{CliffordPauliAngle, FreePauliAngle, PauliAngle, PauliExp, PauliLetter, PauliString},
	synthesize::synthesize,
};

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

/// How many "layers" we need
fn gate_dept<const N: usize, P: PauliAngle>(circuit: &[PauliExp<N, P>]) -> usize {
	let mut layers: Vec<BitVec> = Vec::new();

	for exp in circuit.iter() {
		if exp.len() < 2 {
			continue;
		}

		let mut stop: Option<usize> = None;
		'layer: for (i, layer) in layers.iter().enumerate().rev() {
			for (qubit, _) in exp.string.letters() {
				if layer[qubit] {
					stop = Some(i);
					break 'layer;
				}
			}
		}

		let layer: usize = match stop {
			Some(i) => {
				if (i + 1) == layers.len() {
					layers.push(BitVec::repeat(false, N));
				}
				i + 1
			}
			None => {
				if layers.is_empty() {
					layers.push(BitVec::repeat(false, N));
				}
				0
			}
		};

		for (qubit, _) in exp.string.letters() {
			layers[layer].set(qubit, true);
		}
	}

	layers.len()
}

fn multi_gate_count<const N: usize, A: PauliAngle>(gates: &[PauliExp<N, A>]) -> usize {
	gates
		.iter()
		.filter(|p| p.len() > 1)
		.collect::<Vec<_>>()
		.len()
}

const N_EXPS: usize = 700;
const N_QUBITS: usize = 27;
const GATE_SIZE: usize = 2;
const N_ROUNDS: usize = 100;
const USE_TABLEAU: bool = true;

fn main() {
	let mut rng = ChaCha8Rng::seed_from_u64(2);

	let mut count_sum = 0;
	let mut depth_sum = 0;

	println!("N_EXPS: {N_EXPS}");
	println!("N_QUBITS: {N_QUBITS}");
	println!("GATE_SIZE: {GATE_SIZE}");
	println!("N_ROUNDS: {N_ROUNDS}");

	for i in 0..N_ROUNDS {
		let mut original_exponentials: Vec<PauliExp<N_QUBITS, FreePauliAngle>> = Vec::new();
		for _ in 0..N_EXPS {
			original_exponentials.push(random_exp::<N_QUBITS, _>(&mut rng));
		}

		#[cfg(not(feature = "return_ordered"))]
		let (mut circuit, clifford) = synthesize(
			original_exponentials,
			NonZeroEvenUsize::new(GATE_SIZE).unwrap(),
		);

		#[cfg(feature = "return_ordered")]
		let (mut circuit, clifford, _) = synthesize(
			original_exponentials,
			NonZeroEvenUsize::new(GATE_SIZE).unwrap(),
		);

		let mut clifford: Vec<PauliExp<{ N_QUBITS }, FreePauliAngle>> = if USE_TABLEAU {
			let mut tableau = CliffordTableau::id();

			for op in clifford.iter() {
				let neg = match op.angle {
					CliffordPauliAngle::PiOver4 => false,
					CliffordPauliAngle::NeqPiOver4 => true,
				};
				tableau.merge_pi_over_4_pauli(neg, &op.string);
			}

			let decomposition = tableau.decompose(NonZeroEvenUsize::new(GATE_SIZE).unwrap());

			if multi_gate_count(&decomposition) < multi_gate_count(&clifford) {
				decomposition.into_iter().map(PauliExp::from).collect()
			} else {
				clifford.into_iter().map(PauliExp::from).collect()
			}
		} else {
			clifford.into_iter().map(PauliExp::from).collect()
		};

		circuit.append(&mut clifford);

		for exp in circuit.iter() {
			assert!(exp.len() == 1 || exp.len() == GATE_SIZE);
		}

		let gate_count = multi_gate_count(&circuit);

		let gate_depth = gate_dept(&circuit);
		count_sum += gate_count;
		depth_sum += gate_depth;

		println!("Round: {i}, gate count: {gate_count}, gate_depth: {gate_depth}");
	}

	println!();
	println!("Average count: {}", count_sum as f64 / N_ROUNDS as f64);
	println!("Average depth: {}", depth_sum as f64 / N_ROUNDS as f64);
}
