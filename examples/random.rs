use nanorand::{Rng, WyRand};
use test_transpiler::{
	misc::NonZeroEvenUsize,
	pauli::{FreePauliAngle, PauliExp, PauliLetter, PauliString},
	synthesize::synthesize,
};

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

const N_EXPS: usize = 175;
const N_QUBITS: usize = 7;
const GATE_SIZE: usize = 2;
const N_ROUNDS: usize = 100;

fn main() {
	let mut summ = 0;
	for i in 0..N_ROUNDS {
		let mut rng = WyRand::new();
		let input: Vec<PauliExp<N_QUBITS, FreePauliAngle>> = (0..N_EXPS)
			.map(move |_| random_exp::<N_QUBITS>(&mut rng))
			.collect();

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

		summ += n_qubit_gates + n_clifford;
		println!();
	}

	print!("Average: {}", summ as f64 / N_ROUNDS as f64);
}
