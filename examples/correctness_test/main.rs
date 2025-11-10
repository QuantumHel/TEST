use rand::prelude::*;
use test_transpiler::{
	clifford_tableau::CliffordTableau,
	connectivity::Connectivity,
	misc::NonZeroEvenUsize,
	pauli::{CliffordPauliAngle, PauliAngle, PauliExp, PauliLetter, PauliString},
	synthesize::synthesize,
};

const N_EXPS: usize = 30;
const N_QUBITS: usize = 13;
const GATE_SIZE: usize = 4;
const N_ROUNDS: usize = 10;
const USE_TABLEAU: bool = true;

fn random_exp<R: Rng>(max_size: usize, rng: &mut R) -> PauliExp<PauliAngle> {
	let n_letters = (1_usize..=max_size).choose(rng).unwrap();
	let mut selection: Vec<usize> = (0..max_size).collect();
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

	let angle = if rng.random::<bool>() {
		match rng.random_range(0..4) {
			0 => PauliAngle::Clifford(CliffordPauliAngle::NegPiOver2),
			1 => PauliAngle::Clifford(CliffordPauliAngle::PiOver2),
			2 => PauliAngle::Clifford(CliffordPauliAngle::NegPiOver4),
			3 => PauliAngle::Clifford(CliffordPauliAngle::PiOver4),
			_ => unreachable!(),
		}
	} else {
		PauliAngle::MultipleOfPi(rng.random())
	};

	PauliExp { string, angle }
}

fn main() {
	let connectivity = Some(Connectivity::create_line(4, 4));

	for i in 0..N_ROUNDS {
		let mut rng = rand::rng();
		let input: Vec<PauliExp<PauliAngle>> = (0..N_EXPS)
			.map(move |_| random_exp::<_>(N_QUBITS, &mut rng))
			.collect();

		let (mut circuit, clifford, order) = synthesize(
			input,
			NonZeroEvenUsize::new(GATE_SIZE).unwrap(),
			connectivity.as_ref(),
		);

		let clifford: Vec<PauliExp<CliffordPauliAngle>> = if USE_TABLEAU {
			let mut tableau: CliffordTableau = CliffordTableau::id();
			for op in clifford.into_iter() {
				tableau.merge_clifford(op);
			}

			tableau.decompose(
				NonZeroEvenUsize::new(GATE_SIZE).unwrap(),
				connectivity.as_ref(),
			)
		} else {
			clifford
		};

		let mut clifford = clifford
			.into_iter()
			.map(PauliExp::<PauliAngle>::from)
			.collect();

		circuit.append(&mut clifford);

		for exp in circuit.iter() {
			assert!(exp.len() == 1 || exp.len() == GATE_SIZE);
			if let Some(ref connectivity) = connectivity {
				assert!(connectivity.supports_operation_on(&exp.string.targets()))
			}
		}

		PauliExp::write_exp_file(
			&circuit,
			&format!("./examples/correctness_test/circuit{i}.exp"),
		);
		PauliExp::write_exp_file(
			&order,
			&format!("./examples/correctness_test/circuit{i}.exp.order"),
		);
	}
}
