use super::{Edge, Graph, Node};

pub trait GraphExt<N: Node, E: Edge>: Graph<N, E> {
	fn is_empty(&self) -> bool {
		for potential_node in 0..self.node_storage_size() {
			if self.get_node(potential_node).is_some() {
				return false;
			}
		}

		true
	}

	/// Returns (node, parent) pairs
	fn postorder_traversal(&self, root: usize) -> Vec<(usize, Option<usize>)> {
		if self.get_node(root).is_none() {
			if self.is_empty() {
				return Vec::new();
			} else {
				panic!("Something went wrong");
			}
		}

		// Rooted DFS postorder over a tree.
		// Returns (node, parent) pairs for all nodes except `root`, where nodes are
		// emitted after their descendants.
		let mut parents: Vec<Option<usize>> = vec![None; self.node_storage_size()];
		parents[root] = Some(root);

		let mut result = Vec::new();
		// (node, parent, children_processed?)
		let mut stack: Vec<(usize, usize, bool)> = Vec::new();
		stack.push((root, root, false));

		while let Some((node, parent, processed)) = stack.pop() {
			if !processed {
				stack.push((node, parent, true));

				for edge_idx in self.get_node(node).unwrap().edges().iter() {
					let edge = self.get_edge(*edge_idx).unwrap();
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
	fn preorder_traversal(&self, root: usize) -> Vec<(usize, Option<usize>)> {
		if self.get_node(root).is_none() {
			if self.is_empty() {
				return Vec::new();
			} else {
				panic!("Something went wrong");
			}
		}

		// Rooted DFS preorder over a tree.
		// Returns (node, parent) pairs for all nodes except `root`
		let mut parents: Vec<Option<usize>> = vec![None; self.node_storage_size()];
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

			for edge_idx in self.get_node(node).unwrap().edges().iter() {
				let edge = self.get_edge(*edge_idx).unwrap();
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
}

impl<G: Graph<N, E>, N: Node, E: Edge> GraphExt<N, E> for G {}
