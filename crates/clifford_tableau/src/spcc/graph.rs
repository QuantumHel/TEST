/// An undirected graph over caller-chosen `usize` node identifiers.
///
/// Identifiers are sparse and arbitrary — `7`, `1000`, and `3` may coexist
/// with nothing in between. Removing a node drops every edge touching it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Graph {
	adjacency: Vec<Vec<bool>>,
	active: Vec<bool>,
	node_type: Vec<char>,
	node_count: usize,
	degree: Vec<usize>,
}

impl Graph {
	pub fn new(size: usize) -> Self {
		Graph {
			adjacency: vec![vec![false; size]; size],
			active: vec![false; size],
			node_type: vec![' '; size],
			node_count: 0,
			degree: vec![0; size],
		}
	}

	// ---- nodes ----------------------------------------------------------

	/// Inserts a node. Returns `false` if it was already present.
	pub fn add_node(&mut self, id: usize) -> bool {
		if self.active[id] {
			return false;
		}
		self.active[id] = true;
		self.degree[id] = 0;
		self.node_count += 1;
		true
	}

	/// Removes a node and every edge incident to it.
	/// Returns `false` if the node was not present.
	pub fn remove_node(&mut self, id: usize) -> bool {
		if !self.active[id] {
			return false;
		}
		self.active[id] = false;
		self.node_count -= 1;
		for i in 0..self.adjacency.len() {
			if self.adjacency[id][i] {
				self.adjacency[id][i] = false;
				self.adjacency[i][id] = false;
				self.degree[i] -= 1;
			}
		}
		self.degree[id] = 0;
		true
	}

	pub fn has_node(&self, id: usize) -> bool {
		self.active[id]
	}

	pub fn node_count(&self) -> usize {
		self.node_count
	}

	/// Iterates node ids in arbitrary order.
	pub fn nodes(&self) -> impl Iterator<Item = usize> + '_ {
		self.active
			.iter()
			.enumerate()
			.filter(|(_, active)| **active)
			.map(|(id, _)| id)
	}

	pub fn set_node_type(&mut self, id: usize, t: char) {
		self.node_type[id] = t;
	}

	pub fn get_node_type(&self, id: usize) -> Option<char> {
		if self.active[id] {
			Some(self.node_type[id])
		} else {
			None
		}
	}

	// ---- edges ----------------------------------------------------------

	/// Adds an undirected edge. Returns `false` if either endpoint is
	/// missing, if the edge already exists, or if `a == b` (no self-loops).
	pub fn add_edge(&mut self, a: usize, b: usize) -> bool {
		if a == b || !self.active[a] || !self.active[b] || self.adjacency[a][b] {
			return false;
		}
		self.adjacency[a][b] = true;
		self.adjacency[b][a] = true;
		self.degree[a] += 1;
		self.degree[b] += 1;
		true
	}

	/// Removes an edge. Returns `false` if it was not there.
	pub fn remove_edge(&mut self, a: usize, b: usize) -> bool {
		if !self.adjacency[a][b] {
			return false;
		}
		self.adjacency[a][b] = false;
		self.adjacency[b][a] = false;
		self.degree[a] -= 1;
		self.degree[b] -= 1;
		true
	}

	pub fn has_edge(&self, a: usize, b: usize) -> bool {
		self.adjacency[a][b]
	}

	/// Neighbors of `id`, or an empty iterator if the node is absent.
	pub fn neighbors(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
		self.adjacency[id]
			.iter()
			.enumerate()
			.filter(|(_, active)| **active)
			.map(|(i, _)| i)
	}

	// ---- degrees --------------------------------------------------------

	/// Degree of `id`, or `None` if the node does not exist.
	///
	/// `None` and `Some(0)` mean different things: absent versus isolated.
	pub fn degree(&self, id: usize) -> Option<usize> {
		if self.active[id] {
			Some(self.degree[id])
		} else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// 10 — 20 — 30, plus isolated 99.
	fn path_graph() -> Graph {
		let mut g = Graph::new(100);
		for id in [10, 20, 30, 99] {
			g.add_node(id);
		}
		g.add_edge(10, 20);
		g.add_edge(20, 30);
		g
	}

	#[test]
	fn add_node_is_idempotent() {
		let mut g = Graph::new(8);
		assert!(g.add_node(7));
		assert!(!g.add_node(7));
		assert_eq!(g.node_count(), 1);
	}

	#[test]
	fn add_edge_rejects_missing_nodes_and_self_loops() {
		let mut g = Graph::new(3);
		g.add_node(1);
		assert!(!g.add_edge(1, 2), "node 2 does not exist");
		assert!(!g.add_edge(1, 1), "self-loops are rejected");
		g.add_node(2);
		assert!(g.add_edge(1, 2));
		assert!(!g.add_edge(2, 1), "already present, either orientation");
	}

	#[test]
	fn removing_a_node_drops_its_edges() {
		let mut g = path_graph();

		assert!(g.remove_node(20));
		assert!(!g.has_node(20));
		assert_eq!(g.degree(10), Some(0));
		assert_eq!(g.degree(30), Some(0));
		assert_eq!(g.degree(20), None, "absent, not isolated");
		assert!(!g.remove_node(20), "second removal is a no-op");
	}

	#[test]
	fn remove_edge_leaves_nodes_in_place() {
		let mut g = path_graph();
		assert!(g.remove_edge(10, 20));
		assert!(!g.remove_edge(10, 20));
		assert!(g.has_node(10));
		assert_eq!(g.degree(10), Some(0));
	}
}
