use circuit::Circuit;
use circuit::gates::CNot;
use core::{Compiler, TargetQubits};
use parity_matrix::{Basis, CNotRzXYH, GraySynth, ParityMatrix, TPar, decompose::PatelMarkovHayes};
use std::num::NonZero;
use test_experiment::source::OnlineOpenQasm2Source;

struct PatelMarkovHayesVariations;

impl Compiler<ParityMatrix, Circuit<CNot>, ()> for PatelMarkovHayesVariations {
	fn compile(&self, input: ParityMatrix, _: &()) -> Circuit<CNot> {
		let elim = PatelMarkovHayes::new_with_equation(|n| {
			NonZero::new(((n as f64).log2() / 2.).ceil().max(1.) as usize).unwrap()
		});
		let res = elim.compile(input.clone(), &());
		let res_t = Circuit {
			gates: elim
				.compile(input.transpose(), &())
				.into_iter()
				.map(|c| c.reverse())
				.rev()
				.collect(),
		};

		let mut other_basis = match input.basis() {
			Basis::Hadamard => ParityMatrix::standard_basis(),
			Basis::Standard => ParityMatrix::hadamard_basis(),
		};

		for gate in res.iter() {
			other_basis.insert_cnot(*gate);
		}

		let o_res = elim.compile(other_basis.clone(), &());
		let o_res_t = Circuit {
			gates: elim
				.compile(other_basis.transpose(), &())
				.into_iter()
				.map(|c| c.reverse())
				.rev()
				.collect(),
		};

		let results = vec![res, res_t, o_res, o_res_t];
		results.into_iter().min_by_key(Circuit::len).unwrap()
	}
}

fn is_multiqubit<T: TargetQubits>(gate: &T) -> bool {
	gate.target_qubits().count() > 1
}

fn main() {
	let targets = vec![
		"grover_5",
		"mod5_4",
		"vbe_adder_3",
		"csla_mux_3",
		"csum_mux_9",
		"qcla_com_7",
		"qcla_mod_7",
		"qcla_adder_10",
		"adder_8",
		"rc_adder_6",
		"mod_red_21",
		"mod_mult_55",
		"mod_adder_1024",
		"barenco_tof_10",
		"barenco_tof_3",
		"barenco_tof_4",
		"barenco_tof_5",
	];
	let targets: Vec<_> = targets
		.into_iter()
		.map(|name| {
			(
				String::from(name),
				format!(
					"https://raw.githubusercontent.com/meamy/feynman/refs/heads/master/benchmarks/qasm/{}.qasm",
					name
				),
			)
		})
		.collect();
	let frontend = CNotRzXYH::open_qasm2_frontend();
	let circuit_source = OnlineOpenQasm2Source::new(frontend, targets);

	let compiler = TPar::new(GraySynth, PatelMarkovHayesVariations);

	println!("name;start_count;start_depth;end_count;end_depth");
	for (name, circuit) in circuit_source {
		let start_count = circuit.gates.iter().filter(|&g| is_multiqubit(g)).count();
		let start_depth = circuit.filtered_depth(is_multiqubit);
		let compiled_circuit = compiler.compile(circuit, &());
		let end_count = compiled_circuit
			.gates
			.iter()
			.filter(|&g| is_multiqubit(g))
			.count();
		let end_depth = compiled_circuit.filtered_depth(is_multiqubit);

		println!("{name};{start_count};{start_depth};{end_count};{end_depth}");
	}
}
