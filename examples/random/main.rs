use bitvec::vec::BitVec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::io::Write;
use std::thread;
use std::{fs::File, sync::Arc};
use test_transpiler::pauli::Negate;
use test_transpiler::{
	clifford_tableau::CliffordTableau,
	connectivity::Connectivity,
	misc::NonZeroEvenUsize,
	pauli::{CliffordPauliAngle, PauliAngle, PauliExp, PauliLetter, PauliString},
	synthesize::synthesize,
};

fn random_exp<const N: usize, R: Rng>(max_exp_size: usize, rng: &mut R) -> PauliExp<N, PauliAngle> {
	let n_letters = (1_usize..=max_exp_size).choose(rng);
	let mut selection: Vec<usize> = (0..max_exp_size).collect();
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
		angle: PauliAngle::MultipleOfPi(rng.random()),
	}
}

/// How many "layers" we need
fn gate_dept<const N: usize, P: Negate>(circuit: &[PauliExp<N, P>]) -> usize {
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

fn multi_gate_count<const N: usize, A: Negate>(gates: &[PauliExp<N, A>]) -> usize {
	gates
		.iter()
		.filter(|p| p.len() > 1)
		.collect::<Vec<_>>()
		.len()
}

const N_QUBITS: usize = 20;

#[derive(Clone, Copy, Debug)]
struct Parameters {
	n_exps: usize,
	max_exp_size: usize,
	gate_size: usize,
	n_rounds: usize,
	use_tableau: bool,
}

fn run_experiment(parameters: Parameters, connectivity: Arc<Option<Connectivity>>) {
	let file_name = format!(
		"./examples/random/experiment_{}_{}_{}_{}_{}_{}_{}.out",
		parameters.n_exps,
		N_QUBITS,
		parameters.max_exp_size,
		parameters.gate_size,
		parameters.n_rounds,
		parameters.use_tableau,
		connectivity.is_some()
	);

	let mut file = File::options()
		.read(false)
		.write(true)
		.create_new(true)
		.open(&file_name)
		.expect("Failed to create file");

	let mut rng = ChaCha8Rng::seed_from_u64(2);

	let mut count_sum = 0;
	let mut depth_sum = 0;

	for i in 0..parameters.n_rounds {
		let mut original_exponentials: Vec<PauliExp<N_QUBITS, PauliAngle>> = Vec::new();
		for _ in 0..parameters.n_exps {
			original_exponentials
				.push(random_exp::<N_QUBITS, _>(parameters.max_exp_size, &mut rng));
		}

		#[cfg(not(feature = "return_ordered"))]
		let (mut circuit, clifford) = synthesize(
			original_exponentials,
			NonZeroEvenUsize::new(parameters.gate_size).unwrap(),
			connectivity.as_ref().as_ref(),
		);
		#[cfg(feature = "return_ordered")]
		let (mut circuit, clifford, _) = synthesize(
			original_exponentials,
			NonZeroEvenUsize::new(parameters.gate_size).unwrap(),
			connectivity.as_ref().as_ref(),
		);

		let mut clifford: Vec<PauliExp<{ N_QUBITS }, PauliAngle>> = if parameters.use_tableau {
			let mut tableau = CliffordTableau::id();

			for op in clifford.iter() {
				let neg = match op.angle {
					CliffordPauliAngle::PiOver4 => false,
					CliffordPauliAngle::NegPiOver4 => true,
					_ => unreachable!(),
				};
				tableau.merge_pi_over_4_pauli(neg, &op.string);
			}

			let decomposition = tableau.decompose(
				NonZeroEvenUsize::new(parameters.gate_size).unwrap(),
				connectivity.as_ref().as_ref(),
			);

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
			assert!(exp.len() == 1 || exp.len() == parameters.gate_size);
		}

		let gate_count = multi_gate_count(&circuit);
		let gate_depth = gate_dept(&circuit);
		count_sum += gate_count;
		depth_sum += gate_depth;

		writeln!(
			file,
			"Round: {i}, gate count: {gate_count}, gate_depth: {gate_depth}"
		)
		.unwrap();
	}

	writeln!(file).unwrap();
	writeln!(
		file,
		"Average count: {}",
		count_sum as f64 / parameters.n_rounds as f64
	)
	.unwrap();
	writeln!(
		file,
		"Average depth: {}",
		depth_sum as f64 / parameters.n_rounds as f64
	)
	.unwrap();
}

fn main() {
	/*
	let parameters = (1..=10).flat_map(|i| {
		(1..=7).map(move |j| Parameters {
			n_exps: 100 * j,
			max_exp_size: N_QUBITS,
			gate_size: i * 2,
			n_rounds: 100,
			use_tableau: true,
		})
	});
	*/

	// 2, 4, 6, 8, 10, 12, 14, 16, 18, 20
	// 2 + 1 * 198 = 100
	// 4 + 3 * 32 = 100
	// 6 + 5 * 19 = 101
	// 8 + 7 * 14 = 106
	// 10 + 9 * 10 = 100
	// 12 + 11 * 8 = 100
	// 14 + 13 * 7 = 105
	// 16 + 15 * 6 = 106
	// 18 + 17 * 5 = 103
	// 20 + 19 * 5 = 115

	let parameters = (1..=10).map(|i| Parameters {
		n_exps: 700,
		max_exp_size: N_QUBITS,
		gate_size: i * 2,
		n_rounds: 100,
		use_tableau: true,
	});

	let connectivity: Arc<Option<Connectivity>> = Arc::new(None);

	let mut handles = Vec::new();
	for parameters in parameters {
		let connectivity = connectivity.clone();

		handles.push(thread::spawn(move || {
			run_experiment(parameters, connectivity);
		}));
	}

	for handle in handles {
		handle.join().unwrap();
	}
}
