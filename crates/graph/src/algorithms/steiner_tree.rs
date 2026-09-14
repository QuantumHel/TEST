use std::{
	collections::{BinaryHeap, HashSet},
	fmt::Debug,
};

use crate::{Edge, Graph, Node, utils::DisjointSetForest};
use crate::{
	GraphExt,
	subgraph::{Subgraph, SubgraphBuilder},
};

struct Tuple {
	/// A terminal that is source(p2) if p2 is some, and otherwise there is an
	/// edge (p1, t), and s is a possilbe candidate for source(t)
	t: usize,
	// d = length(p1) + Option(length(p_2)) + d(p1, p2)
	d: f64,
	/// A terminal that is source(p1)
	s: usize,
	p1: usize,
	/// If p2 is some this is the edge between p1, and p2, otherwise (if some)
	/// it is the edge between t and p1.
	edge: Option<usize>,
	/// If this is some, then (s, t) is a possible edge to be used in the
	/// generalized spanning tree, and there is and edge between p1 and p2.
	/// second is element is the index of the edge.
	p2: Option<usize>,
}

impl PartialEq for Tuple {
	fn eq(&self, other: &Self) -> bool {
		self.d == other.d
	}
}

impl Eq for Tuple {}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Tuple {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		other.d.partial_cmp(&self.d)
	}
}

impl Ord for Tuple {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		other.d.total_cmp(&self.d)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Previous {
	node: usize,
	edge: usize,
}

#[derive(Debug)]
struct MSTEdge {
	/// Inserted (s, t)
	#[cfg(debug_assertions)]
	#[allow(unused)]
	mst_edge: (usize, usize),
	/// The edge between header nodes.
	header_edge: usize,
	/// Inserted (p1,  p2)
	header: (usize, usize),
}

/// # Panics:
///
/// Panics if a terminal is not contained in connectivity.
pub fn steiner_tree<'a, 'b: 'c, 'c, G: Graph<N, E> + ?Sized, N: Node, E: Edge>(
	terminals: &'a [usize],
	graph: &'b G,
) -> Subgraph<'c, N, E> {
	// Remove dublicates so that we can assume that there are none.
	let terminals: Vec<usize> = {
		let mut v: Vec<usize> = terminals.to_vec();
		v.sort_unstable();
		v.dedup();
		v
	};

	// Handle special cases that contain no edges
	if terminals.is_empty() {
		return SubgraphBuilder::new(graph).finish();
	} else if terminals.len() == 1 {
		let terminal = *terminals.first().unwrap();
		let mut subgraph = SubgraphBuilder::new(graph);
		subgraph.add_node(terminal);
		return subgraph.finish();
	}

	// Step 1.
	// immediate predecessor
	let mut pred: Vec<Option<Previous>> = vec![None; graph.node_storage_size()];
	let mut source: Vec<Option<usize>> = vec![None; graph.node_storage_size()];
	let mut length: Vec<f64> = vec![f64::INFINITY; graph.node_storage_size()];

	for qubit in terminals.iter() {
		source[*qubit] = Some(*qubit);
		length[*qubit] = 0.;
	}

	// Step 2.
	let mut q: BinaryHeap<Tuple> = BinaryHeap::new();

	for s in terminals.iter() {
		for (t, d, edge) in graph
			.get_node(*s)
			.unwrap()
			.edges()
			.iter()
			.flat_map(|e| {
				let edge = graph.get_edge(*e).unwrap();
				let weight = edge.weight();
				edge.nodes().into_iter().map(move |r| (r, weight, *e))
			})
			.filter(|(r, _, _)| !(terminals.contains(r) && r <= s))
		{
			q.push(Tuple {
				t,
				d,
				s: *s,
				p1: *s,
				edge: Some(edge),
				p2: if terminals.contains(&t) {
					Some(t)
				} else {
					None
				},
			});
		}
	}

	// Step 3.
	// This maps terminals to sets
	let terminal_to_set = {
		let mut terminals_to_set = vec![0; graph.node_storage_size()];
		for (i, terminal) in terminals.iter().enumerate() {
			terminals_to_set[*terminal] = i;
		}
		terminals_to_set
	};
	let mut sets = DisjointSetForest::new(terminals.len());

	// Step 4.
	let mut mst_edges = Vec::new();

	while sets.n_trees() > 1 {
		let tuple = q.pop().expect("Should not be none");
		if source[tuple.t].is_none() {
			source[tuple.t] = Some(tuple.s);
			if let Some(edge) = tuple.edge {
				pred[tuple.t] = Some(Previous {
					node: tuple.p1,
					edge,
				});
			}
			length[tuple.t] = tuple.d;

			for edge_index in graph.get_node(tuple.t).unwrap().edges().iter() {
				let edge = graph.get_edge(*edge_index).unwrap();
				for r in edge.nodes() {
					if source[r].is_none() {
						q.push(Tuple {
							t: r,
							d: tuple.d + edge.weight(),
							s: tuple.s,
							p1: tuple.t,
							edge: Some(*edge_index),
							p2: None,
						});
					}
				}
			}
		} else if sets.find(terminal_to_set[source[tuple.t].unwrap()])
			!= sets.find(terminal_to_set[tuple.s])
		{
			if let Some(p2) = tuple.p2 {
				assert!(terminals.contains(&tuple.t));
				// Case 3.1.
				// There is an MST edge between s and t
				sets.union(terminal_to_set[tuple.s], terminal_to_set[tuple.t]);

				// TODO: edge between headers is missing
				mst_edges.push(MSTEdge {
					#[cfg(debug_assertions)]
					mst_edge: (tuple.s, tuple.t),
					header_edge: tuple.edge.unwrap(),
					header: (tuple.p1, p2),
				});
			} else {
				assert!(!terminals.contains(&tuple.t));
				// Case 3.2.
				q.push(Tuple {
					t: source[tuple.t].unwrap(),
					d: tuple.d + length[tuple.t],
					s: tuple.s,
					p1: tuple.p1,
					edge: tuple.edge,
					p2: Some(tuple.t),
				});
			}
		}
	}

	// Step 5.
	// Resolving found paths to hyperedges
	let mut edges: HashSet<usize> = HashSet::new();
	let mut nodes: HashSet<usize> = HashSet::new();

	fn add_edge(
		node: usize,
		edges: &mut HashSet<usize>,
		nodes: &mut HashSet<usize>,
		pred: &Vec<Option<Previous>>,
	) {
		nodes.insert(node);
		if let Some(previous) = pred[node] {
			edges.insert(previous.edge);
			add_edge(previous.node, edges, nodes, pred);
		}
	}

	for edge in mst_edges.iter() {
		edges.insert(edge.header_edge);
		add_edge(edge.header.0, &mut edges, &mut nodes, &pred);
		add_edge(edge.header.1, &mut edges, &mut nodes, &pred);
	}

	let mut subgraph = SubgraphBuilder::new(graph);

	for edge in edges.into_iter() {
		subgraph.add_edge(edge);
	}

	for node in nodes.into_iter() {
		subgraph.add_node(node);
	}

	let subgraph = subgraph.finish();
	assert!(subgraph.is_tree_with(&terminals));

	subgraph
}

#[cfg(test)]
mod tests {
	use super::steiner_tree;
	use crate::{GraphExt, buildin_graphs::test_graph::TestGraph};

	#[test]
	fn aaa() {
		let mut graph = TestGraph::default();
		graph.add_edge(10., &[0, 1]); // V1 - V2  0
		graph.add_edge(1., &[0, 8]); // V1 - V9  1
		graph.add_edge(8., &[1, 2]); // V2 - V3  2
		graph.add_edge(9., &[2, 3]); // V3 - V4  3
		graph.add_edge(2., &[3, 4]); // V4 - V5  4
		graph.add_edge(1., &[1, 5]); // V2 - V6  5
		graph.add_edge(2., &[2, 4]); // V3 - V5  6
		graph.add_edge(1., &[4, 5]); // V5 - V6  7
		graph.add_edge(1., &[4, 8]); // V5 - V9  8
		graph.add_edge(1., &[5, 6]); // V6 - V7 9
		graph.add_edge(0.5, &[6, 7]); // V7 - V8 10
		graph.add_edge(0.5, &[7, 8]); // V8 - V9 11

		assert!(!graph.is_tree());
		let tree = steiner_tree(&[0, 1, 2, 3], &graph);
		assert!(tree.is_tree());
		assert!(tree.is_tree_with(&[0, 1, 2, 3]));
	}

	#[test]
	fn none_or_one_terminal() {
		let mut graph = TestGraph::default();
		graph.add_edge(10., &[0, 1]); // V1 - V2  0
		graph.add_edge(1., &[0, 8]); // V1 - V9  1
		graph.add_edge(8., &[1, 2]); // V2 - V3  2
		graph.add_edge(9., &[2, 3]); // V3 - V4  3
		graph.add_edge(2., &[3, 4]); // V4 - V5  4
		graph.add_edge(1., &[1, 5]); // V2 - V6  5
		graph.add_edge(2., &[2, 4]); // V3 - V5  6
		graph.add_edge(1., &[4, 5]); // V5 - V6  7
		graph.add_edge(1., &[4, 8]); // V5 - V9  8
		graph.add_edge(1., &[5, 6]); // V6 - V7 9
		graph.add_edge(0.5, &[6, 7]); // V7 - V8 10
		graph.add_edge(0.5, &[7, 8]); // V8 - V9 11

		let tree = steiner_tree(&[1], &graph);
		assert!(tree.is_tree());
		assert!(tree.is_tree_with(&[1]));

		let tree = steiner_tree(&[0], &graph);
		assert!(tree.is_tree());
		assert!(tree.is_tree_with(&[0]));
	}

	#[test]
	fn hypergraph() {
		let mut graph = TestGraph::default();
		graph.add_edge(1., &[0, 1, 2]);
		graph.add_edge(1., &[3, 4, 2]);
		graph.add_edge(1., &[1, 5, 6, 7]);
		graph.add_edge(1., &[5, 6]);
		graph.add_edge(1., &[6, 7, 8, 9]);
		graph.add_edge(1., &[11, 10, 12, 9]);
		graph.add_edge(1., &[1, 2, 4]);
		graph.add_edge(1., &[8, 9]);
		graph.add_edge(1., &[10, 12]);

		let tree = steiner_tree(&[3, 10, 0], &graph);
		assert!(tree.is_tree());
		assert!(tree.is_tree_with(&[3, 10, 0]));
	}

	#[test]
	fn hypergraph_example() {
		let mut graph = TestGraph::default();
		graph.add_edge(1., &[0, 1, 2]);
		graph.add_edge(1., &[2, 3, 4]);
	}
}
