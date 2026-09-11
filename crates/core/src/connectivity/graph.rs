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

pub use encapsulation::ConstCardinalityEdge;

/// Allows for the trait and type to have the same name
mod encapsulation {
	use super::Edge;

	/// A marker trait that promises that a type implements [Edge] in such a way
	/// that [Edge::nodes] can only return vectors with the size indicated by
	/// [ConstCardinalityEdge::CARDINALITY].
	///
	/// This means that users can assume that all [Edge]s of this type the indicated
	/// cardinality of [ConstCardinalityEdge::CARDINALITY].
	pub trait ConstCardinalityEdge: Edge {
		/// The constant cardinality of the [Edge].
		///
		/// Because `min_generic_const_args` is not stable this requires the use of
		/// a helper struct [Cardinality\<N\>](Cardinality) where `N` indicates the
		/// cardinality.  
		#[allow(private_bounds)]
		type CARDINALITY: Cardinality;
	}

	/// This is a trait for evading missing constant handling that is behind
	/// `min_generic_const_args`. The only struct implementing this trait is
	/// [Cardinality\<N\>](Cardinality) which contains the needed constant `N`.
	///
	/// The trait is intentionally hidden.
	#[doc(hidden)]
	pub(super) trait Cardinality {}
}

/// This is a helper struct to indicate the cardinality `N` within
/// [ConstCardinalityEdge].
///
/// This struct is for going around the missing const handling from
/// `min_generic_const_args`.
pub struct Cardinality<const N: usize>;

impl<const N: usize> encapsulation::Cardinality for Cardinality<N> {}

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
		self.edges.get(index).and_then(|edge| edge.as_ref())
	}

	fn get_node(&self, index: usize) -> Option<&Subnode<'a, N>> {
		self.nodes.get(index).and_then(|node| node.as_ref())
	}

	fn get_edge_mut(&mut self, index: usize) -> Option<&mut Subedge<'a, E>> {
		self.edges.get_mut(index).and_then(|edge| edge.as_mut())
	}

	fn get_node_mut(&mut self, index: usize) -> Option<&mut Subnode<'a, N>> {
		self.nodes.get_mut(index).and_then(|node| node.as_mut())
	}
}
