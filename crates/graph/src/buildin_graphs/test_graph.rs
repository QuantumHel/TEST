use crate::{Edge, Graph, Node};

#[derive(Default, Debug)]
pub struct TestEdge {
	nodes: Vec<usize>,
	weight: f64,
}

impl Edge for TestEdge {
	fn nodes(&self) -> Vec<usize> {
		self.nodes.clone()
	}

	fn weight(&self) -> f64 {
		self.weight
	}
}

#[derive(Default, Debug)]
pub struct TestNode {
	edges: Vec<usize>,
}

impl Node for TestNode {
	fn edges(&self) -> Vec<usize> {
		self.edges.clone()
	}
}

#[derive(Default, Debug)]
pub struct TestGraph {
	nodes: Vec<TestNode>,
	edges: Vec<TestEdge>,
}

impl TestGraph {
	pub fn add_edge(&mut self, weight: f64, nodes: &[usize]) {
		let Some(&n) = nodes.iter().max() else {
			return;
		};
		while self.nodes.len() <= n {
			self.nodes.push(TestNode::default());
		}

		let edge = self.edges.len();
		self.edges.push(TestEdge {
			nodes: nodes.to_vec(),
			weight,
		});

		for &node in nodes {
			self.nodes.get_mut(node).unwrap().edges.push(edge);
		}
	}
}

impl Graph<TestNode, TestEdge> for TestGraph {
	fn edge_storage_size(&self) -> usize {
		self.edges.len()
	}

	fn node_storage_size(&self) -> usize {
		self.nodes.len()
	}

	fn get_edge(&self, index: usize) -> Option<&TestEdge> {
		self.edges.get(index)
	}

	fn get_node(&self, index: usize) -> Option<&TestNode> {
		self.nodes.get(index)
	}

	fn enumerate_edges<'a>(&'a self) -> impl Iterator<Item = (usize, &'a TestEdge)>
	where
		TestEdge: 'a,
	{
		self.edges.iter().enumerate()
	}

	fn enumerate_nodes<'a>(&'a self) -> impl Iterator<Item = (usize, &'a TestNode)>
	where
		TestNode: 'a,
	{
		self.nodes.iter().enumerate()
	}

	fn iter_edges<'a>(&'a self) -> impl Iterator<Item = &'a TestEdge>
	where
		TestEdge: 'a,
	{
		self.edges.iter()
	}

	fn iter_nodes<'a>(&'a self) -> impl Iterator<Item = &'a TestNode>
	where
		TestNode: 'a,
	{
		self.nodes.iter()
	}
}
