mod graph;
mod steiner_tree;
mod subgraph;

use std::collections::{HashSet, VecDeque};

pub use graph::Graph;
pub use steiner_tree::steiner_tree;
pub use subgraph::{Subedge, Subgraph, Subnode};

pub use crate::connectivity::graph::{Edge, Node};

#[derive(Debug, Default)]
pub struct ConnectivityNode {
	pub edges: Vec<usize>,
}

impl Node for ConnectivityNode {
	fn edges(&self) -> Vec<usize> {
		self.edges.clone()
	}
}

/// This is still missing many checks, for example edges that have same elements
/// with each other are accepted, but break things. This means that the user
/// needs to manage this case for now. Later this will give an error.
///
/// I think that I solution could be that edges contain the qubits that they act
/// on and a list of the specific gates for said qubit.
#[derive(Debug)]
pub struct Connectivity<T: Edge> {
	edges: Vec<T>,
	nodes: Vec<ConnectivityNode>,
}

impl<T: Edge> Default for Connectivity<T> {
	fn default() -> Self {
		Self {
			edges: Vec::new(),
			nodes: Vec::new(),
		}
	}
}

impl<T: Edge> Connectivity<T> {
	/// Need to make later a Connectivitybuilder, so that we can assume all
	/// connectivity graphs to be valid in that all qubits are connected
	/// somehow.
	pub fn new() -> Self {
		Self::default()
	}

	/// Creates an identical [Subgraph]
	pub fn create_subgraph(&self) -> Subgraph<'_, ConnectivityNode, T> {
		Subgraph {
			edges: self
				.edges
				.iter()
				.map(|original| {
					Some(Subedge {
						nodes: original.nodes(),
						original,
					})
				})
				.collect(),
			nodes: self
				.nodes
				.iter()
				.map(|original| {
					Some(Subnode {
						edges: original.edges.clone(),
						original,
					})
				})
				.collect(),
		}
	}

	pub fn add_edge(&mut self, edge: T) {
		let edge_index = self.edges.len();

		for qubit in edge.nodes() {
			if self.nodes.len() <= qubit {
				self.nodes.resize_with(qubit + 1, ConnectivityNode::default);
			}
			self.nodes.get_mut(qubit).unwrap().edges.push(edge_index);
		}

		self.edges.push(edge);
	}

	pub fn edges(&self) -> &[T] {
		&self.edges
	}

	pub fn nodes(&self) -> &[ConnectivityNode] {
		&self.nodes
	}

	pub fn neighbors(&self, qubit: usize) -> Vec<usize> {
		self.nodes[qubit]
			.edges
			.iter()
			.flat_map(|e| {
				let edge = self.edges.get(*e).unwrap();
				edge.nodes().into_iter()
			})
			.filter(|i| qubit != *i)
			.collect()
	}

	pub fn is_fully_connected(&self) -> bool {
		let mut visited: HashSet<usize> = HashSet::new();
		let mut used_edges: HashSet<usize> = HashSet::new();
		let mut to_visit: VecDeque<usize> = VecDeque::new();

		match self.nodes.first() {
			Some(_) => {
				to_visit.push_front(0);
				visited.insert(0);
			}
			None => {
				return true;
			}
		};

		while let Some(node) = to_visit.pop_back() {
			for edge in self.nodes[node].edges.iter() {
				if used_edges.contains(edge) {
					continue;
				}

				for neighbor in self.edges[*edge].nodes().iter() {
					if *neighbor == node {
						continue;
					}

					if !visited.contains(neighbor) {
						visited.insert(*neighbor);
						to_visit.push_front(*neighbor);
					}
				}

				used_edges.insert(*edge);
			}
		}

		visited.len() == self.nodes.len()
	}

	pub fn no_duplicates(&self) -> bool {
		let mut edges: HashSet<Vec<usize>> = HashSet::new();

		for edge in self.edges.iter() {
			let mut nodes = edge.nodes();
			nodes.sort();
			if !edges.insert(nodes) {
				return false;
			}
		}

		true
	}
}
