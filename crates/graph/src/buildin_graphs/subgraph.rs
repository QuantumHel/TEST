use crate::{ConstCardinalityEdge, Edge, Graph, Node};

#[derive(Debug)]
pub struct Subedge<'a, T: Edge> {
	pub original: &'a T,
	pub(super) nodes: Vec<usize>,
}

impl<'a, E: Edge> Edge for Subedge<'a, E> {
	fn weight(&self) -> f64 {
		self.original.weight()
	}

	fn nodes(&self) -> Vec<usize> {
		self.nodes.clone()
	}
}

impl<'a, T: Edge> Subedge<'a, T> {
	pub fn nodes(&self) -> &[usize] {
		&self.nodes
	}
}

impl<'a, E: ConstCardinalityEdge> ConstCardinalityEdge for Subedge<'a, E> {
	type CARDINALITY = E::CARDINALITY;
}

#[derive(Debug)]
pub struct Subnode<'a, N: Node> {
	pub original: &'a N,
	pub(super) edges: Vec<usize>,
}

impl<'a, N: Node> Node for Subnode<'a, N> {
	fn edges(&self) -> Vec<usize> {
		self.edges.clone()
	}
}

impl<'a, N: Node> Subnode<'a, N> {
	pub fn is_leaf(&self) -> bool {
		self.edges.len() == 1
	}

	pub fn edges(&self) -> &[usize] {
		&self.edges
	}
}

/// A tool for builing [Subgraph]s out of [Graph]s by defining what nodes and
/// edges belong to the [Subgraph].
///
/// # Example:
/// ```rust
/// // TODO
/// ```
#[derive(Debug)]
pub struct SubgraphBuilder<'a, G: Graph<N, E> + ?Sized, N: Node, E: Edge> {
	original: &'a G,
	subgraph: Subgraph<'a, N, E>,
}

impl<'a, G: Graph<N, E> + ?Sized, N: Node, E: Edge> SubgraphBuilder<'a, G, N, E> {
	pub fn new(graph: &'a G) -> Self {
		let mut subgraph = Subgraph {
			edges: Vec::with_capacity(graph.edge_storage_size()),
			nodes: Vec::with_capacity(graph.node_storage_size()),
		};
		subgraph
			.edges
			.resize_with(graph.edge_storage_size(), || None);
		subgraph
			.nodes
			.resize_with(graph.node_storage_size(), || None);

		Self {
			original: graph,
			subgraph,
		}
	}

	pub fn add_edge(&mut self, index: usize) -> Option<&E> {
		match self.original.get_edge(index) {
			Some(edge) => {
				self.subgraph.edges[index] = Some(Subedge {
					original: edge,
					nodes: Vec::new(),
				});
				Some(edge)
			}
			_ => None,
		}
	}

	pub fn remove_edge(&mut self, index: usize) -> Option<&E> {
		match self.subgraph.edges.get_mut(index).map(|a| a.take()) {
			Some(Some(edge)) => Some(edge.original),
			_ => None,
		}
	}

	pub fn has_edge(&self, index: usize) -> bool {
		self.subgraph.edges.get(index).is_some_and(|a| a.is_some())
	}

	pub fn add_node(&mut self, index: usize) -> Option<&N> {
		match self.original.get_node(index) {
			Some(node) => {
				self.subgraph.nodes[index] = Some(Subnode {
					original: node,
					edges: Vec::new(),
				});
				Some(node)
			}
			_ => None,
		}
	}

	pub fn remove_node(&mut self, index: usize) -> Option<&N> {
		match self.subgraph.nodes.get_mut(index).map(|a| a.take()) {
			Some(Some(node)) => Some(node.original),
			_ => None,
		}
	}

	pub fn has_node(&self, index: usize) -> bool {
		self.subgraph.nodes.get(index).is_some_and(|a| a.is_some())
	}

	pub fn finish(self) -> Subgraph<'a, N, E> {
		let Self {
			original,
			mut subgraph,
		} = self;
		let node_indeces: Vec<_> = subgraph
			.nodes
			.iter()
			.enumerate()
			.flat_map(|(i, v)| v.is_some().then_some(i))
			.collect();

		for node in node_indeces.into_iter() {
			for edge in original.get_node(node).unwrap().edges().into_iter() {
				if let Some(Some(subedge)) = subgraph.edges.get_mut(edge) {
					subedge.nodes.push(node);
					subgraph.nodes[node].as_mut().unwrap().edges.push(edge);
				}
			}
		}

		subgraph
	}
}

#[derive(Debug)]
pub struct Subgraph<'a, N: Node, T: Edge> {
	/// Indexes for the edges in the original graph
	edges: Vec<Option<Subedge<'a, T>>>,
	/// Indexes for the qubits in the original graph
	nodes: Vec<Option<Subnode<'a, N>>>,
}

impl<'a, N: Node, T: Edge> Subgraph<'a, N, T> {
	/// Creates an [Subgraph] that contains the whole original graph.
	pub fn full<G: Graph<N, T> + ?Sized>(original: &G) -> Subgraph<'_, N, T> {
		Subgraph {
			edges: (0..original.edge_storage_size())
				.map(|index| {
					original.get_edge(index).map(|original| Subedge {
						nodes: original.nodes(),
						original,
					})
				})
				.collect(),
			nodes: (0..original.node_storage_size())
				.map(|index| {
					original.get_node(index).map(|original| Subnode {
						edges: original.edges(),
						original,
					})
				})
				.collect(),
		}
	}

	/// FIXME: Specify if weak or strong remove (maybe make two functions).
	/// This function results from applying (wrongly) normal graph assumptions
	/// to hypergraphs.
	pub fn remove_node(&mut self, nodes: usize) {
		if let Some(target) = self.nodes.get_mut(nodes).and_then(|a| a.take()) {
			for edge_index in target.edges {
				let edge = self.edges[edge_index].as_mut().unwrap();
				let index = edge.nodes.iter().position(|a| *a == nodes).unwrap();
				edge.nodes.swap_remove(index);
				if edge.nodes.len() < 2 {
					if let Some(&node_in_edge) = edge.nodes.first() {
						let node_in_edge = self.nodes[node_in_edge].as_mut().unwrap();
						let i = node_in_edge
							.edges
							.iter()
							.position(|a| *a == edge_index)
							.unwrap();
						node_in_edge.edges.swap_remove(i);
					}
					self.edges[edge_index] = None;
				}
			}
		}
	}

	/// FIXME: Specify if weak or strong remove (maybe make two functions).
	/// This function results from applying (wrongly) normal graph assumptions
	/// to hypergraphs.
	pub fn remove_edge(&mut self, edge: usize) {
		if let Some(target) = self.edges.get_mut(edge).and_then(|a| a.take()) {
			for qubit_index in target.nodes {
				let qubit = self.nodes[qubit_index].as_mut().unwrap();
				let index = qubit.edges.iter().position(|a| *a == edge).unwrap();
				qubit.edges.swap_remove(index);
				if qubit.edges.is_empty() {
					self.nodes[qubit_index] = None;
				}
			}
		}
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

	fn enumerate_edges<'b>(&'b self) -> impl Iterator<Item = (usize, &'b Subedge<'a, E>)>
	where
		Subedge<'a, E>: 'b,
	{
		self.edges
			.iter()
			.enumerate()
			.filter_map(|(i, a)| a.as_ref().map(|b| (i, b)))
	}

	fn enumerate_nodes<'b>(&'b self) -> impl Iterator<Item = (usize, &'b Subnode<'a, N>)>
	where
		Subnode<'a, N>: 'b,
	{
		self.nodes
			.iter()
			.enumerate()
			.filter_map(|(i, a)| a.as_ref().map(|b| (i, b)))
	}

	fn iter_edges<'b>(&'b self) -> impl Iterator<Item = &'b Subedge<'a, E>>
	where
		Subedge<'a, E>: 'b,
	{
		self.edges.iter().flatten()
	}

	fn iter_nodes<'b>(&'b self) -> impl Iterator<Item = &'b Subnode<'a, N>>
	where
		Subnode<'a, N>: 'b,
	{
		self.nodes.iter().flatten()
	}
}
