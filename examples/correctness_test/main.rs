use rand::prelude::*;
use test_transpiler::{
	clifford_tableau::CliffordTableau,
	misc::NonZeroEvenUsize,
	pauli::{CliffordPauliAngle, FreePauliAngle, PauliExp, PauliLetter, PauliString, as_exp_file},
	synthesize::synthesize,
};

const N_EXPS: usize = 30;
const N_QUBITS: usize = 12;
const GATE_SIZE: usize = 6;
const N_ROUNDS: usize = 10;
const USE_TABLEAU: bool = false;

fn random_exp<const N: usize, R: Rng>(rng: &mut R) -> PauliExp<N, FreePauliAngle> {
	let n_letters = (1_usize..=N).choose(rng).unwrap();
	let mut selection: Vec<usize> = (0..N).collect();
	selection.shuffle(rng);
	let mut string = PauliString::default();
	for qubit in selection.into_iter().take(n_letters) {
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

fn main() {
	for i in 0..N_ROUNDS {
		let mut rng = rand::rng();
		let input: Vec<PauliExp<N_QUBITS, FreePauliAngle>> = (0..N_EXPS)
			.map(move |_| random_exp::<N_QUBITS, _>(&mut rng))
			.collect();

		let (mut circuit, clifford, order) =
			synthesize(input, NonZeroEvenUsize::new(GATE_SIZE).unwrap(), None);

		let clifford: Vec<PauliExp<{ N_QUBITS }, CliffordPauliAngle>> = if USE_TABLEAU {
			let mut tableau: CliffordTableau<{ N_QUBITS }> = CliffordTableau::id();
			for op in clifford.into_iter() {
				let sign = match op.angle {
					CliffordPauliAngle::NeqPiOver4 => true,
					CliffordPauliAngle::PiOver4 => false,
				};
				tableau.merge_pi_over_4_pauli(sign, &op.string);
			}

			tableau.decompose(NonZeroEvenUsize::new(GATE_SIZE).unwrap(), None)
		} else {
			clifford
		};

		let mut clifford = clifford
			.into_iter()
			.map(PauliExp::<{ N_QUBITS }, FreePauliAngle>::from)
			.collect();

		circuit.append(&mut clifford);

		for exp in circuit.iter() {
			assert!(exp.len() == 1 || exp.len() == GATE_SIZE);
		}

		as_exp_file(
			&format!("./examples/correctness_test/circuit{i}.exp"),
			&circuit,
		);
		as_exp_file(
			&format!("./examples/correctness_test/circuit{i}.exp.order"),
			&order,
		);
	}
}
