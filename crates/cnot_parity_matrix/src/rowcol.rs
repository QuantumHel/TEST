use bits::Bits;
use circuit::gates::CNot;
use test_core::prelude::*;

use crate::ParityMatrix;

#[derive(Debug)]
pub struct TwoQubitEdge([usize; 2]);

impl Edge for TwoQubitEdge {
	fn nodes(&self) -> Vec<usize> {
		self.0.to_vec()
	}

	fn weight(&self) -> f64 {
		1.0
	}
}

/// Returns (node, parent) pairs
fn postorder_traversal<G: Graph<N, E>, N: Node, E: Edge>(
	root: usize,
	graph: &G,
) -> Vec<(usize, Option<usize>)> {
	// Rooted DFS postorder over a tree.
	// Returns (node, parent) pairs for all nodes except `root`, where nodes are
	// emitted after their descendants.
	let mut parents: Vec<Option<usize>> = vec![None; graph.node_storage_size()];
	parents[root] = Some(root);

	let mut result = Vec::new();
	// (node, parent, children_processed?)
	let mut stack: Vec<(usize, usize, bool)> = Vec::new();
	stack.push((root, root, false));

	while let Some((node, parent, processed)) = stack.pop() {
		if !processed {
			stack.push((node, parent, true));

			for edge_idx in graph.get_node(node).unwrap().edges().iter() {
				let edge = graph.get_edge(*edge_idx).unwrap();
				for &neighbor in edge.nodes().iter() {
					// This means that we haven't processed the neighbor yet.
					if parents[neighbor].is_none() {
						parents[neighbor] = Some(node);
						stack.push((neighbor, node, false));
					}
				}
			}
		} else if node == root {
			result.push((node, None));
		} else {
			result.push((node, Some(parent)))
		}
	}

	result
}

/// Returns (node, parent) pairs
fn preorder_traversal<G: Graph<N, E>, N: Node, E: Edge>(
	root: usize,
	graph: &G,
) -> Vec<(usize, Option<usize>)> {
	// Rooted DFS preorder over a tree.
	// Returns (node, parent) pairs for all nodes except `root`
	let mut parents: Vec<Option<usize>> = vec![None; graph.node_storage_size()];
	parents[root] = Some(root);

	let mut result = Vec::new();
	// (node, parent)
	let mut stack: Vec<(usize, usize)> = Vec::new();
	stack.push((root, root));

	while let Some((node, parent)) = stack.pop() {
		if node == root {
			result.push((node, None));
		} else {
			result.push((node, Some(parent)));
		}

		for edge_idx in graph.get_node(node).unwrap().edges().iter() {
			let edge = graph.get_edge(*edge_idx).unwrap();
			for &neighbor in edge.nodes().iter() {
				// If the parent is some we have processed neighbor already
				if parents[neighbor].is_none() {
					parents[neighbor] = Some(node);
					stack.push((neighbor, node));
				}
			}
		}
	}

	result
}

/// An implementation of the rowcol algorithm described in
/// https://doi.org/10.1103/PhysRevResearch.5.013065
pub fn rowcol(
	mut matrix: ParityMatrix,
	n: usize,
	connectivity: &Connectivity<TwoQubitEdge>,
) -> Vec<CNot> {
	let mut result = Vec::new();
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
			for (j, k) in postorder_traversal(i, &tree) {
				if let Some(k) = k
					&& matrix.get(j, i)
					&& !matrix.get(k, i)
				{
					result.push(matrix.add_row(j, k));
				}
			}

			// 5
			for (j, k) in postorder_traversal(i, &tree) {
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

			for (j, parent) in preorder_traversal(i, &tree_prime) {
				if let Some(parent) = parent
					&& !s_prime.contains(&j)
				{
					result.push(matrix.add_row(j, parent));
				}
			}

			for (j, parent) in postorder_traversal(i, &tree_prime) {
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

#[cfg(test)]
mod tests {
	use super::*;
	use rand::prelude::*;
	use rand_chacha::ChaCha8Rng;

	#[test]
	fn rowcol_random_test() {
		const TEST_COUNT: usize = 100;
		let mut rng = ChaCha8Rng::seed_from_u64(2);

		let mut g: Connectivity<TwoQubitEdge> = Connectivity::new();
		g.add_edge(TwoQubitEdge([2, 5]));
		g.add_edge(TwoQubitEdge([1, 4]));
		g.add_edge(TwoQubitEdge([1, 3]));
		g.add_edge(TwoQubitEdge([0, 2]));
		g.add_edge(TwoQubitEdge([0, 1]));

		let n = g.nodes().len();

		for _ in 0..TEST_COUNT {
			let cnots: Vec<_> = (0..(n * 100)).map(|_| CNot::random(n, &mut rng)).collect();

			let mut parity_matrix = ParityMatrix::default();
			for cnot in cnots {
				parity_matrix.insert_cnot(cnot);
			}

			let out = rowcol(parity_matrix.clone(), n, &g);

			for cnot in out.iter().rev() {
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
		let result = postorder_traversal(0, &g);
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

		let result = postorder_traversal(0, &g);
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
		let result = preorder_traversal(0, &g);
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

		let result = preorder_traversal(0, &g);
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
