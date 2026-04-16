// Need a trait for graph so that can use things like steiner tree thing on
// subgraph and connectivity

use crate::{
	connectivity::{ConnectivityNode, Subedge, Subgraph, Subnode},
	prelude::Connectivity,
};

pub trait Edge {
	fn weight(&self) -> f64;

	fn nodes(&self) -> Vec<usize>;
}

pub trait Node {
	fn edges(&self) -> Vec<usize>;
}

pub trait Graph<N: Node, E: Edge> {
	fn node_storage_size(&self) -> usize;

	fn edge_storage_size(&self) -> usize;

	fn get_node_mut(&mut self, index: usize) -> Option<&mut N>;

	fn get_edge_mut(&mut self, index: usize) -> Option<&mut E>;

	fn get_node(&self, index: usize) -> Option<&N>;

	fn get_edge(&self, index: usize) -> Option<&E>;
}

impl<E: Edge> Graph<ConnectivityNode, E> for Connectivity<E> {
	fn node_storage_size(&self) -> usize {
		self.nodes.len()
	}

	fn edge_storage_size(&self) -> usize {
		self.edges.len()
	}

	fn get_node_mut(&mut self, index: usize) -> Option<&mut ConnectivityNode> {
		self.nodes.get_mut(index)
	}

	fn get_edge_mut(&mut self, index: usize) -> Option<&mut E> {
		self.edges.get_mut(index)
	}

	fn get_node(&self, index: usize) -> Option<&ConnectivityNode> {
		self.nodes.get(index)
	}

	fn get_edge(&self, index: usize) -> Option<&E> {
		self.edges.get(index)
	}
}

impl<'a, N: Node, E: Edge> Graph<Subnode<'a, N>, Subedge<'a, E>> for Subgraph<'a, N, E> {
	fn edge_storage_size(&self) -> usize {
		self.edges.len()
	}

	fn node_storage_size(&self) -> usize {
		self.nodes.len()
	}

	fn get_edge(&self, index: usize) -> Option<&Subedge<'a, E>> {
		self.edges.get(index).map(|edge| edge.as_ref()).flatten()
	}

	fn get_node(&self, index: usize) -> Option<&Subnode<'a, N>> {
		self.nodes.get(index).map(|node| node.as_ref()).flatten()
	}

	fn get_edge_mut(&mut self, index: usize) -> Option<&mut Subedge<'a, E>> {
		self.edges
			.get_mut(index)
			.map(|edge| edge.as_mut())
			.flatten()
	}

	fn get_node_mut(&mut self, index: usize) -> Option<&mut Subnode<'a, N>> {
		self.nodes
			.get_mut(index)
			.map(|node| node.as_mut())
			.flatten()
	}
}
