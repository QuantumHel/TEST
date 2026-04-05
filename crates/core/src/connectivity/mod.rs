mod steiner_tree;
mod sub_graph;

pub use sub_graph::{SubEdge, SubGraph, SubNode};

pub trait ConnectivityEdge {
	fn weight(&self) -> f64;

	fn qubits(&self) -> Vec<usize>;
}

#[derive(Debug, Default)]
pub struct ConnectivityNode {
	pub edges: Vec<usize>,
}

#[derive(Debug)]
pub struct Connectivity<T: ConnectivityEdge> {
	edges: Vec<T>,
	qubits: Vec<ConnectivityNode>,
}

impl<T: ConnectivityEdge> Default for Connectivity<T> {
	fn default() -> Self {
		Self {
			edges: Vec::new(),
			qubits: Vec::new(),
		}
	}
}

impl<T: ConnectivityEdge> Connectivity<T> {
	/// Need to make later a Connectivitybuilder, so that we can assume all
	/// connectivity graphs to be valid in that all qubits are connected
	/// somehow.
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add_edge(&mut self, edge: T) {
		let edge_index = self.edges.len();

		for qubit in edge.qubits() {
			if self.qubits.len() <= qubit {
				self.qubits
					.resize_with(qubit + 1, ConnectivityNode::default);
			}
			self.qubits.get_mut(qubit).unwrap().edges.push(edge_index);
		}

		self.edges.push(edge);
	}

	pub fn edges(&self) -> &[T] {
		&self.edges
	}

	pub fn qubits(&self) -> &[ConnectivityNode] {
		&self.qubits
	}

	pub fn neighbors(&self, qubit: usize) -> Vec<usize> {
		self.qubits[qubit]
			.edges
			.iter()
			.flat_map(|e| {
				let edge = self.edges.get(*e).unwrap();
				edge.qubits().into_iter()
			})
			.filter(|i| qubit != *i)
			.collect()
	}
}
