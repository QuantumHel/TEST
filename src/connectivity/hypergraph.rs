use std::collections::BTreeSet;

pub type HyperNodeIndex = usize;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HyperEdgeIndex(pub(super) usize);

#[derive(Debug)]
pub struct HyperNode {
	pub edges: Vec<HyperEdgeIndex>,
}

#[derive(Debug)]

pub struct HyperEdge {
	pub nodes: Vec<HyperNodeIndex>,
}

#[derive(Debug)]
pub struct IndexOutOfBoundsError;

#[derive(Default, Debug)]
pub struct HyperGraph {
	pub(super) nodes: Vec<HyperNode>,
	pub(super) edges: Vec<HyperEdge>,
}

impl HyperGraph {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add_node(&mut self) -> HyperNodeIndex {
		let index = self.nodes.len();

		self.nodes.push(HyperNode { edges: Vec::new() });

		index
	}

	pub fn add_edge<I>(&mut self, iterable: I) -> Result<HyperEdgeIndex, IndexOutOfBoundsError>
	where
		I: IntoIterator<Item = HyperNodeIndex>,
	{
		let index = HyperEdgeIndex(self.edges.len());
		let nodes: Vec<HyperNodeIndex> = iterable.into_iter().collect();
		for node in nodes.iter() {
			if *node >= self.nodes.len() {
				return Err(IndexOutOfBoundsError);
			}
		}

		for node in nodes.iter() {
			self.nodes[*node].edges.push(index);
		}

		self.edges.push(HyperEdge { nodes });

		Ok(index)
	}

	pub fn fully_connected(&self) -> bool {
		if self.nodes.is_empty() {
			return true;
		}
		let mut visited: BTreeSet<usize> = BTreeSet::new();
		let mut to_visit: Vec<usize> = vec![0];
		while !to_visit.is_empty() {
			let gonna_visit: Vec<_> = std::mem::take(&mut to_visit);
			for node in gonna_visit.into_iter() {
				if visited.contains(&node) {
					continue;
				}
				visited.insert(node);

				for edge in self.nodes.get(node).unwrap().edges.iter() {
					let edge = self.get_edge(*edge).unwrap();
					for node in edge.nodes.iter() {
						to_visit.push(*node);
					}
				}
			}
		}

		visited.len() == self.nodes.len()
	}

	pub fn get_node(&self, index: HyperNodeIndex) -> Option<&HyperNode> {
		self.nodes.get(index)
	}

	pub fn get_edge(&self, index: HyperEdgeIndex) -> Option<&HyperEdge> {
		self.edges.get(index.0)
	}
}
