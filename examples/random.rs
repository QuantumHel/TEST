use bitvec::vec::BitVec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use test_transpiler::{
	misc::NonZeroEvenUsize,
	pauli::{FreePauliAngle, PauliAngle, PauliExp, PauliLetter, PauliString},
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

const N_EXPS: usize = 5;
const N_QUBITS: usize = 5;
const GATE_SIZE: usize = 2;
const N_ROUNDS: usize = 4;

fn main() {
	let mut rng = ChaCha8Rng::seed_from_u64(1);

	let mut summ = 0;
	for i in 0..N_ROUNDS {
		let mut input: Vec<PauliExp<N_QUBITS, FreePauliAngle>> = Vec::new();
		for _ in 0..N_EXPS {
			input.push(random_exp::<N_QUBITS, _>(&mut rng));
		}

		#[cfg(not(feature = "return_ordered"))]
		let (circuit, clifford) = synthesize(input, NonZeroEvenUsize::new(GATE_SIZE).unwrap());

		#[cfg(feature = "return_ordered")]
		let (circuit, clifford, _) = synthesize(input, NonZeroEvenUsize::new(GATE_SIZE).unwrap());

		for exp in circuit.iter() {
			assert!(exp.len() == 1 || exp.len() == GATE_SIZE);
		}

		for exp in clifford.iter() {
			assert!(exp.len() == 1 || exp.len() == GATE_SIZE);
		}

		println!("Round: {}", i + 1);
		println!("N qubits: {N_QUBITS}");
		println!("N start exponentials: {N_EXPS}");
		let n_qubit_gates = circuit
			.iter()
			.filter(|p| p.len() > 1)
			.collect::<Vec<_>>()
			.len();

		let n_clifford = clifford.len();
		println!(
			"Output circuit has {n_qubit_gates}x {GATE_SIZE}-qubit gates and {n_clifford} cliffords"
		);
		println!("circuit depth {}", gate_dept(&circuit));

		summ += n_qubit_gates + n_clifford;
		println!();
	}

	print!("Average: {}", summ as f64 / N_ROUNDS as f64);
}
