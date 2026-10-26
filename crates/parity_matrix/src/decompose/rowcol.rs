use circuit::{Circuit, gates::CNot};
use core::{connectivity::TwoQubitEdge, prelude::*};
use graph::prelude::*;

use crate::{ParityMatrix, choose::qubit::ChooseQubit};

/// An implementation of the rowcol algorithm described in
/// https://doi.org/10.1103/PhysRevResearch.5.013065
pub struct RowCol<T: ChooseQubit> {
	pub choose_qubit: T,
}

impl<T: ChooseQubit> Compiler<ParityMatrix, Circuit<CNot>, Connectivity<TwoQubitEdge>>
	for RowCol<T>
{
	fn compile(
		&self,
		mut matrix: ParityMatrix,
		device: &Connectivity<TwoQubitEdge>,
	) -> Circuit<CNot> {
		let mut circuit = Circuit::new();
		let mut graph = device.full_subgraph();

		loop {
			let non_cutting = graph.non_cutting_nodes();
			if non_cutting.is_empty() {
				break;
			}

			let i = self
				.choose_qubit
				.choose_qubit(&non_cutting, &matrix, device);
			circuit.append(&mut matrix.eliminate_column(i, i, &graph));
			circuit.append(&mut matrix.eliminate_row(i, i, &graph));

			graph.remove_node(i);
		}

		circuit.reverse();
		circuit
	}
}

#[cfg(test)]
mod tests {
	use crate::choose::qubit::FirstQubit;

	use super::*;
	use circuit::RandomGate;
	use rand::prelude::*;
	use rand_chacha::ChaCha8Rng;

	#[test]
	fn rowcol_random_test() {
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

			let compiler = RowCol {
				choose_qubit: FirstQubit,
			};
			let out = compiler.compile(parity_matrix.clone(), &g);

			for cnot in out.iter().rev() {
				assert!(g.neighbors(*cnot.control()).contains(cnot.target()));
				parity_matrix.insert_cnot(*cnot);
			}

			assert!(parity_matrix.is_identity());
			println!("Converted {CNOT_COUNT} cnots to {} cnots", out.len());
		}
	}

	#[test]
	fn postorder_traversal_test1() {
		let mut g: Connectivity<TwoQubitEdge> = Connectivity::new();
		g.add_edge(TwoQubitEdge([0, 2]));
		g.add_edge(TwoQubitEdge([0, 1]));
		let result = g.postorder_traversal(0);
		assert_eq!(result, vec![(1, Some(0)), (2, Some(0)), (0, None)]);
	}

	#[test]
	fn postorder_traversal_test2() {
		let mut g: Connectivity<TwoQubitEdge> = Connectivity::new();
		g.add_edge(TwoQubitEdge([2, 5]));
		g.add_edge(TwoQubitEdge([1, 4]));
		g.add_edge(TwoQubitEdge([1, 3]));
		g.add_edge(TwoQubitEdge([0, 2]));
		g.add_edge(TwoQubitEdge([0, 1]));

		let result = g.postorder_traversal(0);
		assert_eq!(
			result,
			vec![
				(3, Some(1)),
				(4, Some(1)),
				(1, Some(0)),
				(5, Some(2)),
				(2, Some(0)),
				(0, None)
			]
		);
	}

	#[test]
	fn preorder_traversal_test1() {
		let mut g: Connectivity<TwoQubitEdge> = Connectivity::new();
		g.add_edge(TwoQubitEdge([0, 2]));
		g.add_edge(TwoQubitEdge([0, 1]));
		let result = g.preorder_traversal(0);
		assert_eq!(result, vec![(0, None), (1, Some(0)), (2, Some(0))]);
	}

	#[test]
	fn preorder_traversal_test2() {
		let mut g: Connectivity<TwoQubitEdge> = Connectivity::new();
		g.add_edge(TwoQubitEdge([2, 5]));
		g.add_edge(TwoQubitEdge([1, 4]));
		g.add_edge(TwoQubitEdge([1, 3]));
		g.add_edge(TwoQubitEdge([0, 2]));
		g.add_edge(TwoQubitEdge([0, 1]));

		let result = g.preorder_traversal(0);
		assert_eq!(
			result,
			vec![
				(0, None),
				(1, Some(0)),
				(3, Some(1)),
				(4, Some(1)),
				(2, Some(0)),
				(5, Some(2))
			]
		);
	}
}
