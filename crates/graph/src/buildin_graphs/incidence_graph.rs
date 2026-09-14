use crate::{Cardinality, ConstCardinalityEdge, Edge, Graph, Node};

#[derive(Debug)]
pub enum IncidenceNode {
	Node { index: usize, edges: Vec<usize> },
	Edge { index: usize, edges: Vec<usize> },
}

impl Node for IncidenceNode {
	fn edges(&self) -> Vec<usize> {
		match self {
			Self::Node { edges, .. } | Self::Edge { edges, .. } => edges.clone(),
		}
	}
}

impl IncidenceNode {
	fn add_edge(&mut self, index: usize) {
		match self {
			Self::Node { edges, .. } | Self::Edge { edges, .. } => edges.push(index),
		}
	}
}

#[derive(Debug)]
pub struct IncidenceEdge {
	edges: [usize; 2],
}

impl Edge for IncidenceEdge {
	fn weight(&self) -> f64 {
		1.
	}

	fn nodes(&self) -> Vec<usize> {
		self.edges.to_vec()
	}
}

impl ConstCardinalityEdge for IncidenceEdge {
	type CARDINALITY = Cardinality<2>;
}

#[derive(Debug)]
pub struct IncidenceGraph {
	edges: Vec<IncidenceEdge>,
	nodes: Vec<IncidenceNode>,
}

impl Graph<IncidenceNode, IncidenceEdge> for IncidenceGraph {
	fn edge_storage_size(&self) -> usize {
		self.edges.len()
	}

	fn get_edge(&self, index: usize) -> Option<&IncidenceEdge> {
		self.edges.get(index)
	}

	fn get_node(&self, index: usize) -> Option<&IncidenceNode> {
		self.nodes.get(index)
	}

	fn node_storage_size(&self) -> usize {
		self.nodes.len()
	}

	fn enumerate_edges<'a>(&'a self) -> impl Iterator<Item = (usize, &'a IncidenceEdge)>
	where
		IncidenceEdge: 'a,
	{
		self.edges.iter().enumerate()
	}

	fn enumerate_nodes<'a>(&'a self) -> impl Iterator<Item = (usize, &'a IncidenceNode)>
	where
		IncidenceNode: 'a,
	{
		self.nodes.iter().enumerate()
	}

	fn iter_edges<'a>(&'a self) -> impl Iterator<Item = &'a IncidenceEdge>
	where
		IncidenceEdge: 'a,
	{
		self.edges.iter()
	}

	fn iter_nodes<'a>(&'a self) -> impl Iterator<Item = &'a IncidenceNode>
	where
		IncidenceNode: 'a,
	{
		self.nodes.iter()
	}
}

impl IncidenceGraph {
	pub fn new_from<G: Graph<N, E> + ?Sized, N: Node, E: Edge>(original: &G) -> IncidenceGraph {
		let n_nodes = original.node_storage_size();
		let n_edges = original.edge_storage_size();

		// map from original index to (new index, )
		let mut index_counter: usize = 0;
		let mut node_nodes: Vec<_> = (0..n_nodes)
			.map(|index| match original.get_node(index) {
				Some(_) => {
					let new_index = index_counter;
					index_counter += 1;
					Some((
						new_index,
						IncidenceNode::Node {
							index,
							edges: Vec::new(),
						},
					))
				}
				_ => None,
			})
			.collect();
		let mut edge_nodes: Vec<_> = (0..n_edges)
			.map(|index| match original.get_edge(index) {
				Some(_) => {
					let new_index = index_counter;
					index_counter += 1;
					Some((
						new_index,
						IncidenceNode::Edge {
							index,
							edges: Vec::new(),
						},
					))
				}
				_ => None,
			})
			.collect();

		let mut edges: Vec<IncidenceEdge> = Vec::new();
		for node_index in 0..n_nodes {
			let Some(node) = original.get_node(node_index) else {
				continue;
			};
			let Some(Some((a_index, a))) = node_nodes.get_mut(node_index) else {
				continue;
			};
			for edge_index in node.edges() {
				let Some(Some((b_index, b))) = edge_nodes.get_mut(edge_index) else {
					continue;
				};
				a.add_edge(edges.len());
				b.add_edge(edges.len());
				edges.push(IncidenceEdge {
					edges: [*a_index, *b_index],
				});
			}
		}

		IncidenceGraph {
			edges,
			nodes: node_nodes
				.into_iter()
				.chain(edge_nodes)
				.flatten()
				.map(|(_, n)| n)
				.collect(),
		}
	}
}
