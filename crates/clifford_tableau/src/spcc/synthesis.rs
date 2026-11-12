use std::collections::HashMap;
use std::collections::VecDeque;

use super::CNotHSV;
use super::ct_fast::CTFast;
use super::resolve_in_pairs::GateMapKey;
use crate::spcc::graph::Graph;
use crate::spcc::resolve_in_pairs::GateMap;
use crate::spcc::resolve_in_pairs::GateStep;
use crate::spcc::resolve_in_pairs::construct_gate_map;
use crate::spcc::resolve_in_pairs::min_edge_options;
use circuit::{
	Circuit,
	gates::{CNot, H, S, V},
};
use core::qubit_map::PartialQubitMap;
use core::qubit_map::QubitMap;

pub fn tableau_to_circuit_pairs(
	tableau: &CTFast,
	device_tree: &Graph,
	allow_permutation: bool,
	output_mapping: bool,
) -> (Circuit<CNotHSV>, QubitMap) {
	//	println!("Start tableau:\n{}", tableau.display());
	let num_qubits = tableau.size();
	let mut qc_out: Circuit<CNotHSV> = Circuit::new();
	let mut tableau = if output_mapping {
		tableau.inverse()
	} else {
		tableau.clone()
	};

	// Array representing removed gadgets. 1 if gadget is removed, 0 otherwise.
	let mut removed_qubits: Vec<bool> = vec![false; num_qubits];
	let mut removed_rows: Vec<bool> = vec![false; num_qubits];

	// Output permutation
	let mut perm_out: Vec<usize> = vec![usize::MAX; num_qubits];

	// Remove non-used qubits: both rows are identity
	for (row, removed_row) in removed_rows.iter_mut().enumerate() {
		let mut remove = true;
		let mut target_qubit = usize::MAX;
		for qubit in 0..num_qubits {
			if tableau.get_x_row_col(row, qubit) == 0 && tableau.get_z_row_col(row, qubit) == 0 {
				continue;
			}
			if tableau.get_x_row_col(row, qubit) == 1 && tableau.get_z_row_col(row, qubit) == 2 {
				if target_qubit == usize::MAX {
					target_qubit = qubit;
					continue;
				} else {
					remove = false;
					break;
				}
			}
			remove = false;
			break;
		}
		if remove {
			*removed_row = true;
			removed_qubits[target_qubit] = true;
			perm_out[target_qubit] = row;
		}
	}
	let gate_map = construct_gate_map();
	//	for (key, value) in &gate_map {
	//		println!("Key: {:?}, Value: {:?}", key, value);
	//	}
	let mut tableau_tree = construct_tableau_tree(&removed_rows, device_tree);
	let apsp = all_pairs_shortest_paths(&tableau_tree, num_qubits);
	// Main loop of the algorithm. We will keep removing gadget pairs until all gadget pairs
	// are removed and all qubits are mapped.
	while tableau_tree.node_count() > 1 {
		// select which pair to remove and which is target qubit for that row
		let mut gadget_pair_scores: HashMap<usize, usize> = HashMap::new();
		for (x, removed_row) in removed_rows.iter().enumerate() {
			if !removed_row {
				let score = gadget_pair_score(&tableau, &tableau_tree, x);
				gadget_pair_scores.insert(x, score);
			}
		}
		let mut min_node_score: i64 = i64::MAX;
		let mut min_pair = usize::MAX;
		let mut min_column = usize::MAX;
		let mut min_gadget_score = usize::MAX;
		for column in 0..num_qubits {
			if !removed_qubits[column] && tableau_tree.degree(column) == Some(1) {
				let mut node_score: i64 = 0;
				for j in 0..num_qubits {
					if !removed_qubits[j] {
						node_score -= apsp[column][j] as i64;
					}
				}
				if !allow_permutation {
					min_column = column;
					min_node_score = node_score;
					min_gadget_score = 0;
					min_pair = column;
					continue;
				}
				for pair in 0..num_qubits {
					if !removed_rows[pair]
						&& (tableau.get_x_row_col(pair, column) != 0
							|| tableau.get_z_row_col(pair, column) != 0)
					{
						let gadget_score = gadget_pair_scores[&pair];
						if min_column == usize::MAX
							|| gadget_score < min_gadget_score
							|| (gadget_score == min_gadget_score && node_score < min_node_score)
						{
							min_column = column;
							min_node_score = node_score;
							min_gadget_score = gadget_score;
							min_pair = pair;
						}
					}
				}
			}
		}
		if min_pair == usize::MAX || min_column == usize::MAX {
			println!("Removed qubits: {:?}", removed_qubits);
			println!("Removed rows  : {:?}", removed_rows);
			println!("Tableau tree: {:?}", tableau_tree);
			print!("Tableau:\n{:?}\n", tableau);
			panic!("ERROR: No row pair found to remove");
		}
		// Resolve gadget pair
		reduce_row(
			min_pair,
			min_column,
			&mut tableau,
			&mut qc_out,
			&mut tableau_tree,
			&gate_map,
			output_mapping,
		);

		// remove leaf from qubit_tree and remove gadget-pair from remaining_gadgets
		tableau_tree.remove_node(min_column);
		removed_rows[min_pair] = true;
		removed_qubits[min_column] = true;
		// Update permutation
		perm_out[min_column] = min_pair;
		//print_tableau(&tableau);
	}

	// Correct zx in last rows last pair
	let row = removed_rows
		.iter()
		.enumerate()
		.find_map(|(r, &removed)| (!removed).then_some(r))
		.unwrap_or(usize::MAX);
	let last_qubit = tableau_tree.nodes().next().unwrap();
	if tableau.get_x_row_col(row, last_qubit) == 3 {
		add_gate_to_circuit_and_tableau('S', last_qubit, &mut tableau, &mut qc_out, output_mapping);
	} else if tableau.get_x_row_col(row, last_qubit) == 2 {
		add_gate_to_circuit_and_tableau('H', last_qubit, &mut tableau, &mut qc_out, output_mapping);
	}
	if tableau.get_z_row_col(row, last_qubit) == 3 {
		add_gate_to_circuit_and_tableau('V', last_qubit, &mut tableau, &mut qc_out, output_mapping);
	}
	perm_out[last_qubit] = row;

	// Undo phase signs
	for row in 0..num_qubits {
		if tableau.get_x_phase(row) {
			let mut qubit: usize = usize::MAX;
			for i in 0..num_qubits {
				if tableau.get_x_row_col(row, i) != 0 {
					qubit = i;
					break;
				}
			}
			add_gate_to_circuit_and_tableau('S', qubit, &mut tableau, &mut qc_out, output_mapping);
			add_gate_to_circuit_and_tableau('S', qubit, &mut tableau, &mut qc_out, output_mapping);
		}
		if tableau.get_z_phase(row) {
			let mut qubit: usize = usize::MAX;
			for i in 0..num_qubits {
				if tableau.get_z_row_col(row, i) != 0 {
					qubit = i;
					break;
				}
			}
			add_gate_to_circuit_and_tableau('V', qubit, &mut tableau, &mut qc_out, output_mapping);
			add_gate_to_circuit_and_tableau('V', qubit, &mut tableau, &mut qc_out, output_mapping);
		}
	}
	if !output_mapping {
		qc_out.reverse();
	}

	let mut partial_mapping = PartialQubitMap::default();
	for (from, to) in perm_out.into_iter().enumerate() {
		if output_mapping {
			partial_mapping.map_qubit(from, to).unwrap();
		} else {
			partial_mapping.map_qubit(to, from).unwrap();
		}
	}
	let mapping: QubitMap = partial_mapping.try_into().unwrap();

	//	println!("Final tableau:\n{}", tableau.display());
	(qc_out, mapping)
}

fn all_pairs_shortest_paths(tableau_tree: &Graph, num_qubits: usize) -> Vec<Vec<usize>> {
	let mut distances = vec![vec![usize::MAX; num_qubits]; num_qubits];

	for (start, distance_row) in distances.iter_mut().enumerate() {
		let mut queue = VecDeque::new();
		distance_row[start] = 0;
		queue.push_back(start);

		while let Some(node) = queue.pop_front() {
			for neighbor in tableau_tree.neighbors(node) {
				if distance_row[neighbor] == usize::MAX {
					distance_row[neighbor] = distance_row[node] + 1;
					queue.push_back(neighbor);
				}
			}
		}
	}
	distances
}

fn gadget_pair_score(tableau: &CTFast, tableau_tree: &Graph, pair: usize) -> usize {
	let (gadget_tree, nr_i) = create_gadget_tree_graph(tableau, tableau_tree, pair, pair);
	gadget_tree.node_count() + nr_i
}

fn create_gadget_tree_graph(
	tableau: &CTFast,
	tableau_tree: &Graph,
	pair: usize,
	target_qubit: usize,
) -> (Graph, usize) {
	let mut gadget_tree = tableau_tree.clone();
	let mut nr_i: usize = 0;
	for q in tableau_tree.nodes() {
		let paulix = tableau.get_x_row_col(pair, q);
		let pauliy = tableau.get_z_row_col(pair, q);
		if paulix == 0 && pauliy == 0 {
			gadget_tree.set_node_type(q, 'I');
			nr_i += 1;
		} else if paulix != 0 && pauliy == 0 {
			gadget_tree.set_node_type(q, '>');
		} else if paulix == 0 && pauliy != 0 {
			gadget_tree.set_node_type(q, '<');
		} else if paulix != pauliy {
			gadget_tree.set_node_type(q, 'A');
		} else {
			gadget_tree.set_node_type(q, 'C');
		}
	}

	let mut quit = false;
	while !quit {
		let leafs: Vec<usize> = gadget_tree
			.nodes()
			.filter(|&q| {
				gadget_tree.degree(q) == Some(1)
					&& q != target_qubit
					&& gadget_tree.get_node_type(q) == 'I'
			})
			.collect();
		quit = true;
		for q in leafs {
			nr_i -= 1;
			quit = false;
			gadget_tree.remove_node(q);
		}
	}
	(gadget_tree, nr_i)
}

fn reduce_row(
	row: usize,
	target_qubit: usize,
	tableau: &mut CTFast,
	qc_out: &mut Circuit<CNotHSV>,
	tableau_tree: &mut Graph,
	gate_map: &GateMap,
	output_mapping: bool,
) {
	let num_qubits = tableau.size();
	loop {
		let (gadget_tree, _nr_i) =
			create_gadget_tree_graph(tableau, tableau_tree, row, target_qubit);
		if gadget_tree.node_count() < 2 {
			// Only target_qubit left, we are done
			break;
		}
		//		for node in gadget_tree.nodes() {
		//			if node != target_qubit
		//				&& gadget_tree.degree(node) == Some(1)
		//				&& gadget_tree.get_node_type(node) == Some('I')
		//			{
		//				panic!("ERROR: leaf node is I");
		//			}
		//		}

		// Find a leaf and abstract gate sequences to reduce leaf
		let abs_gates_options: Vec<Vec<GateStep>> = min_edge_options(&gadget_tree, target_qubit);
		let abs_gates = &abs_gates_options[0]; // There are several leafs where to proceed, we choose trivially the first one.

		for gate_step in abs_gates {
			let current_paulis = [
				[
					tableau.get_x_row_col(row, gate_step.edge.0),
					tableau.get_x_row_col(row, gate_step.edge.1),
				],
				[
					tableau.get_z_row_col(row, gate_step.edge.0),
					tableau.get_z_row_col(row, gate_step.edge.1),
				],
			];
			// Find possible circuit gate sequances from abstarcted gate sequences
			let gate_map_key = GateMapKey {
				old_state: gate_step.old_type.clone(),
				new_state: gate_step.new_type.clone(),
				operations: current_paulis,
			};
			let gate_options = gate_map.get(&gate_map_key).expect("No gate options found"); // Returns (qubits), qubit1 gate, qubit2 gate
			// Check what circuit gate sequence is best in terms of remaining rows
			let circ_gates = gate_options[0]; // Just take the first option for now
			// Execute best circuit gate sequnce
			add_cnot_and_single_qubit_gates(
				gate_step.edge,
				circ_gates,
				tableau,
				qc_out,
				output_mapping,
			);
		}
	}

	// Change last qubit to Z/X
	let mut last_qubit = usize::MAX;
	for q in 0..num_qubits {
		if tableau.get_x_row_col(row, q) != 0 || tableau.get_z_row_col(row, q) != 0 {
			last_qubit = q;
			break;
		}
	}
	if tableau.get_x_row_col(row, last_qubit) == 3 {
		add_gate_to_circuit_and_tableau('S', last_qubit, tableau, qc_out, output_mapping);
	} else if tableau.get_x_row_col(row, last_qubit) == 2 {
		add_gate_to_circuit_and_tableau('H', last_qubit, tableau, qc_out, output_mapping);
	}
	if tableau.get_z_row_col(row, last_qubit) == 3 {
		add_gate_to_circuit_and_tableau('V', last_qubit, tableau, qc_out, output_mapping);
	}
}

fn add_gate_to_circuit_and_tableau(
	gate: char,
	qubit: usize,
	tableau: &mut CTFast,
	qc_out: &mut Circuit<CNotHSV>,
	output_mapping: bool,
) {
	add_single_gate_to_circuit(gate, qubit, qc_out);
	if output_mapping {
		append_single_gate_to_tableau(gate, qubit, tableau);
	} else {
		append_single_gate_to_tableau_rev(gate, qubit, tableau);
	}
}

fn add_single_gate_to_circuit(gate: char, qubit: usize, qc_out: &mut Circuit<CNotHSV>) {
	match gate {
		'S' => qc_out.push(CNotHSV::S(S { target: qubit })),
		'V' => qc_out.push(CNotHSV::V(V { target: qubit })),
		'H' => qc_out.push(CNotHSV::H(H { target: qubit })),
		'I' => {} // Do nothing for identity
		_ => panic!(
			"ERROR: Unsupported gate in add_cnot_and_single_qubit_gates: {}",
			gate
		),
	}
}

fn append_single_gate_to_tableau_rev(gate: char, qubit: usize, tableau: &mut CTFast) {
	match gate {
		'S' => tableau.apply_sdg(qubit),
		'V' => tableau.apply_vdg(qubit),
		'H' => tableau.apply_h(qubit),
		'I' => {} // Do nothing for identity
		_ => panic!("ERROR: Unsupported gate in add_cnot_and_single_qubit_gates"),
	}
}

fn append_single_gate_to_tableau(gate: char, qubit: usize, tableau: &mut CTFast) {
	match gate {
		'S' => tableau.apply_s(qubit),
		'V' => tableau.apply_v(qubit),
		'H' => tableau.apply_h(qubit),
		'I' => {} // Do nothing for identity
		_ => panic!("ERROR: Unsupported gate in add_cnot_and_single_qubit_gates"),
	}
}

fn add_cnot_and_single_qubit_gates(
	edge: (usize, usize),
	gates: (char, char),
	tableau: &mut CTFast,
	qc_out: &mut Circuit<CNotHSV>,
	output_mapping: bool,
) {
	add_gate_to_circuit_and_tableau(gates.0, edge.0, tableau, qc_out, output_mapping);
	add_gate_to_circuit_and_tableau(gates.1, edge.1, tableau, qc_out, output_mapping);

	let cnot = CNot::new(edge.0, edge.1).expect("CNOT control and target must differ");
	qc_out.push(CNotHSV::CNot(cnot));
	tableau.apply_cnot(edge.0, edge.1);
}

fn construct_tableau_tree(removed_rows: &[bool], device_tree: &Graph) -> Graph {
	let mut tableau_tree = device_tree.clone();
	let node_list: Vec<usize> = tableau_tree.nodes().collect();
	for qubit in node_list {
		if removed_rows[qubit] {
			tableau_tree.remove_node(qubit);
		}
	}
	tableau_tree
}

#[cfg(test)]
mod tests {
	use crate::spcc::ct_fast::CTFast;
	use crate::spcc::mapping;
	use circuit::Circuit;
	use rand::SeedableRng;
	use rand_chacha::ChaCha8Rng;

	use super::CNotHSV;
	use super::tableau_to_circuit_pairs;
	use core::connectivity::TwoQubitEdge;
	use mapping::construct_tree;
	use std::time;

	#[test]
	fn outputmap_test() {
		const ROUNDS: usize = 10;
		const GATES: usize = 10000;
		const QUBITS: usize = 9;
		let connectivity = TwoQubitEdge::square_lattice(QUBITS);
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for _round in 0..ROUNDS {
			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			let mut tableau = CTFast::new(QUBITS);
			for gate in input.iter() {
				tableau.merge_clifford(gate);
			}

			let num_logical_qubits = tableau.size();
			let (_root_node, device_tree) = construct_tree(&connectivity, num_logical_qubits)
				.expect("failed to construct SPCC tree");
			let (circ, output_mapping) =
				tableau_to_circuit_pairs(&tableau, &device_tree, true, true);
			let mut result_tableau = CTFast::new(QUBITS); // Identity
			for gate in circ.iter() {
				if let CNotHSV::CNot(cnot) = gate
					&& !connectivity
						.neighbors(*cnot.target())
						.contains(cnot.control())
				{
					panic!(
						"Got CNot({}, {}) that is not supported by connectivity",
						cnot.control(),
						cnot.target()
					);
				}
				result_tableau.merge_clifford(gate);
			}
			//			result_tableau.merge_qubit_map(&input_mapping);
			let permutation_tableau = CTFast::permutation_tableau(QUBITS, &output_mapping);
			result_tableau = result_tableau.then(&permutation_tableau);
			assert_eq!(result_tableau, tableau);
		}
	}

	#[test]
	fn inputmap_test() {
		const ROUNDS: usize = 10;
		const GATES: usize = 10000;
		const QUBITS: usize = 9;
		let connectivity = TwoQubitEdge::square_lattice(QUBITS);
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for _round in 0..ROUNDS {
			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			let mut tableau = CTFast::new(QUBITS);
			for gate in input.iter() {
				tableau.merge_clifford(gate);
			}

			let num_logical_qubits = tableau.size();
			let (_root_node, device_tree) = construct_tree(&connectivity, num_logical_qubits)
				.expect("failed to construct SPCC tree");
			let (circ, input_mapping) =
				tableau_to_circuit_pairs(&tableau, &device_tree, true, false);
			let mut result_tableau = CTFast::new(QUBITS); // Identity
			let permutation_tableau = CTFast::permutation_tableau(QUBITS, &input_mapping);
			result_tableau = result_tableau.then(&permutation_tableau);
			for gate in circ.iter() {
				if let CNotHSV::CNot(cnot) = gate
					&& !connectivity
						.neighbors(*cnot.target())
						.contains(cnot.control())
				{
					panic!(
						"Got CNot({}, {}) that is not supported by connectivity",
						cnot.control(),
						cnot.target()
					);
				}
				result_tableau.merge_clifford(gate);
			}
			//			result_tableau.merge_qubit_map(&input_mapping);
			assert_eq!(result_tableau, tableau);
		}
	}

	#[test]
	fn benchmark_test() {
		const ROUNDS: usize = 100;
		const SIZES: [(usize, usize); 21] = [
			(2, 2),
			(2, 3),
			(3, 3),
			(3, 4),
			(4, 4),
			(4, 5),
			(5, 5),
			(5, 6),
			(6, 6),
			(6, 7),
			(7, 7),
			(7, 8),
			(8, 8),
			(8, 9),
			(9, 9),
			(9, 10),
			(10, 10),
			(10, 11),
			(11, 11),
			(11, 12),
			(12, 12),
		];
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for (rows, cols) in SIZES {
			let connectivity = TwoQubitEdge::rectangle_lattice(rows, cols);
			let mut time_sum = 0.0;
			let mut cnot_sum = 0;
			let mut depth_sum = 0;
			for _round in 0..ROUNDS {
				let qubits = rows * cols;
				let gates = qubits * 1000;
				let input: Circuit<CNotHSV> = Circuit::random(gates, qubits, &mut rng);
				let mut tableau = CTFast::new(qubits);
				for gate in input.iter() {
					tableau.merge_clifford(gate);
				}
				let num_logical_qubits = tableau.size();
				let (_root_node, device_tree) = construct_tree(&connectivity, num_logical_qubits)
					.expect("failed to construct SPCC tree");
				let start = time::Instant::now();
				let (circ, initial_mapping) =
					tableau_to_circuit_pairs(&tableau, &device_tree, true, false);
				let elapsed = start.elapsed();
				time_sum += elapsed.as_millis() as f64;

				let mut result_tableau = CTFast::new(qubits);
				let permutation = CTFast::permutation_tableau(qubits, &initial_mapping);
				result_tableau = result_tableau.then(&permutation);
				for gate in circ.iter() {
					if let CNotHSV::CNot(cnot) = gate
						&& !connectivity
							.neighbors(*cnot.target())
							.contains(cnot.control())
					{
						panic!(
							"Got CNot({}, {}) that is not supported by connectivity",
							cnot.control(),
							cnot.target()
						);
					}
					result_tableau.merge_clifford(gate);
				}

				assert_eq!(result_tableau, tableau);
				cnot_sum += circ.fileter_len(|g| matches!(g, CNotHSV::CNot(_)));
				depth_sum += circ.filtered_depth(|x| matches!(x, CNotHSV::CNot(_)));
			}
			println!(
				"{},{},{},{}",
				rows * cols,
				(cnot_sum as f64 / ROUNDS as f64).round(),
				(depth_sum as f64 / ROUNDS as f64).round(),
				(time_sum / ROUNDS as f64).round()
			);
		}
	}
}
