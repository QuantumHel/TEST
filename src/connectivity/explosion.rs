use std::collections::{BTreeMap, HashMap};

use petgraph::{Undirected, graph::UnGraph, prelude::StableGraph};

use crate::connectivity::hypergraph::{HyperEdgeIndex, HyperGraph, HyperNodeIndex};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ExplosionNode {
	pub(crate) hyper_edges: Vec<HyperEdgeIndex>,
	pub(crate) hyper_nodes: Vec<HyperNodeIndex>,
}

impl HyperGraph {
	/// Creates a graph where every [HyperEdge] is a node, and these nodes have
	/// edges between them to nodes corresponding to the [HyperNode]s that are shared
	/// between the [HyperEdge]s.
	pub(super) fn explode(&self) -> UnGraph<ExplosionNode, usize> {
		let mut graph: UnGraph<ExplosionNode, usize> = UnGraph::new_undirected();
		let mut edge_nodes = Vec::new();

		let mut node_edge_map: BTreeMap<HyperNodeIndex, Vec<HyperEdgeIndex>> = BTreeMap::new();
		for i in 0..self.nodes.len() {
			node_edge_map.insert(i, Vec::new());
		}
		for (i, edge) in self.edges.iter().enumerate() {
			for node in edge.nodes.iter() {
				node_edge_map.get_mut(node).unwrap().push(HyperEdgeIndex(i));
			}
			edge_nodes.push(graph.add_node(ExplosionNode {
				hyper_edges: vec![HyperEdgeIndex(i)],
				hyper_nodes: Vec::new(),
			}));
		}

		let mut edge_node_map: HashMap<Vec<HyperEdgeIndex>, Vec<HyperNodeIndex>> = HashMap::new();
		for (node, edges) in node_edge_map.into_iter() {
			edge_node_map.entry(edges).or_default().push(node);
		}

		for (mut edges, mut nodes) in edge_node_map.into_iter() {
			if edges.len() == 1 {
				// This is just a normal edge
				let edge = *edges.first().unwrap();
				graph
					.node_weight_mut(*edge_nodes.get(edge.0).unwrap())
					.unwrap()
					.hyper_nodes
					.append(&mut nodes);
			} else {
				// we still need edges
				let node = graph.add_node(ExplosionNode {
					hyper_edges: Vec::new(),
					hyper_nodes: nodes,
				});

				// Connect node to all normal edges
				for edge in edges.iter() {
					let edge_node = edge_nodes.get(edge.0).unwrap();
					graph.add_edge(node, *edge_node, 1);
				}

				// add edge to node
				graph
					.node_weight_mut(node)
					.unwrap()
					.hyper_edges
					.append(&mut edges);
			}
		}

		graph
	}
}

pub(super) fn as_instructions(
	mut steiner_tree: StableGraph<ExplosionNode, usize, Undirected>,
) -> Vec<(HyperEdgeIndex, Option<Vec<HyperNodeIndex>>)> {
	let mut result: Vec<(HyperEdgeIndex, Option<Vec<HyperNodeIndex>>)> = Vec::new();

	while steiner_tree.node_count() != 0 {
		let indices: Vec<_> = steiner_tree.node_indices().collect();
		for index in indices {
			let neighbors: Vec<_> = steiner_tree.neighbors(index).collect();
			if neighbors.len() == 1 {
				let mut node = steiner_tree.remove_node(index).unwrap();
				if node.hyper_edges.len() == 1 {
					// represents and edge
					let edge = node.hyper_edges.pop().unwrap();

					let neighbor_node = steiner_tree
						.node_weight(*neighbors.first().unwrap())
						.unwrap();
					let targets = neighbor_node.hyper_nodes.clone();

					result.push((edge, Some(targets)));
				}
			} else if neighbors.is_empty() {
				let weight = steiner_tree.node_weight(index).unwrap();
				if weight.hyper_edges.len() == 1 {
					assert_eq!(steiner_tree.node_count(), 1);
					let mut node = steiner_tree.remove_node(index).unwrap();
					let edge = node.hyper_edges.pop().unwrap();
					result.push((edge, None));
				} else {
					steiner_tree.remove_node(index).unwrap();
				}
			}
		}
	}

	result
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;

	use petgraph::graph::NodeIndex;

	use crate::connectivity::hypergraph::HyperGraph;

	#[test]
	fn test_explosion() {
		let mut hypergraph = HyperGraph::new();
		let node_0 = hypergraph.add_node();
		let node_1 = hypergraph.add_node();
		let node_2 = hypergraph.add_node();
		let node_3 = hypergraph.add_node();
		let node_4 = hypergraph.add_node();
		let node_5 = hypergraph.add_node();
		let node_6 = hypergraph.add_node();
		let node_7 = hypergraph.add_node();

		let edge_a = hypergraph
			.add_edge([node_0, node_1, node_2, node_3])
			.unwrap();
		let edge_b = hypergraph
			.add_edge([node_1, node_3, node_4, node_5])
			.unwrap();
		let edge_c = hypergraph
			.add_edge([node_1, node_5, node_6, node_7])
			.unwrap();

		let explosion = hypergraph.explode();
		assert_eq!(explosion.edge_count(), 7);
		assert_eq!(explosion.node_count(), 6);

		let mut edgemap: BTreeMap<usize, NodeIndex> = BTreeMap::new();
		for index in explosion.node_indices() {
			let weight = explosion.node_weight(index).unwrap();
			let mut edges = weight.hyper_edges.clone();
			edges.sort();
			let mut nodes = weight.hyper_nodes.clone();
			nodes.sort();
			let target = if edges == vec![edge_a] && nodes == vec![node_0, node_2] {
				0
			} else if edges == vec![edge_a, edge_b] && nodes == vec![node_3] {
				1
			} else if edges == vec![edge_b] && nodes == vec![node_4] {
				2
			} else if edges == vec![edge_b, edge_c] && nodes == vec![node_5] {
				3
			} else if edges == vec![edge_c] && nodes == vec![node_6, node_7] {
				4
			} else if edges == vec![edge_a, edge_b, edge_c] && nodes == vec![node_1] {
				5
			} else {
				unreachable!()
			};

			edgemap.insert(target, index);
		}
		assert_eq!(edgemap.len(), 6);

		let edges = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (0, 5), (2, 5)];
		for (a, b) in edges {
			let a = *edgemap.get(&a).unwrap();
			let b = *edgemap.get(&b).unwrap();
			assert!(explosion.contains_edge(a, b));
		}
	}
}
