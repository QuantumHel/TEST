use test_bits::Bits;
use test_circuit::{Circuit, gates::CNot};
use test_core::prelude::*;

use crate::{ParityMatrix, TwoQubitEdge};

/// An implementation of the rowcol algorithm described in
/// https://doi.org/10.1103/PhysRevResearch.5.013065
pub struct RowCol;

impl Compiler<ParityMatrix, Circuit<CNot>, Connectivity<TwoQubitEdge>> for RowCol {
	fn compile(
		&self,
		mut matrix: ParityMatrix,
		connectivity: &Connectivity<TwoQubitEdge>,
	) -> Circuit<CNot> {
		let n = connectivity.nodes().len();
		let mut result = Circuit::new();
		let mut g = connectivity.create_subgraph();
		// Change to BFS at some point?
		let mut total_tree = steiner_tree(&(0..n).collect::<Vec<usize>>(), connectivity);

		loop {
			let leafs = total_tree.leaf_nodes();
			if leafs.is_empty() {
				break;
			}

			// 1
			for i in leafs.iter().map(|(i, _)| *i).collect::<Vec<_>>() {
				// 2
				let s: Vec<_> = (0..n).filter(|j| matrix.get(*j, i)).chain([i]).collect();

				// 3
				let tree = steiner_tree(&s, &g);

				// 4
				for (j, k) in tree.postorder_traversal(i) {
					if let Some(k) = k
						&& matrix.get(j, i)
						&& !matrix.get(k, i)
					{
						result.push(matrix.add_row(j, k));
					}
				}

				// 5
				for (j, k) in tree.postorder_traversal(i) {
					for edge in tree.get_node(j).unwrap().edges() {
						let neighbor: usize = *tree
							.get_edge(*edge)
							.unwrap()
							.nodes()
							.iter()
							.find(|n| **n != j)
							.unwrap();

						if let Some(k) = k
							&& neighbor == k
						{
							continue;
						}

						result.push(matrix.add_row(j, neighbor));
					}
				}

				for j in 0..n {
					assert_eq!(matrix.get(j, i), j == i);
				}

				// 6
				let sum_target = {
					let mut original = matrix.get_row(i);
					original.set(i, !original.get(i));
					original
				};

				let s_prime: Vec<usize> = matrix
					.span_bits(&sum_target)
					.expect("should be impossible")
					.iter_ones()
					.collect();
				let terminals = {
					let mut terminals: Vec<usize> = s_prime.clone();
					terminals.push(i);
					terminals
				};

				let tree_prime = steiner_tree(&terminals, &g);

				for (j, parent) in tree_prime.preorder_traversal(i) {
					if let Some(parent) = parent
						&& !s_prime.contains(&j)
					{
						result.push(matrix.add_row(j, parent));
					}
				}

				for (j, parent) in tree_prime.postorder_traversal(i) {
					if let Some(parent) = parent {
						result.push(matrix.add_row(j, parent));
					}
				}

				assert_eq!(matrix.get_row(i), Bits::with_one(i));

				g.remove_node(i);
				total_tree.remove_node(i);
			}
		}

		result.reverse();
		result
	}
}

#[cfg(test)]
mod tests {
	use super::*;
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

			let compiler = RowCol;
			let out = compiler.compile(parity_matrix.clone(), &g);

			for cnot in out.iter().rev() {
				assert!(g.neighbors(cnot.control()).contains(&cnot.target()));
				parity_matrix.insert_cnot(*cnot);
			}

			assert!(parity_matrix.is_identity());
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
