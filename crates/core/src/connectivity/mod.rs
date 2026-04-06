mod steiner_tree;
mod subgraph;

pub use subgraph::{Subedge, Subgraph, Subnode};

pub trait ConnectivityEdge {
	fn weight(&self) -> f64;

	fn nodes(&self) -> Vec<usize>;
}

#[derive(Debug, Default)]
pub struct ConnectivityNode {
	pub edges: Vec<usize>,
}

#[derive(Debug)]
pub struct Connectivity<T: ConnectivityEdge> {
	edges: Vec<T>,
	nodes: Vec<ConnectivityNode>,
}

impl<T: ConnectivityEdge> Default for Connectivity<T> {
	fn default() -> Self {
		Self {
			edges: Vec::new(),
			nodes: Vec::new(),
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

	/// Creates an identical [Subgraph]
	pub fn create_subgraph(&self) -> Subgraph<'_, T> {
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
						original: &original.edges,
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
}
