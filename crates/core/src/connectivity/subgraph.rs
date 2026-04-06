use std::{
	collections::{HashSet, VecDeque},
	usize,
};

use crate::connectivity::{Connectivity, ConnectivityEdge};

#[derive(Debug)]
pub struct Subedge<'a, T: ConnectivityEdge> {
	pub original: &'a T,
	pub(super) nodes: Vec<usize>,
}

impl<'a, T: ConnectivityEdge> Subedge<'a, T> {
	pub fn nodes(&self) -> &[usize] {
		&self.nodes
	}
}

#[derive(Debug)]
pub struct Subnode<'a> {
	pub original: &'a Vec<usize>,
	pub(super) edges: Vec<usize>,
}

impl<'a> Subnode<'a> {
	pub fn edges(&self) -> &[usize] {
		&self.edges
	}
}

#[derive(Debug)]
pub struct Subgraph<'a, T: ConnectivityEdge> {
	/// Indexes for the edges in the original graph
	pub(super) edges: Vec<Option<Subedge<'a, T>>>,
	/// Indexes for the qubits in the original graph
	pub(super) nodes: Vec<Option<Subnode<'a>>>,
}

impl<'a, T: ConnectivityEdge> Subgraph<'a, T> {
	pub(super) fn empty(connectivity: &'a Connectivity<T>) -> Self {
		let mut edges: Vec<Option<Subedge<'a, T>>> = Vec::with_capacity(connectivity.edges.len());
		edges.resize_with(connectivity.edges.len(), || None);
		let mut nodes: Vec<Option<Subnode<'a>>> = Vec::with_capacity(connectivity.nodes.len());
		nodes.resize_with(connectivity.nodes.len(), || None);

		Self { edges, nodes }
	}

	pub fn remove_node(&mut self, nodes: usize) {
		if let Some(target) = self.nodes.get_mut(nodes).map(|a| a.take()).flatten() {
			for edge_index in target.edges {
				let edge = self.edges[edge_index].as_mut().unwrap();
				let index = edge.nodes.iter().find(|a| **a == nodes).unwrap();
				edge.nodes.swap_remove(*index);
				if edge.nodes.is_empty() {
					self.edges[edge_index] = None;
				}
			}
		}
	}

	pub fn remove_edge(&mut self, edge: usize) {
		if let Some(target) = self.edges.get_mut(edge).map(|a| a.take()).flatten() {
			for qubit_index in target.nodes {
				let qubit = self.nodes[qubit_index].as_mut().unwrap();
				let index = qubit.edges.iter().find(|a| **a == edge).unwrap();
				qubit.edges.swap_remove(*index);
				if qubit.edges.is_empty() {
					self.nodes[qubit_index] = None;
				}
			}
		}
	}

	/// qubits that belong to only one edge
	pub fn leaf_nodes(&self) -> Vec<(usize, &Subnode<'_>)> {
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

	pub fn nodes(&self) -> Vec<(usize, &Subnode<'_>)> {
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
