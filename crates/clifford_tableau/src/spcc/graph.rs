use std::collections::{HashMap, HashSet, VecDeque};

/// An undirected graph over caller-chosen `usize` node identifiers.
///
/// Identifiers are sparse and arbitrary — `7`, `1000`, and `3` may coexist
/// with nothing in between. Removing a node drops every edge touching it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Graph {
	adj: HashMap<usize, HashSet<usize>>,
	node_type: HashMap<usize, char>,
}

impl Graph {
	pub fn new() -> Self {
		Graph {
			adj: HashMap::new(),
			node_type: HashMap::new(),
		}
	}

	pub fn with_capacity(n: usize) -> Self {
		Graph {
			adj: HashMap::with_capacity(n),
			node_type: HashMap::with_capacity(n),
		}
	}

	// ---- nodes ----------------------------------------------------------

	/// Inserts a node. Returns `false` if it was already present.
	pub fn add_node(&mut self, id: usize) -> bool {
		self.adj.insert(id, HashSet::new()).is_none() && self.node_type.insert(id, ' ').is_none()
	}

	/// Removes a node and every edge incident to it.
	/// Returns `false` if the node was not present.
	pub fn remove_node(&mut self, id: usize) -> bool {
		let Some(neighbors) = self.adj.remove(&id) else {
			return false;
		};
		for n in neighbors {
			if let Some(set) = self.adj.get_mut(&n) {
				set.remove(&id);
			}
		}
		self.node_type.remove(&id);
		true
	}

	pub fn has_node(&self, id: usize) -> bool {
		self.adj.contains_key(&id)
	}

	pub fn node_count(&self) -> usize {
		self.adj.len()
	}

	/// Iterates node ids in arbitrary order.
	pub fn nodes(&self) -> impl Iterator<Item = usize> + '_ {
		self.adj.keys().copied()
	}

	/// Node ids in ascending order.
	pub fn sorted_nodes(&self) -> Vec<usize> {
		let mut v: Vec<usize> = self.nodes().collect();
		v.sort_unstable();
		v
	}

	pub fn set_node_type(&mut self, id: usize, t: char) {
		self.node_type.insert(id, t);
	}

	pub fn get_node_type(&self, id: usize) -> Option<char> {
		self.node_type.get(&id).copied()
	}

	// ---- edges ----------------------------------------------------------

	/// Adds an undirected edge. Returns `false` if either endpoint is
	/// missing, if the edge already exists, or if `a == b` (no self-loops).
	pub fn add_edge(&mut self, a: usize, b: usize) -> bool {
		if a == b || !self.has_node(a) || !self.has_node(b) {
			return false;
		}
		let inserted = self.adj.get_mut(&a).unwrap().insert(b);
		self.adj.get_mut(&b).unwrap().insert(a);
		inserted
	}

	/// Adds an edge, creating either endpoint if it does not exist yet.
	pub fn add_edge_creating(&mut self, a: usize, b: usize) -> bool {
		self.add_node(a);
		self.add_node(b);
		self.add_edge(a, b)
	}

	/// Removes an edge. Returns `false` if it was not there.
	pub fn remove_edge(&mut self, a: usize, b: usize) -> bool {
		let removed = self.adj.get_mut(&a).map_or(false, |s| s.remove(&b));
		if let Some(set) = self.adj.get_mut(&b) {
			set.remove(&a);
		}
		removed
	}

	pub fn has_edge(&self, a: usize, b: usize) -> bool {
		self.adj.get(&a).map_or(false, |s| s.contains(&b))
	}

	pub fn edge_count(&self) -> usize {
		self.adj.values().map(|s| s.len()).sum::<usize>() / 2
	}

	/// Each undirected edge once, as `(min, max)`, in ascending order.
	pub fn sorted_edges(&self) -> Vec<(usize, usize)> {
		let mut v: Vec<(usize, usize)> = self
			.adj
			.iter()
			.flat_map(|(&a, ns)| ns.iter().map(move |&b| (a.min(b), a.max(b))))
			.collect();
		v.sort_unstable();
		v.dedup();
		v
	}

	/// Neighbors of `id`, or an empty iterator if the node is absent.
	pub fn neighbors(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
		self.adj
			.get(&id)
			.into_iter()
			.flat_map(|s| s.iter().copied())
	}

	// ---- degrees --------------------------------------------------------

	/// Degree of `id`, or `None` if the node does not exist.
	///
	/// `None` and `Some(0)` mean different things: absent versus isolated.
	pub fn degree(&self, id: usize) -> Option<usize> {
		self.adj.get(&id).map(|s| s.len())
	}

	/// Nodes of degree exactly 1, in ascending order.
	pub fn leaves(&self) -> Vec<usize> {
		let mut v: Vec<usize> = self
			.adj
			.iter()
			.filter(|(_, ns)| ns.len() == 1)
			.map(|(&id, _)| id)
			.collect();
		v.sort_unstable();
		v
	}

	/// Nodes of degree exactly 0, in ascending order.
	pub fn isolated(&self) -> Vec<usize> {
		let mut v: Vec<usize> = self
			.adj
			.iter()
			.filter(|(_, ns)| ns.is_empty())
			.map(|(&id, _)| id)
			.collect();
		v.sort_unstable();
		v
	}

	// ---- shortest paths -------------------------------------------------

	/// Hop counts from `src` to every node reachable from it, including
	/// `src` itself at distance 0. Unreachable nodes are omitted.
	///
	/// Returns `None` if `src` does not exist.
	pub fn distances_from(&self, src: usize) -> Option<HashMap<usize, usize>> {
		if !self.has_node(src) {
			return None;
		}
		let mut dist = HashMap::new();
		dist.insert(src, 0usize);
		let mut queue = VecDeque::new();
		queue.push_back(src);

		while let Some(u) = queue.pop_front() {
			let d = dist[&u];
			for v in self.neighbors(u) {
				if !dist.contains_key(&v) {
					dist.insert(v, d + 1);
					queue.push_back(v);
				}
			}
		}
		Some(dist)
	}

	/// All-pairs shortest path lengths, by breadth-first search from every
	/// node. Unreachable pairs are absent from the inner map rather than
	/// stored as infinity.
	///
	/// Runs in O(V·(V+E)) time.
	pub fn all_pairs_shortest_paths(&self) -> HashMap<usize, HashMap<usize, usize>> {
		self.nodes()
			.map(|src| (src, self.distances_from(src).unwrap()))
			.collect()
	}

	/// One shortest path from `a` to `b` inclusive, or `None` if either
	/// node is missing or no path exists. `a == b` gives `vec![a]`.
	pub fn shortest_path(&self, a: usize, b: usize) -> Option<Vec<usize>> {
		if !self.has_node(a) || !self.has_node(b) {
			return None;
		}
		if a == b {
			return Some(vec![a]);
		}

		let mut prev: HashMap<usize, usize> = HashMap::new();
		let mut seen: HashSet<usize> = HashSet::from([a]);
		let mut queue = VecDeque::from([a]);

		while let Some(u) = queue.pop_front() {
			for v in self.neighbors(u) {
				if seen.insert(v) {
					prev.insert(v, u);
					if v == b {
						let mut path = vec![b];
						let mut cur = b;
						while let Some(&p) = prev.get(&cur) {
							path.push(p);
							cur = p;
						}
						path.reverse();
						return Some(path);
					}
					queue.push_back(v);
				}
			}
		}
		None
	}

	/// True when every node is reachable from every other. The empty graph
	/// counts as connected.
	pub fn is_connected(&self) -> bool {
		match self.nodes().next() {
			None => true,
			Some(start) => self.distances_from(start).unwrap().len() == self.node_count(),
		}
	}
}

impl FromIterator<(usize, usize)> for Graph {
	/// Builds a graph from an edge list, creating endpoints as needed.
	fn from_iter<I: IntoIterator<Item = (usize, usize)>>(iter: I) -> Self {
		let mut g = Graph::new();
		for (a, b) in iter {
			g.add_edge_creating(a, b);
		}
		g
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// 10 — 20 — 30, plus isolated 99.
	fn path_graph() -> Graph {
		let mut g = Graph::new();
		for id in [10, 20, 30, 99] {
			g.add_node(id);
		}
		g.add_edge(10, 20);
		g.add_edge(20, 30);
		g
	}

	#[test]
	fn add_node_is_idempotent() {
		let mut g = Graph::new();
		assert!(g.add_node(7));
		assert!(!g.add_node(7));
		assert_eq!(g.node_count(), 1);
	}

	#[test]
	fn add_edge_rejects_missing_nodes_and_self_loops() {
		let mut g = Graph::new();
		g.add_node(1);
		assert!(!g.add_edge(1, 2), "node 2 does not exist");
		assert!(!g.add_edge(1, 1), "self-loops are rejected");
		g.add_node(2);
		assert!(g.add_edge(1, 2));
		assert!(!g.add_edge(2, 1), "already present, either orientation");
		assert_eq!(g.edge_count(), 1);
	}

	#[test]
	fn removing_a_node_drops_its_edges() {
		let mut g = path_graph();
		assert_eq!(g.edge_count(), 2);

		assert!(g.remove_node(20));
		assert!(!g.has_node(20));
		assert_eq!(g.edge_count(), 0, "both incident edges went with it");
		assert_eq!(g.degree(10), Some(0));
		assert_eq!(g.degree(30), Some(0));
		assert_eq!(g.degree(20), None, "absent, not isolated");
		assert!(!g.remove_node(20), "second removal is a no-op");
	}

	#[test]
	fn degree_and_leaves() {
		let g = path_graph();
		assert_eq!(g.degree(10), Some(1));
		assert_eq!(g.degree(20), Some(2));
		assert_eq!(g.degree(99), Some(0));
		assert_eq!(g.degree(12345), None);

		assert_eq!(g.leaves(), vec![10, 30], "isolated node is not a leaf");
		assert_eq!(g.isolated(), vec![99]);
	}

	#[test]
	fn leaves_update_after_removal() {
		let mut g = path_graph();
		g.remove_node(10);
		assert_eq!(g.leaves(), vec![20, 30], "20 dropped to degree 1");
	}

	#[test]
	fn all_pairs_on_a_disconnected_graph() {
		let g = path_graph();
		let apsp = g.all_pairs_shortest_paths();

		assert_eq!(apsp[&10][&10], 0);
		assert_eq!(apsp[&10][&20], 1);
		assert_eq!(apsp[&10][&30], 2);
		assert_eq!(apsp[&30][&10], 2, "symmetric");
		assert!(!apsp[&10].contains_key(&99), "unreachable is absent");
		assert_eq!(apsp[&99].len(), 1, "isolated node reaches only itself");
		assert!(!g.is_connected());
	}

	#[test]
	fn shortest_path_picks_the_short_side_of_a_cycle() {
		// 0—1—2—3—4—0 : a 5-cycle.
		let g: Graph = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)]
			.into_iter()
			.collect();

		assert_eq!(g.shortest_path(0, 3), Some(vec![0, 4, 3]));
		assert_eq!(g.shortest_path(0, 2), Some(vec![0, 1, 2]));
		assert_eq!(g.shortest_path(2, 2), Some(vec![2]));
		assert_eq!(g.shortest_path(0, 77), None);
		assert!(g.is_connected());
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
	fn sorted_edges_are_deduplicated_and_normalized() {
		let g: Graph = [(5, 1), (1, 9), (9, 5)].into_iter().collect();
		assert_eq!(g.sorted_edges(), vec![(1, 5), (1, 9), (5, 9)]);
	}
}
