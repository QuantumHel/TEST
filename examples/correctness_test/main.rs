use nanorand::{Rng, WyRand};
use test_transpiler::{
	misc::NonZeroEvenUsize,
	pauli::{FreePauliAngle, PauliExp, PauliLetter, PauliString, as_exp_file},
	synthesize::synthesize,
};

const N_EXPS: usize = 30;
const N_QUBITS: usize = 12;
const GATE_SIZE: usize = 6;
const N_ROUNDS: usize = 10;

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
		angle: FreePauliAngle::MultipleOfPi(rng.generate()),
	}
}

fn main() {
	for i in 0..N_ROUNDS {
		let mut rng = WyRand::new();
		let input: Vec<PauliExp<N_QUBITS, FreePauliAngle>> = (0..N_EXPS)
			.map(move |_| random_exp::<N_QUBITS>(&mut rng))
			.collect();

		let (mut circuit, clifford, order) =
			synthesize(input, NonZeroEvenUsize::new(GATE_SIZE).unwrap());

		let mut clifford: Vec<PauliExp<{ N_QUBITS }, FreePauliAngle>> = clifford
			.into_iter()
			.map(|p| PauliExp {
				string: p.string,
				angle: FreePauliAngle::Clifford(p.angle),
			})
			.rev()
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
