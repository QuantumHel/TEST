use crate::{ParityMatrix, TwoQubitEdge};
use circuit::{Circuit, gates::CNot};
use core::{Compiler, QubitMapping, QubitMappingBuilder, connectivity::Connectivity};
use graph::prelude::*;

/// An implementation of the PermRowCol algorithm from
/// https://arxiv.org/abs/2205.00724v4
pub struct PermRowCol;

impl Compiler<ParityMatrix, (Circuit<CNot>, QubitMapping), Connectivity<TwoQubitEdge>>
	for PermRowCol
{
	fn compile(
		&self,
		parity_matrix: ParityMatrix,
		device: &Connectivity<TwoQubitEdge>,
	) -> (Circuit<CNot>, QubitMapping) {
		// Take the transpose in order to get output qubit mapping.
		// Without transpose it would be the input qubit mapping
		let mut parity_matrix = parity_matrix.transpose();
		let mut graph = device.full_subgraph();

		let mut available_qubits = vec![true; graph.node_storage_size()];
		let mut circuit: Circuit<CNot> = Circuit::new();
		let mut qubit_mapping = QubitMappingBuilder::default();
		while graph.iter_nodes().count() > 1 {
			let candidates = graph.non_cutting_nodes();
			let chosen_row = choose_row(&candidates, &parity_matrix);
			let chosen_column = choose_column(
				chosen_row,
				available_qubits
					.iter()
					.enumerate()
					.filter_map(|(i, &v)| v.then_some(i))
					.collect::<Vec<_>>()
					.as_slice(),
				&parity_matrix,
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
			qubit_mapping.map_qubit(chosen_row, chosen_column);
		}

		let free_column = available_qubits
			.iter()
			.enumerate()
			.find_map(|(i, v)| v.then_some(i))
			.unwrap();
		let free_row = graph.enumerate_nodes().next().unwrap().0;
		qubit_mapping.map_qubit(free_row, free_column);

		// we dont reverse the circuit as we used the transpose paritymatrix
		(circuit, qubit_mapping.finish().unwrap())
	}
}

fn choose_row(options: &[usize], parity_matrix: &ParityMatrix) -> usize {
	options
		.iter()
		.map(|&i| (i, parity_matrix.get_row(i).count_ones()))
		.min_by(|a, b| a.1.cmp(&b.1))
		.unwrap()
		.0
}

fn choose_column(selected_row: usize, options: &[usize], parity_matrix: &ParityMatrix) -> usize {
	let parity_matrix = parity_matrix.transpose();
	// for columns with an 1 in the pivor row, pick the one with the least 1s
	options
		.iter()
		.filter_map(|&i| {
			let row = parity_matrix.get_row(i);
			row.get(selected_row).then(|| (i, row.count_ones()))
		})
		.min_by(|a, b| a.1.cmp(&b.1))
		.unwrap()
		.0
}

#[cfg(test)]
mod tests {
	use core::Compiler;
	use core::connectivity::Connectivity;

	use circuit::{RandomGate, gates::CNot};
	use rand::prelude::*;
	use rand_chacha::ChaCha8Rng;

	use crate::{ParityMatrix, TwoQubitEdge, algorithm::PermRowCol};

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

			let compiler = PermRowCol;
			let (circuit, qubit_mapping) = compiler.compile(parity_matrix.clone(), &g);

			parity_matrix.insert_qubit_mapping(&qubit_mapping.as_reversed());
			for cnot in circuit.iter().rev() {
				assert!(g.neighbors(*cnot.control()).contains(cnot.target()));
				parity_matrix.insert_cnot(*cnot);
			}

			assert!(parity_matrix.is_identity());
		}
	}
}
