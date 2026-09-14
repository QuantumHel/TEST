use crate::{Cardinality, ConstCardinalityEdge, Graph, Node};

/// This traits implements additional functionality to normal graphs, aka graphs
/// where [Edge](super::Edge)s implement
/// [ConstCardinalityEdge](super::ConstCardinalityEdge) with
/// [Cardinality\<2\>](Cardinality).
pub trait NormalGraphExt<N: Node, E: ConstCardinalityEdge<CARDINALITY = Cardinality<2>>>:
	Graph<N, E>
{
	/// Returns the indexes of all non-cutting edges. As indicated by the
	/// `_normal` prefix, this is a special version for graphs that have only
	/// normal edges (edges with two nodes).
	///
	/// Based on work of John Hopcroft and Robert Tarjan
	/// https://doi.org/10.1145/362248.362272
	fn normal_non_cutting_nodes(&self) -> Vec<usize> {
		let node_space = self.node_storage_size();

		let mut stack_positions = vec![0usize; node_space];
		// Lowest stack position reachable via subtree
		let mut low = vec![0usize; node_space];
		let mut stack_counter = 1usize;
		let mut non_cutting = vec![false; node_space];

		for root in 0..node_space {
			if self.get_node(root).is_none() {
				continue;
			}

			if *stack_positions.get(root).unwrap() == 0 {
				let child_count = non_cutting_nodes_dfs(
					self,
					root,
					usize::MAX,
					&mut stack_positions,
					&mut low,
					&mut stack_counter,
					&mut non_cutting,
				);

				if child_count < 2 {
					non_cutting[root] = true;
				}
			}
		}

		non_cutting
			.into_iter()
			.enumerate()
			.filter_map(|(i, v)| v.then_some(i))
			.collect()
	}
}

/// Returns child count
fn non_cutting_nodes_dfs<
	N: Node,
	E: ConstCardinalityEdge<CARDINALITY = Cardinality<2>>,
	G: Graph<N, E> + ?Sized,
>(
	graph: &G,
	node_index: usize,
	parent_edge: usize, // Edge index we arrived on (usize::MAX if none)
	stack_positions: &mut [usize],
	low: &mut [usize],
	stack_counter: &mut usize,
	non_cutting: &mut [bool],
) -> usize {
	stack_positions[node_index] = *stack_counter;
	low[node_index] = *stack_counter;
	*stack_counter += 1;

	let node = match graph.get_node(node_index) {
		Some(node) => node,
		None => return 0,
	};

	let mut cuts = false;
	let mut child_count = 0;

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
				child_count += 1;
				non_cutting_nodes_dfs(
					graph,
					neighbor_index,
					edge_index,
					stack_positions,
					low,
					stack_counter,
					non_cutting,
				);

				low[node_index] = low[node_index].min(low[neighbor_index]);

				if low[neighbor_index] >= stack_positions[node_index] {
					cuts = true;
				}
			} else if edge_index != parent_edge {
				low[node_index] = low[node_index].min(stack_positions[neighbor_index]);
			}
		}
	}

	if !cuts {
		non_cutting[node_index] = true;
	}

	child_count
}

impl<G: Graph<N, E>, N: Node, E: ConstCardinalityEdge<CARDINALITY = Cardinality<2>>>
	NormalGraphExt<N, E> for G
{
}
