use std::{
	collections::{HashSet, VecDeque},
	usize,
};

use crate::connectivity::{Edge, Graph, Node};

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

#[derive(Debug)]
pub struct Subgraph<'a, N: Node, T: Edge> {
	/// Indexes for the edges in the original graph
	pub(super) edges: Vec<Option<Subedge<'a, T>>>,
	/// Indexes for the qubits in the original graph
	pub(super) nodes: Vec<Option<Subnode<'a, N>>>,
}

impl<'a, N: Node, T: Edge> Subgraph<'a, N, T> {
	pub(super) fn empty<G: Graph<N, T>>(graph: &'a G) -> Self {
		let mut edges: Vec<Option<Subedge<'a, T>>> = Vec::with_capacity(graph.edge_storage_size());
		edges.resize_with(graph.edge_storage_size(), || None);
		let mut nodes: Vec<Option<Subnode<'a, N>>> = Vec::with_capacity(graph.node_storage_size());
		nodes.resize_with(graph.node_storage_size(), || None);

		Self { edges, nodes }
	}

	pub fn get_edge(&self, index: usize) -> Option<&Subedge<'a, T>> {
		self.edges.get(index).map(Option::as_ref).flatten()
	}

	pub fn get_node(&self, index: usize) -> Option<&Subnode<'a, N>> {
		self.nodes.get(index).map(Option::as_ref).flatten()
	}

	pub fn remove_node(&mut self, nodes: usize) {
		if let Some(target) = self.nodes.get_mut(nodes).map(|a| a.take()).flatten() {
			for edge_index in target.edges {
				let edge = self.edges[edge_index].as_mut().unwrap();
				let index = edge.nodes.iter().position(|a| *a == nodes).unwrap();
				edge.nodes.swap_remove(index);
				if edge.nodes.len() < 2 {
					self.remove_edge(edge_index);
				}
			}
		}
	}

	pub fn remove_edge(&mut self, edge: usize) {
		if let Some(target) = self.edges.get_mut(edge).map(|a| a.take()).flatten() {
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

	/// qubits that belong to only one edge
	pub fn leaf_nodes(&self) -> Vec<(usize, &Subnode<'_, N>)> {
		self.nodes
			.iter()
			.map(Option::as_ref)
			.enumerate()
			.filter_map(|(i, node)| node.map(|v| (i, v)))
			.filter(|(_, node)| node.edges.len() == 1)
			.collect()
	}

	pub fn edges(&self) -> Vec<(usize, &Subedge<'_, T>)> {
		self.edges
			.iter()
			.map(Option::as_ref)
			.enumerate()
			.filter_map(|(i, edge)| edge.map(|v| (i, v)))
			.collect()
	}

	pub fn nodes(&self) -> Vec<(usize, &Subnode<'_, N>)> {
		self.nodes
			.iter()
			.map(Option::as_ref)
			.enumerate()
			.filter_map(|(i, node)| node.map(|v| (i, v)))
			.collect()
	}

	pub fn is_tree(&self) -> bool {
		let mut visited: HashSet<usize> = HashSet::new();
		let mut used_edges: HashSet<usize> = HashSet::new();
		let mut to_visit: VecDeque<usize> = VecDeque::new();

		{
			let mut first: Option<usize> = None;
			for (i, node) in self.nodes.iter().enumerate() {
				if let Some(_) = node {
					first = Some(i)
				}
			}
			match first {
				Some(first) => {
					to_visit.push_front(first);
					visited.insert(first);
				}
				_ => {
					return true;
				}
			}
		};

		while let Some(node) = to_visit.pop_back() {
			for edge in self.nodes[node].as_ref().unwrap().edges.iter() {
				if used_edges.contains(edge) {
					continue;
				}

				for neighbor in self.edges[*edge].as_ref().unwrap().nodes.iter() {
					if *neighbor == node {
						continue;
					}

					if visited.contains(neighbor) {
						return false;
					} else {
						visited.insert(*neighbor);
						to_visit.push_front(*neighbor);
					}
				}

				used_edges.insert(*edge);
			}
		}

		let count = self.nodes.iter().filter_map(|v| v.as_ref()).count();

		visited.len() == count
	}

	pub fn is_tree_with(&self, terminals: &[usize]) -> bool {
		if !self.is_tree() {
			return false;
		}

		for terminal in terminals {
			if let Some(Some(_)) = self.nodes.get(*terminal) {
				continue;
			}

			return false;
		}

		return true;
	}
}
