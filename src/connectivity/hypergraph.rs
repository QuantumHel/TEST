#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HyperNodeIndex(pub(super) usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HyperEdgeIndex(pub(super) usize);

pub struct HyperNode<T> {
	pub data: T,
	pub edges: Vec<HyperEdgeIndex>,
}

pub struct HyperEdge {
	pub nodes: Vec<HyperNodeIndex>,
}

#[derive(Debug)]
pub struct IndexOutOfBoundsError;

pub struct HyperGraph<T> {
	pub(super) nodes: Vec<HyperNode<T>>,
	pub(super) edges: Vec<HyperEdge>,
}

impl<T> Default for HyperGraph<T> {
	fn default() -> Self {
		Self {
			nodes: Vec::new(),
			edges: Vec::new(),
		}
	}
}

impl<T> HyperGraph<T> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add_node(&mut self, data: T) -> HyperNodeIndex {
		let index = HyperNodeIndex(self.nodes.len());

		self.nodes.push(HyperNode {
			data,
			edges: Vec::new(),
		});

		index
	}

	pub fn add_edge<I>(&mut self, iterable: I) -> Result<HyperEdgeIndex, IndexOutOfBoundsError>
	where
		I: IntoIterator<Item = HyperNodeIndex>,
	{
		let index = HyperEdgeIndex(self.edges.len());
		let nodes: Vec<HyperNodeIndex> = iterable.into_iter().collect();
		for node in nodes.iter() {
			if node.0 >= self.nodes.len() {
				return Err(IndexOutOfBoundsError);
			}
		}

		for node in nodes.iter() {
			self.nodes[node.0].edges.push(index);
		}

		self.edges.push(HyperEdge { nodes });

		Ok(index)
	}

	pub fn get_node(&self, index: HyperNodeIndex) -> Option<&HyperNode<T>> {
		self.nodes.get(index.0)
	}

	pub fn get_edge(&self, index: HyperEdgeIndex) -> Option<&HyperEdge> {
		self.edges.get(index.0)
	}

	pub fn biggest_cluster(&self) -> usize {
		todo!()
	}
}
