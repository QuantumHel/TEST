use nanorand::{Rng, WyRand};
use test_transpiler::{
	misc::NonZeroEvenUsize,
	pauli::{PauliAngle, PauliExp, PauliMatrix, PauliString},
	synthesize::synthesize,
};

fn random_exp(qubits: usize, rng: &mut WyRand) -> PauliExp {
	let n_letters = rng.generate_range(1_usize..=qubits);
	let mut selection: Vec<u32> = (0..(qubits as u32)).collect();
	rng.shuffle(&mut selection);
	let mut string = PauliString::default();
	for qubit in selection.into_iter().take(n_letters) {
		let pauli = match rng.generate_range(0_usize..3_usize) {
			0 => PauliMatrix::X,
			1 => PauliMatrix::Y,
			_ => PauliMatrix::Z,
		};
		string.letters.insert(qubit, pauli);
	}

	PauliExp {
		string,
		angle: PauliAngle::Free(rng.generate()),
	}
}

const N_EXPS: usize = 30;
const N_QUBITS: usize = 30;
const GATE_SIZE: usize = 8;
const N_ROUNDS: usize = 4;

fn main() {
	for i in 0..N_ROUNDS {
		let mut rng = WyRand::new();
		let input: Vec<PauliExp> = (0..N_EXPS)
			.map(move |_| random_exp(N_QUBITS, &mut rng))
			.collect();

		let (circuit, clifford) = synthesize(input, NonZeroEvenUsize::new(GATE_SIZE).unwrap());

		for exp in circuit.iter() {
			assert!(exp.len() == 1 || exp.len() == GATE_SIZE);
		}

		for exp in clifford.iter() {
			assert!(exp.angle.is_clifford())
		}
		println!("Round: {}", i + 1);
		println!("N qubits: {}", N_QUBITS);
		println!("N start exponentials: {}", N_EXPS);
		let n_qubit_gates = circuit
			.iter()
			.filter(|p| p.len() > 1)
			.collect::<Vec<_>>()
			.len();
		println!(
			"Output circuit has {}x {}-qubit gates",
			n_qubit_gates, GATE_SIZE
		);
		println!();
	}
}
