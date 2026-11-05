pub mod gateset;
pub mod graph;
pub mod mapping;
pub mod resolve_in_pairs;
pub mod synthesis;

use core::{
	Compiler,
	connectivity::{Connectivity, TwoQubitEdge},
	qubit_map::QubitMap,
};

use ::graph::GraphExt;
use circuit::Circuit;

use crate::CliffordTableau;

use self::gateset::CNotHSV;

use self::mapping::construct_tree;
use self::mapping::find_mapping;
use self::mapping::print_grid;
use self::synthesis::tableau_to_circuit_pairs;
// use self::mapping::implement_mapping;

pub struct SPCC;

impl Compiler<CliffordTableau, (Circuit<CNotHSV>, QubitMap), Connectivity<TwoQubitEdge>> for SPCC {
	fn compile(
		&self,
		input: CliffordTableau,
		device: &Connectivity<TwoQubitEdge>,
	) -> (Circuit<CNotHSV>, QubitMap) {
		let num_logical_qubits = input.size();
		let nr_physical_qubits = device.node_count();
		let (root, device_tree) =
			construct_tree(device, num_logical_qubits).expect("failed to construct SPCC tree");
		//		print_grid((4, 4), &device_tree);
		//		let optimized_mapping = find_mapping(&input, root, &device_tree);
		//		println!("Optimized mapping: {:?}", optimized_mapping);
		//		let mut mapped_tableau: CliffordTableau = implement_mapping(&input, &optimized_mapping);
		let (circ, output_mapping) = tableau_to_circuit_pairs(&input, &device_tree, true);
		//		println!("Output mapping: {:?}", output_mapping);
		(circ, output_mapping)
	}
}

#[cfg(test)]
mod tests {
	use circuit::Circuit;
	use rand::SeedableRng;
	use rand_chacha::ChaCha8Rng;

	use crate::CliffordTableau;

	use super::{SPCC, gateset::CNotHSV};
	use core::{Compiler, connectivity::TwoQubitEdge};
	use std::time;

	#[test]
	#[ignore = "not yet implemented"]
	fn random_spcc_test() {
		const ROUNDS: usize = 100;
		const QUBITS: usize = 64;
		const GATES: usize = QUBITS * 1000;
		let compiler = SPCC;
		let connectivity = TwoQubitEdge::square_lattice(QUBITS);

		let mut rng = ChaCha8Rng::seed_from_u64(67);
		let mut cnot_count_sum = 0;
		let mut time_sum = 0.0;
		for round in 0..ROUNDS {
			println!("\nRound {round} of {ROUNDS}");
			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			let mut tableau = CliffordTableau::id();
			for gate in input.iter() {
				tableau.merge_clifford(gate);
			}
			//			print!("Round {round} input tableau:\n{:?}\n", tableau);
			//			print!("Round {round} input tableau:\n{:?}\n", tableau.size());
			let start = time::Instant::now();
			let (circuit, ouput_mapping) = compiler.compile(tableau.clone(), &connectivity);
			let elapsed = start.elapsed();
			time_sum += elapsed.as_millis() as f64;
			let mut result_tableau = CliffordTableau::id();
			for gate in circuit.iter() {
				if let CNotHSV::CNot(cnot) = gate
					&& !connectivity
						.neighbors(*cnot.target())
						.contains(cnot.control())
				{
					panic!(
						"Got CNot({}, {}) that is not supported by connectivity",
						cnot.control(),
						cnot.target()
					);
				}
				result_tableau.merge_clifford(gate);
			}
			result_tableau.merge_qubit_map(&ouput_mapping);
			assert_eq!(result_tableau, tableau);
			cnot_count_sum += circuit.fileter_len(|g| matches!(g, CNotHSV::CNot(_)));
			println!(
				"Round {round} went from {} to {} CNOT gates. Average CNOT count so far: {}",
				input.fileter_len(|g| { matches!(g, CNotHSV::CNot(_)) }),
				circuit.fileter_len(|g| { matches!(g, CNotHSV::CNot(_)) }),
				(cnot_count_sum as f64 / (round + 1) as f64).round()
			);
			println!(
				"Round {round} took {} ms, average time so far: {} ms",
				elapsed.as_millis(),
				(time_sum / (round + 1) as f64).round()
			);
		}
	}
}
