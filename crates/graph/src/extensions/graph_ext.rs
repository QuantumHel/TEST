use std::collections::{HashSet, VecDeque};

use crate::{
	Edge, Graph, Node,
	incidence_graph::{IncidenceGraph, IncidenceNode},
	subgraph::Subgraph,
};

use super::NormalGraphExt;

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

	/// Returns the indexes of non-cutting edges.
	///
	/// The returned edges are non-cutting assuming weak edge deletion, that is
	/// deleting an edge does not delete nodes.
	///
	/// FIXME: This method needs a proof as it is kinda a non-trivial
	/// generalization.
	///
	/// Based on work of John Hopcroft and Robert Tarjan
	/// https://doi.org/10.1145/362248.362272
	fn non_cutting_edges(&self) -> Vec<usize> {
		let node_space = self.node_storage_size();

		let mut stack_positions = vec![0usize; node_space];
		// Lowest stack position reachable via subtree
		let mut low = vec![0usize; node_space];
		let mut stack_counter = 1usize;
		let mut cutting = vec![false; self.edge_storage_size()];

		for root in 0..node_space {
			if self.get_node(root).is_none() {
				continue;
			}

			if *stack_positions.get(root).unwrap() == 0 {
				non_cutting_edges_dfs(
					self,
					root,
					usize::MAX,
					&mut stack_positions,
					&mut low,
					&mut stack_counter,
					&mut cutting,
				);
			}
		}

		cutting
			.into_iter()
			.enumerate()
			.filter_map(|(i, v)| (!v).then_some(i))
			.collect()
	}

	/// Returns the indexes of non-cutting nodes.
	///
	/// The returned nodes are non-cutting assuming weak node deletion, tiat is
	/// deleting a node, nodes not delete edges that have other nodes.
	///
	/// Uses Theorem 3.24 from https://arxiv.org/pdf/1504.04274 about incidence
	/// graph based non_cutting edge calculation (which is done using a method
	/// based on work of John Hopcroft and Robert Tarjan
	/// https://doi.org/10.1145/362248.362272 )
	fn non_cutting_nodes(&self) -> Vec<usize> {
		let indicence_graph = self.incidence_graph();
		let non_cutting = indicence_graph.normal_non_cutting_nodes();
		non_cutting
			.into_iter()
			.filter_map(|index| {
				if let Some(IncidenceNode::Node { index, .. }) = indicence_graph.get_node(index) {
					Some(*index)
				} else {
					None
				}
			})
			.collect()
	}

	fn incidence_graph(&self) -> IncidenceGraph {
		IncidenceGraph::new_from(self)
	}

	fn steiner_tree<'a>(&'a self, terminals: &[usize]) -> Subgraph<'a, N, E> {
		crate::algorithms::steiner_tree(terminals, self)
	}

	/// Creates a subgraph that contains the whole graph.
	fn full_subgraph(&self) -> Subgraph<'_, N, E> {
		Subgraph::full(self)
	}

	/// Iterates over all nodes that belong to one or less nodes.
	fn leaf_nodes<'a>(&'a self) -> impl Iterator<Item = &'a N>
	where
		N: 'a,
	{
		self.iter_nodes().filter(|n| n.edges().len() < 2)
	}

	/// Iterates over all (index, node) pairs where node belongs to one or less
	/// edges.
	fn enumerate_leaf_nodes<'a>(&'a self) -> impl Iterator<Item = (usize, &'a N)>
	where
		N: 'a,
	{
		self.enumerate_nodes().filter(|(_, n)| n.edges().len() < 2)
	}

	fn is_tree(&self) -> bool {
		let mut visited: HashSet<usize> = HashSet::new();
		let mut used_edges: HashSet<usize> = HashSet::new();
		let mut to_visit: VecDeque<usize> = VecDeque::new();

		if let Some((first, _)) = self.enumerate_nodes().next() {
			to_visit.push_front(first);
			visited.insert(first);
		} else {
			return true;
		}

		while let Some(node) = to_visit.pop_back() {
			for edge in self.get_node(node).unwrap().edges() {
				if used_edges.contains(&edge) {
					continue;
				}

				for neighbor in self.get_edge(edge).unwrap().nodes() {
					if neighbor == node {
						continue;
					}

					if visited.contains(&neighbor) {
						return false;
					} else {
						visited.insert(neighbor);
						to_visit.push_front(neighbor);
					}
				}

				used_edges.insert(edge);
			}
		}

		visited.len() == self.iter_nodes().count()
	}

	fn is_tree_with(&self, terminals: &[usize]) -> bool {
		if !self.is_tree() {
			return false;
		}

		for &terminal in terminals {
			if self.get_node(terminal).is_none() {
				return false;
			}
		}

		true
	}
}

impl<G: Graph<N, E>, N: Node, E: Edge> GraphExt<N, E> for G {}

fn non_cutting_edges_dfs<N: Node, E: Edge, G: Graph<N, E> + ?Sized>(
	graph: &G,
	node_index: usize,
	parent_edge: usize, // Edge index we arrived on (usize::MAX if none)
	stack_positions: &mut [usize],
	low: &mut [usize],
	stack_counter: &mut usize,
	cutting: &mut [bool],
) {
	stack_positions[node_index] = *stack_counter;
	low[node_index] = *stack_counter;
	*stack_counter += 1;

	let node = match graph.get_node(node_index) {
		Some(node) => node,
		None => return,
	};

	if node.edges().len() == 1 {
		cutting[node.edges()[0]] = true
	}

	for &edge_index in &node.edges() {
		let edge = match graph.get_edge(edge_index) {
			Some(e) => e,
			None => continue,
		};

		for &neighbor_index in edge.nodes().iter() {
			if neighbor_index == node_index {
				continue;
			}

			if *stack_positions.get(neighbor_index).unwrap() == 0 {
				non_cutting_edges_dfs(
					graph,
					neighbor_index,
					edge_index,
					stack_positions,
					low,
					stack_counter,
					cutting,
				);

				low[node_index] = low[node_index].min(low[neighbor_index]);

				if low[neighbor_index] > stack_positions[node_index] {
					cutting[edge_index] = true
				}
			} else if edge_index != parent_edge {
				low[node_index] = low[node_index].min(stack_positions[neighbor_index]);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::GraphExt;
	use crate::{Edge, Graph, Node};

	#[derive(Default, Debug)]
	struct TestEdge {
		nodes: Vec<usize>,
	}

	impl Edge for TestEdge {
		fn nodes(&self) -> Vec<usize> {
			self.nodes.clone()
		}

		fn weight(&self) -> f64 {
			1.
		}
	}

	#[derive(Default, Debug)]
	struct TestNode {
		edges: Vec<usize>,
	}

	impl Node for TestNode {
		fn edges(&self) -> Vec<usize> {
			self.edges.clone()
		}
	}

	#[derive(Default, Debug)]
	struct TestGraph {
		nodes: Vec<TestNode>,
		edges: Vec<TestEdge>,
	}

	impl TestGraph {
		fn add_edge(&mut self, nodes: &[usize]) {
			let Some(&n) = nodes.iter().max() else {
				return;
			};
			while self.nodes.len() <= n {
				self.nodes.push(TestNode::default());
			}

			let edge = self.edges.len();
			self.edges.push(TestEdge {
				nodes: nodes.to_vec(),
			});

			for &node in nodes {
				self.nodes.get_mut(node).unwrap().edges.push(edge);
			}
		}
	}

	impl Graph<TestNode, TestEdge> for TestGraph {
		fn edge_storage_size(&self) -> usize {
			self.edges.len()
		}

		fn node_storage_size(&self) -> usize {
			self.nodes.len()
		}

		fn get_edge(&self, index: usize) -> Option<&TestEdge> {
			self.edges.get(index)
		}

		fn get_node(&self, index: usize) -> Option<&TestNode> {
			self.nodes.get(index)
		}
	}

	fn assert_content(mut a: Vec<usize>, mut b: Vec<usize>) {
		a.sort();
		b.sort();
		assert_eq!(a, b)
	}

	#[test]
	fn test_noncut_line() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1]);
		g.add_edge(&[1, 2]);

		assert!(g.non_cutting_edges().is_empty());
		assert_content(g.non_cutting_nodes(), vec![2, 0]);
	}

	#[test]
	fn test_noncut_loop() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1]);
		g.add_edge(&[1, 2]);
		g.add_edge(&[2, 0]);

		assert_content(g.non_cutting_edges(), vec![0, 1, 2]);
		assert_content(g.non_cutting_nodes(), vec![2, 1, 0]);
	}

	#[test]
	fn test_noncut_loop_in_middle() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1]); // 0
		g.add_edge(&[1, 2]); // 1
		g.add_edge(&[2, 3]); // 2
		g.add_edge(&[2, 4]); // 3
		g.add_edge(&[4, 1]); // 4

		assert_content(g.non_cutting_edges(), vec![1, 3, 4]);
		assert_content(g.non_cutting_nodes(), vec![3, 4, 0]);
	}

	#[test]
	fn test_noncut_double_edge() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1]); // 0
		g.add_edge(&[1, 0]); // 1
		assert_content(g.non_cutting_edges(), vec![0, 1]);
		assert_content(g.non_cutting_nodes(), vec![0, 1]);
	}

	#[test]
	fn test_noncut_single_hyper_edge() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1, 2]);
		assert!(g.non_cutting_edges().is_empty());
		assert_content(g.non_cutting_nodes(), vec![0, 1, 2]);
	}

	#[test]
	fn test_noncut_double_overlap() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1, 2]);
		g.add_edge(&[1, 2, 3]);
		assert!(g.non_cutting_edges().is_empty());
		assert_content(g.non_cutting_nodes(), vec![0, 1, 2, 3]);
	}

	#[test]
	fn test_noncut_dublicat_hyper_edge() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1, 2]);
		g.add_edge(&[0, 1, 2]);
		assert_content(g.non_cutting_edges(), vec![0, 1]);
		assert_content(g.non_cutting_nodes(), vec![0, 1, 2]);
	}

	#[test]
	fn test_noncut_two_hyper_edge_cliques() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1, 2]);
		g.add_edge(&[2, 3, 4]);
		assert!(g.non_cutting_edges().is_empty());
		assert_content(g.non_cutting_nodes(), vec![0, 1, 3, 4]);
	}

	#[test]
	fn test_noncut_double_hyper_and_normal_bridge() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1, 2]);
		g.add_edge(&[0, 1, 2]);
		g.add_edge(&[2, 3]);
		assert_content(g.non_cutting_edges(), vec![0, 1]);
		assert_content(g.non_cutting_nodes(), vec![0, 1, 3]);
	}

	#[test]
	fn test_noncut_hyper_chain() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1, 2, 3]);
		g.add_edge(&[3, 4, 5, 6]);
		g.add_edge(&[6, 7, 8, 9]);
		assert!(g.non_cutting_edges().is_empty());
		assert_content(g.non_cutting_nodes(), vec![0, 1, 2, 4, 5, 7, 8, 9]);
	}

	#[test]
	fn test_noncut_overlapping_hyper_chain() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1, 2, 3]);
		g.add_edge(&[2, 3, 4, 5]);
		g.add_edge(&[3, 4, 5, 6]);
		g.add_edge(&[5, 6, 7]);
		g.add_edge(&[6, 7, 8, 9]);
		assert_content(g.non_cutting_edges(), vec![1, 2, 3]);
		assert_content(g.non_cutting_nodes(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
	}

	#[test]
	fn test_noncut_hyperloop() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1, 2]);
		g.add_edge(&[2, 3, 4]);
		g.add_edge(&[4, 5]);
		g.add_edge(&[0, 5]);
		assert_content(g.non_cutting_edges(), vec![2, 3]);
		assert_content(g.non_cutting_nodes(), vec![0, 1, 2, 3, 4, 5]);
	}

	#[test]
	fn test_noncut_hyper_loop2() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1, 2, 3]);
		g.add_edge(&[1, 2, 3]);
		g.add_edge(&[3, 4]);
		assert_content(g.non_cutting_edges(), vec![1]);
		assert_content(g.non_cutting_nodes(), vec![0, 1, 2, 4]);
	}

	#[test]
	fn test_noncut_hyperloop2() {
		let mut g = TestGraph::default();
		g.add_edge(&[0, 1]);
		g.add_edge(&[1, 2]);
		g.add_edge(&[2, 3]);
		g.add_edge(&[0, 2, 3]);
		assert_content(g.non_cutting_edges(), vec![0, 1, 2, 3]);
		assert_content(g.non_cutting_nodes(), vec![0, 1, 2, 3]);
	}
}
