use crate::{
	ParityMatrix,
	choose::{ChooseColumn, ChooseQubit},
};
use circuit::{Circuit, gates::CNot};
use core::{
	Compiler,
	connectivity::{Connectivity, TwoQubitEdge},
	qubit_map::{PartialQubitMap, QubitMap},
};
use graph::prelude::*;

/// An implementation of the PermRowCol algorithm from
/// https://arxiv.org/abs/2205.00724v4
pub struct PermRowCol<R: ChooseQubit, C: ChooseColumn> {
	pub choose_row: R,
	pub choose_column: C,
}

impl<R: ChooseQubit, C: ChooseColumn>
	Compiler<ParityMatrix, (Circuit<CNot>, QubitMap), Connectivity<TwoQubitEdge>>
	for PermRowCol<R, C>
{
	fn compile(
		&self,
		parity_matrix: ParityMatrix,
		device: &Connectivity<TwoQubitEdge>,
	) -> (Circuit<CNot>, QubitMap) {
		// Take the transpose in order to get output qubit mapping.
		// Without transpose it would be the input qubit mapping
		let mut parity_matrix = parity_matrix.transpose();
		let mut circuit: Circuit<CNot> = Circuit::new();
		let mut graph = device.full_subgraph();

		let mut available_qubits = vec![true; graph.node_storage_size()];
		let mut qubit_mapping = PartialQubitMap::default();
		loop {
			let non_cutting = graph.non_cutting_nodes();
			if non_cutting.is_empty() {
				break;
			}

			let chosen_row = self
				.choose_row
				.choose_qubit(&non_cutting, &parity_matrix, device);
			let chosen_column = self.choose_column.choose_column(
				chosen_row,
				available_qubits
					.iter()
					.enumerate()
					.filter_map(|(i, &v)| v.then_some(i))
					.collect::<Vec<_>>()
					.as_slice(),
				&parity_matrix,
				device,
			);

			// Since the parity matrix is transposed, we need to reverse the
			// cnots
			for gate in parity_matrix.eliminate_column(chosen_row, chosen_column, &graph) {
				circuit.push(gate.reverse());
			}
			for gate in parity_matrix.eliminate_row(chosen_row, chosen_column, &graph) {
				circuit.push(gate.reverse());
			}

			available_qubits[chosen_column] = false;
			graph.remove_node(chosen_row);
			qubit_mapping.map_qubit(chosen_row, chosen_column).unwrap();
		}

		// we dont reverse the circuit as we used the transpose paritymatrix
		(circuit, qubit_mapping.try_into().unwrap())
	}
}

#[cfg(test)]
mod tests {
	use core::connectivity::Connectivity;
	use core::{Compiler, connectivity::TwoQubitEdge};

	use circuit::{RandomGate, gates::CNot};
	use rand::prelude::*;
	use rand_chacha::ChaCha8Rng;

	use crate::choose::OriginalPermRowColChoice;
	use crate::{ParityMatrix, decompose::PermRowCol};

	#[test]
	fn perm_row_col_random_test() {
		const TEST_COUNT: usize = 100;
		const QUBIT_COUNT: usize = 100;
		const CNOT_COUNT: usize = QUBIT_COUNT * 100;
		let mut rng = ChaCha8Rng::seed_from_u64(2);

		let g: Connectivity<TwoQubitEdge> = TwoQubitEdge::square_lattice(QUBIT_COUNT);

		for _ in 0..TEST_COUNT {
			let cnots: Vec<_> = (0..(CNOT_COUNT))
				.map(|_| CNot::random(QUBIT_COUNT, &mut rng))
				.collect();

			let mut parity_matrix = ParityMatrix::default();
			for cnot in cnots {
				parity_matrix.insert_cnot(cnot);
			}

			let compiler = PermRowCol {
				choose_row: OriginalPermRowColChoice,
				choose_column: OriginalPermRowColChoice,
			};
			let (circuit, qubit_mapping) = compiler.compile(parity_matrix.clone(), &g);

			parity_matrix.insert_qubit_mapping(&qubit_mapping.as_reversed());
			for cnot in circuit.iter().rev() {
				assert!(g.neighbors(*cnot.control()).contains(cnot.target()));
				parity_matrix.insert_cnot(*cnot);
			}

			assert!(parity_matrix.is_identity());
			println!("Converted {CNOT_COUNT} cnots to {} cnots", circuit.len());
		}
	}
}
