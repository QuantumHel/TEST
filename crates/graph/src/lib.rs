pub mod algorithms;
mod buildin_graphs;
mod edge;
mod extensions;
mod node;
mod utils;

pub use buildin_graphs::*;
pub use edge::{Cardinality, ConstCardinalityEdge, Edge};
pub use extensions::*;
pub use node::Node;

pub mod prelude {
	pub use crate::subgraph::{Subgraph, SubgraphBuilder};
	pub use crate::{Edge, Graph, GraphExt, Node, NormalGraphExt};
}

pub trait Graph<N: Node, E: Edge> {
	fn node_storage_size(&self) -> usize;

	fn edge_storage_size(&self) -> usize;

	fn get_node(&self, index: usize) -> Option<&N>;

	fn iter_nodes<'a>(&'a self) -> impl Iterator<Item = &'a N>
	where
		N: 'a,
	{
		(0..self.node_storage_size()).filter_map(|index| self.get_node(index))
	}

	fn enumerate_nodes<'a>(&'a self) -> impl Iterator<Item = (usize, &'a N)>
	where
		N: 'a,
	{
		(0..self.node_storage_size()).filter_map(|i| self.get_node(i).map(|n| (i, n)))
	}

	fn get_edge(&self, index: usize) -> Option<&E>;

	fn iter_edges<'a>(&'a self) -> impl Iterator<Item = &'a E>
	where
		E: 'a,
	{
		(0..self.node_storage_size()).filter_map(|index| self.get_edge(index))
	}

	fn enumerate_edges<'a>(&'a self) -> impl Iterator<Item = (usize, &'a E)>
	where
		E: 'a,
	{
		(0..self.edge_storage_size()).filter_map(|i| self.get_edge(i).map(|e| (i, e)))
	}
}
