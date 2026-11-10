pub mod ct_fast;
pub mod gateset;
pub mod graph;
pub mod mapping;
pub mod print_tableau;
pub mod resolve_in_pairs;
pub mod synthesis;

use core::{
	Compiler,
	connectivity::{Connectivity, TwoQubitEdge},
	qubit_map::{PartialQubitMap, QubitMap},
};

use self::mapping::{construct_tree, find_mapping};
use self::synthesis::tableau_to_circuit_pairs;
use self::{ct_fast::CTFast, gateset::CNotHSV};
use circuit::Circuit;

pub struct SPCC;

impl Compiler<CTFast, (Circuit<CNotHSV>, PartialQubitMap, QubitMap), Connectivity<TwoQubitEdge>>
	for SPCC
{
	fn compile(
		&self,
		tableau: CTFast,
		device: &Connectivity<TwoQubitEdge>,
	) -> (Circuit<CNotHSV>, PartialQubitMap, QubitMap) {
		let num_logical_qubits = tableau.size();
		//		let num_physical_qubits = device.nodes().len();
		let (root_node, device_tree) =
			construct_tree(device, num_logical_qubits).expect("failed to construct SPCC tree");
		//		print_grid((8, 8), &device_tree);
		let initial_mapping = find_mapping(&tableau, root_node, &device_tree);
		println!("Initial mapping: {:?}", initial_mapping);
		//		let mut mapped_tableau = tableau.clone();
		//		mapped_tableau.merge_qubit_map(&initial_mapping);
		let (circ, output_mapping) = tableau_to_circuit_pairs(&tableau, &device_tree, true);
		(circ, initial_mapping, output_mapping)
	}
}

#[cfg(test)]
mod tests {
	use circuit::Circuit;
	use rand::SeedableRng;
	use rand_chacha::ChaCha8Rng;

	use crate::CliffordTableau;

	use super::{SPCC, ct_fast::CTFast, gateset::CNotHSV};
	use core::{Compiler, connectivity::TwoQubitEdge};
	use std::time;

	#[test]
	#[ignore = "not yet implemented"]
	fn random_spcc_test() {
		const ROUNDS: usize = 30;
		const LOGICAL_QUBITS: usize = 121;
		const PHYSICAL_QUBITS: usize = 121;
		const GATES: usize = LOGICAL_QUBITS * 100;
		let compiler = SPCC;
		let connectivity = TwoQubitEdge::square_lattice(PHYSICAL_QUBITS);

		let mut rng = ChaCha8Rng::seed_from_u64(67);
		let mut cnot_count_sum = 0;
		let mut time_sum = 0.0;
		for round in 0..ROUNDS {
			println!("\nRound {round} of {ROUNDS}");
			let input: Circuit<CNotHSV> = Circuit::random(GATES, LOGICAL_QUBITS, &mut rng);
			let mut tableau_fast = CTFast::new(LOGICAL_QUBITS);
			for gate in input.iter() {
				tableau_fast.merge_clifford(gate);
			}
			let input_tableau = tableau_fast.clone();
			let start = time::Instant::now();
			let (circuit, _initial_mapping, output_mapping) =
				compiler.compile(tableau_fast, &connectivity);
			let elapsed = start.elapsed();
			time_sum += elapsed.as_millis() as f64;
			let mut result_tableau = CTFast::new(PHYSICAL_QUBITS); // Identity tableau
			for gate in circuit.iter() {
				// Circuit
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
			result_tableau.merge_qubit_map(&output_mapping); // Output mapping
			assert_eq!(result_tableau, input_tableau);
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
