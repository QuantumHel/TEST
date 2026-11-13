use std::fmt::{self, Display, Formatter};

/// An undirected graph over caller-chosen `usize` node identifiers.
///
/// Identifiers are sparse and arbitrary — `7`, `1000`, and `3` may coexist
/// with nothing in between. Removing a node drops every edge touching it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Graph {
	adjacency: Vec<Vec<u64>>,
	active: Vec<u64>,
	node_type: Vec<char>,
	node_count: usize,
	blocks: usize,
	size: usize,
}

impl Graph {
	pub fn new(size: usize) -> Self {
		let blocks = size / 64 + 1;
		Graph {
			adjacency: vec![vec![0; blocks]; size],
			active: vec![0; blocks],
			node_type: vec![' '; size],
			node_count: 0,
			blocks,
			size,
		}
	}

	// ---- nodes ----------------------------------------------------------

	/// Inserts a node. Returns `false` if it was already present.
	pub fn add_node(&mut self, id: usize) -> bool {
		if self.active[id / 64] & (1 << (id % 64)) != 0 {
			return false;
		}
		self.active[id / 64] |= 1 << (id % 64);
		self.node_count += 1;
		true
	}

	/// Removes a node and every edge incident to it.
	/// Returns `false` if the node was not present.
	pub fn remove_node(&mut self, id: usize) -> bool {
		if self.active[id / 64] & (1 << (id % 64)) == 0 {
			return false;
		}
		self.active[id / 64] &= !(1 << (id % 64));
		self.node_count -= 1;
		let mask = !(1 << (id % 64));
		let block = id / 64;
		for i in 0..self.blocks {
			let mut bits = self.adjacency[id][i];
			let mut index = 0;
			while bits != 0 {
				let zero_bits = bits.trailing_zeros() as usize;
				index += zero_bits;
				self.adjacency[index + i * 64][block] &= mask;
				bits >>= zero_bits;
				bits >>= 1;
				index += 1;
			}
			self.adjacency[id][i] = 0;
		}
		true
	}

	pub fn has_node(&self, id: usize) -> bool {
		self.active[id / 64] & (1 << (id % 64)) != 0
	}

	pub fn node_count(&self) -> usize {
		self.node_count
	}

	/// Iterates node ids in arbitrary order.
	pub fn nodes(&self) -> impl Iterator<Item = usize> + '_ {
		self.active.iter().enumerate().flat_map(|(block, &word)| {
			let mut bits = word;
			std::iter::from_fn(move || {
				if bits == 0 {
					return None;
				}

				let offset = bits.trailing_zeros() as usize;
				bits &= bits - 1;
				Some(block * 64 + offset)
			})
		})
	}

	pub fn set_node_type(&mut self, id: usize, t: char) {
		self.node_type[id] = t;
	}

	pub fn get_node_type(&self, id: usize) -> char {
		self.node_type[id]
	}

	// ---- edges ----------------------------------------------------------

	/// Adds an undirected edge. Returns `false` if either endpoint is
	/// missing, if the edge already exists, or if `a == b` (no self-loops).
	pub fn add_edge(&mut self, a: usize, b: usize) -> bool {
		if a == b || !self.has_node(a) || !self.has_node(b) || self.has_edge(a, b) {
			return false;
		}
		self.adjacency[a][b / 64] |= 1 << (b % 64);
		self.adjacency[b][a / 64] |= 1 << (a % 64);
		true
	}

	/// Removes an edge. Returns `false` if it was not there.
	pub fn remove_edge(&mut self, a: usize, b: usize) -> bool {
		if !self.has_edge(a, b) {
			return false;
		}
		self.adjacency[a][b / 64] &= !(1 << (b % 64));
		self.adjacency[b][a / 64] &= !(1 << (a % 64));
		true
	}

	pub fn has_edge(&self, a: usize, b: usize) -> bool {
		self.adjacency[a][b / 64] & (1 << (b % 64)) != 0
	}

	/// Neighbors of `id`, or an empty iterator if the node is absent.
	pub fn neighbors(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
		self.adjacency[id]
			.iter()
			.enumerate()
			.flat_map(|(block, &word)| {
				let mut bits = word;
				std::iter::from_fn(move || {
					if bits == 0 {
						return None;
					}

					let offset = bits.trailing_zeros() as usize;
					bits &= bits - 1;
					Some(block * 64 + offset)
				})
			})
	}

	// ---- degrees --------------------------------------------------------

	/// Degree of `id`, or `None` if the node does not exist.
	///
	/// `None` and `Some(0)` mean different things: absent versus isolated.
	pub fn degree(&self, id: usize) -> Option<usize> {
		if self.active[id / 64] & (1 << (id % 64)) != 0 {
			let mut count = 0;
			for i in 0..self.blocks {
				count += self.adjacency[id][i].count_ones() as usize;
			}
			Some(count)
		} else {
			None
		}
	}

	pub fn non_cutting_nodes(&self) -> impl Iterator<Item = usize> + '_ {
		self.nodes().filter(move |&id| {
			let deg = self.degree(id).unwrap();
			if deg == 0 {
				return true;
			}
			let mut seen = vec![false; self.size];
			let mut stack = Vec::new();
			let mut count = 0;
			for neighbor in self.neighbors(id) {
				if !seen[neighbor] {
					count += 1;
					stack.push(neighbor);
					while let Some(node) = stack.pop() {
						for next in self.neighbors(node) {
							if next != id && !seen[next] {
								seen[next] = true;
								stack.push(next);
							}
						}
					}
				}
			}
			count <= 1
		})
	}

	pub fn min_degree(&self) -> Option<usize> {
		self.nodes().map(|id| self.degree(id).unwrap_or(0)).min()
	}

	pub fn min_degree_except(&self, except: usize) -> Option<usize> {
		self.nodes()
			.filter(|&id| id != except)
			.map(|id| self.degree(id).unwrap_or(0))
			.min()
	}

	pub fn non_cutting_nodes_having_min_degree(&self) -> impl Iterator<Item = usize> + '_ {
		self.non_cutting_nodes()
			.filter(move |&id| self.degree(id).unwrap_or(0) == self.min_degree().unwrap_or(0))
	}
}

impl Display for Graph {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		let mut result = String::new();
		for id in self.nodes() {
			result.push_str(&format!("{}({}): (", id, self.get_node_type(id)));
			for neighbor in self.neighbors(id) {
				result.push_str(&format!("{} ", neighbor));
			}
			result.push_str("), ");
		}
		formatter.write_str(&result)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// 10 — 20 — 30, plus isolated 99.
	fn path_graph() -> Graph {
		let mut g = Graph::new(100);
		for id in [10, 20, 30, 70, 80, 90, 99] {
			g.add_node(id);
		}
		g.add_edge(10, 20);
		g.add_edge(20, 30);
		g.add_edge(70, 80);
		g.add_edge(80, 90);
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
	fn removing_a_node_drops_its_edges2() {
		let mut g = path_graph();

		assert!(g.remove_node(80));
		assert!(!g.has_node(80));
		assert_eq!(g.degree(70), Some(0));
		assert_eq!(g.degree(90), Some(0));
		assert_eq!(g.degree(80), None, "absent, not isolated");
		assert!(!g.remove_node(80), "second removal is a no-op");
	}

	#[test]
	fn remove_edge_leaves_nodes_in_place() {
		let mut g = path_graph();
		assert!(g.remove_edge(10, 20));
		assert!(!g.remove_edge(10, 20));
		assert!(g.has_node(10));
		assert_eq!(g.degree(10), Some(0));
	}

	#[test]
	fn test_non_cutting_nodes() {
		let mut g = path_graph();
		g.add_node(11);
		g.add_node(12);
		g.add_edge(11, 12);
		g.add_node(13);
		g.add_edge(12, 13);
		g.add_node(14);
		g.add_edge(13, 14);
		g.add_edge(14, 11);

		let mut non_cutting: Vec<_> = g.non_cutting_nodes().collect();
		non_cutting.sort_unstable();
		assert_eq!(non_cutting, vec![10, 11, 12, 13, 14, 30, 70, 90, 99]);

		let mut non_cutting: Vec<_> = g.non_cutting_nodes_having_min_degree().collect();
		non_cutting.sort_unstable();
		assert_eq!(non_cutting, vec![99]);
		g.remove_node(99);
		let mut non_cutting: Vec<_> = g.non_cutting_nodes_having_min_degree().collect();
		non_cutting.sort_unstable();
		assert_eq!(non_cutting, vec![10, 30, 70, 90]);

		g.remove_node(11);

		let mut non_cutting: Vec<_> = g.non_cutting_nodes().collect();
		non_cutting.sort_unstable();
		assert_eq!(non_cutting, vec![10, 12, 14, 30, 70, 90]);
	}

	#[test]
	fn test_degrees() {
		let mult = 70;
		let mut g = Graph::new(10 + mult);
		g.add_node(1 + mult);
		assert!(g.min_degree_except(0 + mult) == Some(0));
		g.add_node(2 + mult);
		assert!(g.min_degree_except(0 + mult) == Some(0));
		g.add_edge(1 + mult, 2 + mult);
		assert!(g.min_degree_except(0 + mult) == Some(1));
		g.add_node(3 + mult);
		assert!(g.min_degree_except(0 + mult) == Some(0));
		g.add_edge(2 + mult, 3 + mult);
		assert!(g.min_degree_except(0 + mult) == Some(1));
		assert!(g.min_degree_except(1 + mult) == Some(1));
		g.add_node(4 + mult);
		assert!(g.min_degree_except(0 + mult) == Some(0));
		g.add_edge(3 + mult, 4 + mult);
		assert!(g.min_degree_except(0 + mult) == Some(1));
		assert!(g.min_degree_except(1 + mult) == Some(1));
		g.add_edge(2 + mult, 4 + mult);
		assert!(g.min_degree_except(0 + mult) == Some(1));
		assert!(g.min_degree_except(1 + mult) == Some(2));
		assert!(g.min_degree_except(2 + mult) == Some(1));
		g.add_edge(1 + mult, 4 + mult);
		assert!(g.min_degree_except(0 + mult) == Some(2));
		assert!(g.min_degree_except(4 + mult) == Some(2));
		assert!(g.min_degree_except(2 + mult) == Some(2));
		g.add_edge(1 + mult, 3 + mult);
		assert!(g.min_degree_except(0 + mult) == Some(3));
		assert!(g.min_degree_except(4 + mult) == Some(3));
		assert!(g.min_degree_except(2 + mult) == Some(3));
		g.remove_node(1 + mult);
		assert!(g.min_degree_except(0 + mult) == Some(2));
		g.remove_edge(2 + mult, 3 + mult);
		assert!(g.min_degree_except(0 + mult) == Some(1));
		assert!(g.min_degree_except(2 + mult) == Some(1));
	}
}
