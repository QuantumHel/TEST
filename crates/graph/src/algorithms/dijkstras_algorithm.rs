use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::{Edge, Graph, Node};

#[derive(Copy, Clone, PartialEq)]
struct QueueItem {
	cost: f64,
	storage_index: usize,
}

// Cost is always a valid number
impl Eq for QueueItem {}

impl Ord for QueueItem {
	fn cmp(&self, other: &Self) -> Ordering {
		// reverse ordering for min heap
		other
			.cost
			.total_cmp(&self.cost)
			.then_with(|| self.storage_index.cmp(&other.storage_index))
	}
}

impl PartialOrd for QueueItem {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

/// Dijkstra's shortest path algorithm to all nodes.
///
/// Calculates the distance to all reachable nodes
pub fn dijkstras_shortest_paths<G: Graph<N, E> + ?Sized, N: Node, E: Edge>(
	graph: &G,
	start_index: usize,
) -> Vec<Option<f64>> {
	let mut dist: Vec<Option<f64>> = vec![None; graph.node_storage_size()];
	let mut heap = BinaryHeap::new();

	if start_index >= graph.node_storage_size() {
		return dist;
	}

	dist[start_index] = Some(0.);
	heap.push(QueueItem {
		cost: 0.,
		storage_index: start_index,
	});

	while let Some(QueueItem {
		cost,
		storage_index,
	}) = heap.pop()
	{
		if let Some(&Some(dist)) = dist.get(storage_index)
			&& cost > dist
		{
			continue;
		}

		for edge in graph.get_node(storage_index).unwrap().edges() {
			let edge = graph.get_edge(edge).unwrap();
			let weight = edge.weight();
			for node in edge.nodes() {
				let new_cost = cost + weight;
				// need to match on option

				if dist[node].is_none() || dist[node].unwrap() > new_cost {
					dist[node] = Some(new_cost);

					heap.push(QueueItem {
						cost: new_cost,
						storage_index: node,
					});
				}
			}
		}
	}

	dist
}
