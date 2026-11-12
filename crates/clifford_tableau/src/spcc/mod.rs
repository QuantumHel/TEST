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
	qubit_map::QubitMap,
};

use self::gateset::CNotHSV;
use self::mapping::construct_tree;
use self::synthesis::tableau_to_circuit_pairs;
use crate::CliffordTableau;
use circuit::Circuit;

pub struct SPCC;

impl Compiler<CliffordTableau, (Circuit<CNotHSV>, QubitMap), Connectivity<TwoQubitEdge>> for SPCC {
	fn compile(
		&self,
		tableau: CliffordTableau,
		device: &Connectivity<TwoQubitEdge>,
	) -> (Circuit<CNotHSV>, QubitMap) {
		let tableau = ct_fast::CTFast::from_clifford_tableau(&tableau);
		let num_logical_qubits = tableau.size();
		let (_root_node, device_tree) =
			construct_tree(device, num_logical_qubits).expect("failed to construct SPCC tree");
		let (circ, input_mapping) = tableau_to_circuit_pairs(&tableau, &device_tree, true);
		(circ, input_mapping)
	}
}

#[cfg(test)]
mod tests {
	use crate::CliffordTableau;
	use circuit::Circuit;
	use rand::SeedableRng;
	use rand_chacha::ChaCha8Rng;

	use super::{SPCC, gateset::CNotHSV};
	use core::{Compiler, connectivity::TwoQubitEdge};

	#[test]
	fn random_spcc_test() {
		const ROUNDS: usize = 100;
		const LOGICAL_QUBITS: usize = 64;
		const PHYSICAL_QUBITS: usize = 64;
		const GATES: usize = LOGICAL_QUBITS * 1000;
		let compiler = SPCC;
		let connectivity = TwoQubitEdge::square_lattice(PHYSICAL_QUBITS);

		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for round in 0..ROUNDS {
			println!("\nRound {round} of {ROUNDS}");
			let input: Circuit<CNotHSV> = Circuit::random(GATES, LOGICAL_QUBITS, &mut rng);
			let mut tableau = CliffordTableau::id();
			for gate in input.iter() {
				tableau.merge_clifford(gate);
			}

			let (circuit, initial_mapping) = compiler.compile(tableau.clone(), &connectivity);

			let mut result_tableau = CliffordTableau::id(); // Identity tableau
			for gate in circuit.iter() {
				// Apply circuit
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
			result_tableau.merge_qubit_map(&initial_mapping);
			assert_eq!(&result_tableau, &tableau);
		}
	}
}
