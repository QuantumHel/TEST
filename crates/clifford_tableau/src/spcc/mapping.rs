use std::{
	collections::{HashMap, HashSet, VecDeque},
	vec,
};

use super::ct_fast::CTFast;
use graph::GraphExt;
use pauli::PauliString;

use crate::CliffordTableau;
use crate::spcc::graph::Graph;

use core::connectivity::{Connectivity, TwoQubitEdge};
use core::qubit_map::PartialQubitMap;

/// Selects a logical-to-physical mapping using the pair-mapping heuristic.
pub fn find_mapping(tableau: &CTFast, root: usize, tree: &Graph) -> PartialQubitMap {
	let num_qubits = tableau.size();
	let mut mapped_logical_qubits = HashSet::new();
	let mut mapped_physical_qubits = HashSet::new();
	let mut mapping = vec![usize::MAX; num_qubits];
	let mut mapping_reverse: HashMap<usize, usize> = HashMap::new();

	let weights = edges_by_i_index(tableau);

	// Map the heaviest logical qubit to the root of the tree
	let heaviest_logical_qubit = find_heaviest_logical_qubit(tableau);
	mapped_logical_qubits.insert(heaviest_logical_qubit);
	mapped_physical_qubits.insert(root);
	mapping[heaviest_logical_qubit] = root;
	mapping_reverse.insert(root, heaviest_logical_qubit);

	// Find all physical qubits which are children of the root physical qubit (options for next mapping)
	let mut next_physical_qubits = HashSet::new();
	for neighbor in tree.neighbors(root) {
		next_physical_qubits.insert((root, neighbor));
	}

	while mapped_logical_qubits.len() < num_qubits {
		// For all options of next physical qubits,
		// find the logical qubit which has the heaviest weight to the already mapped logical qubits
		let mut max_weight: usize = usize::MAX;
		let mut max_logical_qubit_pair: (usize, usize) = (usize::MAX, usize::MAX);
		let mut max_physical_qubit_pair: (usize, usize) = (usize::MAX, usize::MAX);

		for &(mpq, nmpq) in &next_physical_qubits {
			let mlq = mapping_reverse[&mpq];
			for (nmlq, weights_for_logical_qubit) in weights.iter().enumerate() {
				if mapped_logical_qubits.contains(&nmlq) {
					continue;
				}
				let weight = weights_for_logical_qubit[mlq];
				if max_weight == usize::MAX || weight > max_weight {
					max_weight = weight;
					max_logical_qubit_pair = (mlq, nmlq);
					max_physical_qubit_pair = (mpq, nmpq);
				}
			}
		}
		// Make mapping
		mapped_logical_qubits.insert(max_logical_qubit_pair.1);
		mapped_physical_qubits.insert(max_physical_qubit_pair.1);
		mapping[max_logical_qubit_pair.1] = max_physical_qubit_pair.1;
		mapping_reverse.insert(max_physical_qubit_pair.1, max_logical_qubit_pair.1);
		next_physical_qubits.remove(&max_physical_qubit_pair);

		for q in tree.neighbors(max_physical_qubit_pair.1) {
			if !mapped_physical_qubits.contains(&q) {
				next_physical_qubits.insert((max_physical_qubit_pair.1, q));
			}
		}
	}

	let mut partial_mapping = PartialQubitMap::default();
	for (from, to) in mapping.into_iter().enumerate() {
		partial_mapping.map_qubit(from, to).unwrap();
	}
	partial_mapping
}

/// Counts how many gadgets contain a non-identity Pauli at both qubits.
pub fn edges_by_i_index(tableau: &CTFast) -> Vec<Vec<usize>> {
	let num_qubits = tableau.size();
	let mut edges: Vec<Vec<usize>> = vec![vec![0; num_qubits]; num_qubits];

	for (q1, edges_row) in edges.iter_mut().enumerate().take(num_qubits) {
		for (q2, edge) in edges_row.iter_mut().enumerate().take(num_qubits) {
			let mut count: usize = 0;
			for row in 0..num_qubits {
				if tableau.get_x_row_col(row, q1) != 0 && tableau.get_x_row_col(row, q2) != 0 {
					count += 1;
				}
				if tableau.get_z_row_col(row, q1) != 0 && tableau.get_z_row_col(row, q2) != 0 {
					count += 1;
				}
			}
			*edge = count;
		}
	}
	edges
}

pub fn find_heaviest_logical_qubit(tableau: &CTFast) -> usize {
	let num_qubits = tableau.size();
	let mut max_conn = 0;
	let mut heaviest_logical_qubit = 0;
	for row in 0..num_qubits {
		if tableau.get_x_row_col(row, 0) != 0 {
			max_conn += 1;
		}
		if tableau.get_z_row_col(row, 0) != 0 {
			max_conn += 1;
		}
	}
	for qubit_mapping in 1..num_qubits {
		let mut conn = 0;
		for row in 0..num_qubits {
			if tableau.get_x_row_col(row, qubit_mapping) != 0 {
				conn += 1;
			}
			if tableau.get_z_row_col(row, qubit_mapping) != 0 {
				conn += 1;
			}
		}
		if conn > max_conn {
			max_conn = conn;
			heaviest_logical_qubit = qubit_mapping;
		}
	}
	heaviest_logical_qubit
}

pub fn check_second_hop(
	node: usize,
	device: &Connectivity<TwoQubitEdge>,
	mapped_physical_qubits: &HashSet<usize>,
	available_nodes: &HashSet<usize>,
) -> usize {
	let mut count: usize = 0;
	for neighbor in device.neighbors(node) {
		if !mapped_physical_qubits.contains(&neighbor) && !available_nodes.contains(&neighbor) {
			count += 1;
		}
	}
	count
}

/// Constructs the physical-qubit tree and its initial logical-to-physical map.
pub fn construct_tree(
	device: &Connectivity<TwoQubitEdge>,
	num_logical_qubits: usize,
) -> Result<(usize, Graph), String> {
	if num_logical_qubits == 0 || num_logical_qubits > device.node_count() {
		return Err("invalid logical-qubit count".to_string());
	}

	let mut tree = Graph::new();
	let mut mapped_physical_qubits = HashSet::new();

	let root = center_physical_qubit(device)?;
	tree.add_node(root);
	let mut next_logical_qubit = 1;
	mapped_physical_qubits.insert(root);
	let mut available_edges = HashSet::new();
	let mut available_nodes = HashSet::new();
	for neighbor in device.neighbors(root) {
		if !mapped_physical_qubits.contains(&neighbor) {
			available_edges.insert((root, neighbor));
			available_nodes.insert(neighbor);
		}
	}

	while (num_logical_qubits - next_logical_qubit) > available_edges.len() {
		let mut best_edge = None;
		let mut best_score = usize::MAX;

		for &(parent, child) in &available_edges {
			let mut score = 0;
			for neighbor in device.neighbors(child) {
				if mapped_physical_qubits.contains(&neighbor) {
					continue;
				}
				if available_nodes.contains(&neighbor) {
					continue;
				}
				let second_hop_count =
					check_second_hop(neighbor, device, &mapped_physical_qubits, &available_nodes);
				score += second_hop_count;
			}

			if best_edge.is_none() || score > best_score {
				best_edge = Some((parent, child));
				best_score = score;
			}
		}

		let Some((parent, child)) = best_edge else {
			return Err("topology cannot accommodate all logical qubits".to_string());
		};

		// Make mapping
		next_logical_qubit += 1;
		mapped_physical_qubits.insert(child);
		tree.add_node(child);
		tree.add_edge(parent, child);
		available_edges.remove(&(parent, child));
		available_nodes.remove(&child);

		// Update available edges and nodes
		for neighbor in device.neighbors(child) {
			if mapped_physical_qubits.contains(&neighbor) {
				continue;
			}

			let already_available = available_edges
				.iter()
				.any(|&(_, edge_child)| edge_child == neighbor);

			if !already_available {
				available_edges.insert((child, neighbor));
				available_nodes.insert(neighbor);
			}
		}
	}

	// Map remaining logical qubits to available edges
	while next_logical_qubit < num_logical_qubits {
		let Some(&(parent, child)) = available_edges.iter().next() else {
			return Err("not enough available topology edges".to_string());
		};

		available_edges.remove(&(parent, child));
		available_nodes.remove(&child);
		next_logical_qubit += 1;
		tree.add_node(child);
		tree.add_edge(parent, child);
	}

	Ok((root, tree))
}

pub fn get_available_edges_of_node(
	node: usize,
	full_topo: &Graph,
	mapped_physical_qubits: &HashSet<usize>,
) -> (HashSet<(usize, usize)>, HashSet<usize>) {
	let mut available_edges = HashSet::new();
	let mut available_nodes = HashSet::new();

	for neighbor in full_topo.neighbors(node) {
		if !mapped_physical_qubits.contains(&neighbor) {
			available_edges.insert((node, neighbor));
			available_nodes.insert(neighbor);
		}
	}

	(available_edges, available_nodes)
}

/// Finds a maximum-degree vertex whose nearest minimum-degree vertex is farthest away.
pub fn center_physical_qubit(device: &Connectivity<TwoQubitEdge>) -> Result<usize, String> {
	let num_physical_qubits = device.node_count();
	if num_physical_qubits == 0 {
		return Err("topology must contain at least one node".to_string());
	}
	let distances = all_pairs_shortest_paths(device);

	let mut max_degree: usize = 0;
	let mut min_degree: usize = usize::MAX;
	let mut max_degree_qubits: Vec<usize> = Vec::new();
	let mut min_degree_qubits: Vec<usize> = Vec::new();

	for qubit in 0..num_physical_qubits {
		let degree = device.neighbors(qubit).len();
		if degree > max_degree {
			max_degree = degree;
			max_degree_qubits.clear();
			max_degree_qubits.push(qubit);
		} else if degree == max_degree {
			max_degree_qubits.push(qubit);
		}

		if degree < min_degree {
			min_degree = degree;
			min_degree_qubits.clear();
			min_degree_qubits.push(qubit);
		} else if degree == min_degree {
			min_degree_qubits.push(qubit);
		}
	}

	let mut max_distance_to_edge: usize = usize::MAX;
	let mut max_distance_qubit: usize = usize::MAX;

	for q1 in max_degree_qubits.iter() {
		let mut min_distance_to_edge = usize::MAX;
		for q2 in min_degree_qubits.iter() {
			let distance = distances[*q1][*q2];
			if distance < min_distance_to_edge {
				min_distance_to_edge = distance;
			}
		}

		if max_distance_to_edge == usize::MAX || min_distance_to_edge > max_distance_to_edge {
			max_distance_to_edge = min_distance_to_edge;
			max_distance_qubit = *q1;
		}
	}
	if max_distance_qubit == usize::MAX {
		return Err("failed to find a center physical qubit".to_string());
	}
	Ok(max_distance_qubit)
}

fn all_pairs_shortest_paths(device: &Connectivity<TwoQubitEdge>) -> Vec<Vec<usize>> {
	let num_nodes = device.node_count();
	let mut distances = vec![vec![usize::MAX; num_nodes]; num_nodes];

	for (start, distance_row) in distances.iter_mut().enumerate() {
		let mut queue = VecDeque::new();
		distance_row[start] = 0;
		queue.push_back(start);

		while let Some(node) = queue.pop_front() {
			for neighbor in device.neighbors(node) {
				if distance_row[neighbor] == usize::MAX {
					distance_row[neighbor] = distance_row[node] + 1;
					queue.push_back(neighbor);
				}
			}
		}
	}
	distances
}

pub fn print_grid(size: (usize, usize), tree: &Graph) {
	for i in 0..size.0 {
		for j in 0..size.1 {
			let node = i * size.1 + j;
			if tree.has_node(node) {
				print!("x")
			} else {
				print!("\u{00B7}") // middle dot
			}
			if j < size.1 - 1 {
				if tree.has_edge(node, node + 1) {
					print!("-")
				} else {
					print!(" ")
				}
			}
		}
		println!();
		if i < size.0 - 1 {
			for j in 0..size.1 {
				let node = i * size.1 + j;
				if tree.has_edge(node, node + size.1) {
					print!("| ")
				} else {
					print!("  ")
				}
			}
			println!()
		}
	}
	println!()
}

pub fn implement_mapping(tableau: &CliffordTableau, mapping: &[usize]) -> CliffordTableau {
	let num_logical_qubits = mapping.len();
	let mapped_tableau = CliffordTableau::id();
	for num_logical_qubit in 0..num_logical_qubits {
		// Destabilizers
		let orignal_row: PauliString = tableau.get_x_row(num_logical_qubit);
		let mut new_row = PauliString::id();
		for (i, &mapped_index) in mapping.iter().enumerate() {
			new_row.set(mapped_index, orignal_row.get(i));
		}
		//		mapped_tableau.set_x_row(mapping[num_logical_qubit], new_row);
		// Stabilizers
		let orignal_row: PauliString = tableau.get_z_row(num_logical_qubit);
		let mut new_row = PauliString::id();
		for (i, &mapped_index) in mapping.iter().enumerate() {
			new_row.set(mapped_index, orignal_row.get(i));
		}
		//		mapped_tableau.set_z_row(mapping[num_logical_qubit], new_row);
	}

	// Phases

	mapped_tableau
}
