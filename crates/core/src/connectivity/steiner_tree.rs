use std::collections::{BinaryHeap, HashSet};

use crate::{
	connectivity::{Connectivity, ConnectivityEdge, Subedge, Subgraph, Subnode},
	disjoint_set_forest::DisjointSetForest,
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

impl<T: ConnectivityEdge> Connectivity<T> {
	/// # Panics:
	/// 	if a terminal is not contained in connectivity.
	pub fn steiner_tree(&self, terminals: &[usize]) -> Subgraph<'_, T> {
		// Step 1.
		// immediate predecessor
		let mut pred: Vec<Option<Previous>> = vec![None; self.nodes.len()];
		let mut source: Vec<Option<usize>> = vec![None; self.nodes.len()];
		let mut length: Vec<f64> = vec![f64::INFINITY; self.nodes.len()];

		for qubit in terminals.iter() {
			source[*qubit] = Some(*qubit);
			length[*qubit] = 0.;
		}

		// Step 2.
		let mut q: BinaryHeap<Tuple> = BinaryHeap::new();

		for s in terminals.iter() {
			for (t, d, edge) in self.nodes[*s]
				.edges
				.iter()
				.flat_map(|e| {
					let edge = self.edges.get(*e).unwrap();
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
			let mut terminals_to_set = vec![0; self.nodes.len()];
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

				for edge_index in self.nodes[tuple.t].edges.iter() {
					let edge = self.edges.get(*edge_index).unwrap();
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
			terminals: &[usize],
			edges: &mut HashSet<usize>,
			nodes: &mut HashSet<usize>,
			pred: &Vec<Option<Previous>>,
		) {
			nodes.insert(node);
			if let Some(previous) = pred[node] {
				edges.insert(previous.edge);
				add_edge(previous.node, terminals, edges, nodes, pred);
			}
		}

		for edge in mst_edges.iter() {
			edges.insert(edge.header_edge);
			add_edge(edge.header.0, terminals, &mut edges, &mut nodes, &pred);
			add_edge(edge.header.1, terminals, &mut edges, &mut nodes, &pred);
		}

		let mut sub_graph = Subgraph::empty(&self);

		for edge in edges.iter() {
			sub_graph.edges[*edge] = Some(Subedge {
				original: self.edges.get(*edge).unwrap(),
				nodes: Vec::new(),
			})
		}

		for node in nodes.iter() {
			sub_graph.nodes[*node] = Some(Subnode {
				original: &self.nodes.get(*node).unwrap().edges,
				edges: Vec::new(),
			});

			// inset node into edges, and edge into node
			for edge in self.nodes.get(*node).unwrap().edges.iter() {
				if let Some(Some(sub_edge)) = sub_graph.edges.get_mut(*edge) {
					sub_edge.nodes.push(*node); // ??
					sub_graph.nodes[*node].as_mut().unwrap().edges.push(*edge);
				}
			}
		}

		assert!(sub_graph.is_tree_with(terminals));

		sub_graph
	}
}

#[cfg(test)]
mod tests {
	use crate::connectivity::{Connectivity, ConnectivityEdge};

	#[derive(Debug)]
	struct TestEdge {
		a: usize,
		b: usize,
		weight: f64,
	}

	impl ConnectivityEdge for TestEdge {
		fn nodes(&self) -> Vec<usize> {
			vec![self.a, self.b]
		}

		fn weight(&self) -> f64 {
			self.weight
		}
	}

	#[rustfmt::skip]
	#[test]
	fn aaa() {
		let mut graph: Connectivity<TestEdge> = Connectivity::new();
        graph.add_edge(TestEdge { a: 0, b: 1, weight: 10. }); // V1 - V2  0
        graph.add_edge(TestEdge { a: 0, b: 8, weight: 1. });  // V1 - V9  1
        graph.add_edge(TestEdge { a: 1, b: 2, weight: 8. });  // V2 - V3  2
        graph.add_edge(TestEdge { a: 2, b: 3, weight: 9. });  // V3 - V4  3
        graph.add_edge(TestEdge { a: 3, b: 4, weight: 2. });  // V4 - V5  4
        graph.add_edge(TestEdge { a: 1, b: 5, weight: 1. });  // V2 - V6  5
        graph.add_edge(TestEdge { a: 2, b: 4, weight: 2. });  // V3 - V5  6
        graph.add_edge(TestEdge { a: 4, b: 5, weight: 1. });  // V5 - V6  7
        graph.add_edge(TestEdge { a: 4, b: 8, weight: 1. });  // V5 - V9  8
        graph.add_edge(TestEdge { a: 5, b: 6, weight: 1. });   // V6 - V7 9
        graph.add_edge(TestEdge { a: 6, b: 7, weight: 0.5 }); // V7 - V8 10
        graph.add_edge(TestEdge { a: 7, b: 8, weight: 0.5 }); // V8 - V9 11

		let full_sub = graph.create_subgraph();
		assert!(!full_sub.is_tree());

		let tree = graph.steiner_tree(&[0, 1, 2, 3]);
		assert!(tree.is_tree());
		assert!(tree.is_tree_with(&[0, 1, 2, 3]));
		dbg!(tree);
	}

	#[derive(Debug)]
	struct HyperEdge {
		nodes: Vec<usize>,
		weight: f64,
	}

	impl ConnectivityEdge for HyperEdge {
		fn nodes(&self) -> Vec<usize> {
			self.nodes.clone()
		}

		fn weight(&self) -> f64 {
			self.weight
		}
	}

	#[rustfmt::skip]
	#[test]
	fn hypergraph() {
		let mut graph: Connectivity<HyperEdge> = Connectivity::new();

		// Vertical group on the left (Nodes 1, 2, 3)
		graph.add_edge(HyperEdge { nodes: vec![0, 1, 2], weight: 1. });

		// Horizontal group on the far left (Nodes 4, 5, 3)
		graph.add_edge(HyperEdge { nodes: vec![3, 4, 2], weight: 1. });

		// Top horizontal bridge (Nodes 2, 6, 7, 8)
		graph.add_edge(HyperEdge { nodes: vec![1, 5, 6, 7], weight: 1. });

		// Small ellipse connecting middle nodes (Nodes 6, 7)
		graph.add_edge(HyperEdge { nodes: vec![5, 6], weight: 1. });

		// Large circular cluster on the right (Nodes 7, 8, 9, 10)
		graph.add_edge(HyperEdge { nodes: vec![6, 7, 8, 9], weight: 1. });

		// AAAAAAAAAAAAAAAAAAAAAAAAAAAA (Nodes 12, 11, 13, 10)
		graph.add_edge(HyperEdge { nodes: vec![11, 10, 12, 9], weight: 1. });

		// AAAAAAAAAAAAAAAAAAAAAAAAAAAA (Nodes 2, 3, 5)
		graph.add_edge(HyperEdge { nodes: vec![1, 2, 4], weight: 1. });

		// AAAAAAAAAAAAAAAAAAAAAAAAAAAA (Nodes 9, 10)
		graph.add_edge(HyperEdge { nodes: vec![8, 9], weight: 1. });

		// AAAAAAAAAAAAAAAAAAAAAAAAAAAA (Nodes 11, 13)
		graph.add_edge(HyperEdge { nodes: vec![10, 12], weight: 1. });

		let tree = graph.steiner_tree(&[3, 10, 0]);
		assert!(tree.is_tree());
		assert!(tree.is_tree_with(&[3, 10, 0]));

		dbg!(tree);
	}

	#[rustfmt::skip]
	#[test]
	fn hypergraph_example() {
		let mut graph: Connectivity<HyperEdge> = Connectivity::new();
        graph.add_edge(HyperEdge { nodes: vec![0, 1, 2], weight: 1. });
        graph.add_edge(HyperEdge { nodes: vec![2, 3, 4], weight: 1. });
	}
}
