use pauli::PauliLetter;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::time;

use super::gateset::CNotHSV;
use super::resolve_in_pairs::GateMapKey;
use crate::CliffordTableau;
use crate::spcc::graph::Graph;
use crate::spcc::resolve_in_pairs::GateMap;
use crate::spcc::resolve_in_pairs::GateStep;
use crate::spcc::resolve_in_pairs::construct_gate_map;
use crate::spcc::resolve_in_pairs::min_edge_options;
use circuit::{
	Circuit,
	gates::{CNot, H, S, V},
};

pub fn tableau_to_circuit_pairs(
	input: &CliffordTableau,
	device_tree: &Graph,
	allow_permutation: bool,
) -> (Circuit<CNotHSV>, Vec<usize>) {
	let mut timer = 0;
	let num_qubits = input.size();
	let mut qc_out: Circuit<CNotHSV> = Circuit::new();
	let mut tableau = input.clone();
	//	print!("Initial tableau:\n{:?}\n", num_qubits);

	// Array representing removed gadgets. 1 if gadget is removed, 0 otherwise.
	let mut removed_qubits: Vec<bool> = vec![false; num_qubits];
	let mut removed_rows: Vec<bool> = vec![false; num_qubits];

	// Output permutation
	let mut perm_out: Vec<usize> = vec![usize::MAX; num_qubits];

	// Remove non-used qubits: both rows are identity
	for row in 0..num_qubits {
		let mut remove = true;
		for qubit in 0..num_qubits {
			if qubit == row {
				if tableau.get_x_row(row).get(qubit) != PauliLetter::X
					|| tableau.get_z_row(row).get(qubit) != PauliLetter::Z
				{
					remove = false;
					break;
				}
			} else {
				if tableau.get_x_row(row).get(qubit) != PauliLetter::I
					|| tableau.get_z_row(row).get(qubit) != PauliLetter::I
				{
					remove = false;
					break;
				}
			}
		}
		if remove {
			removed_rows[row] = true;
			removed_qubits[row] = true;
			perm_out[row] = row;
		}
	}
	let gate_map = construct_gate_map();
	//	for (key, value) in &gate_map {
	//		println!("Key: {:?}, Value: {:?}", key, value);
	//	}
	let mut tableau_tree = construct_tableau_tree(&removed_qubits, &device_tree);
	let apsp = all_pairs_shortest_paths(&tableau_tree);

	// Main loop of the algorithm. We will keep removing gadget pairs until all gadget pairs
	// are removed and all qubits are mapped.
	while tableau_tree.node_count() > 1 {
		// select which pair to remove and which is target qubit for that row
		let mut gadget_pair_scores: HashMap<usize, usize> = HashMap::new();
		for x in 0..num_qubits {
			if !removed_rows[x] {
				let score = gadget_pair_score(&tableau, &tableau_tree, x);
				gadget_pair_scores.insert(x, score);
			}
		}
		let mut min_node_score: i64 = i64::MAX;
		let mut min_pair = usize::MAX;
		let mut min_column = usize::MAX;
		let mut min_gadget_score = usize::MAX;
		for column in 0..num_qubits {
			if tableau_tree.has_node(column) && tableau_tree.degree(column) == Some(1) {
				let mut node_score: i64 = 0;
				for j in 0..num_qubits {
					if tableau_tree.has_node(j) {
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
						&& (tableau.get_x_row(pair).get(column) != PauliLetter::I
							|| tableau.get_z_row(pair).get(column) != PauliLetter::I)
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
		timer += reduce_row(
			min_pair,
			min_column,
			&mut tableau,
			&mut qc_out,
			&mut tableau_tree,
			&gate_map,
		);

		// remove leaf from qubit_tree and remove gadget-pair from remaining_gadgets
		tableau_tree.remove_node(min_column);
		removed_rows[min_pair] = true;
		removed_qubits[min_column] = true;
		// Update permutation
		perm_out[min_column] = min_pair;
		//		println!(
		//			"Removed pair: {} (row) and {} (column), remaining rows: {:?}, remaining qubits: {:?}",
		//			min_pair, min_column, removed_rows, removed_qubits
		//		);
		//print_tableau(&tableau);
	}

	// Correct zx in last rows last pair
	let mut row: usize = usize::MAX;
	for r in 0..num_qubits {
		if !removed_rows[r] {
			row = r;
			break;
		}
	}
	let last_qubit = tableau_tree.nodes().next().unwrap();
	if tableau.get_x_row(row).get(last_qubit) == PauliLetter::Y {
		add_gate_to_circuit_and_tableau('S', last_qubit, &mut tableau, &mut qc_out);
	} else if tableau.get_x_row(row).get(last_qubit) == PauliLetter::Z {
		add_gate_to_circuit_and_tableau('H', last_qubit, &mut tableau, &mut qc_out);
	}
	if tableau.get_z_row(row).get(last_qubit) == PauliLetter::Y {
		add_gate_to_circuit_and_tableau('V', last_qubit, &mut tableau, &mut qc_out);
	}
	perm_out[last_qubit] = row;

	// Undo phase signs
	for row in 0..num_qubits {
		if tableau.get_x_signs().get(row) {
			let mut qubit: usize = usize::MAX;
			for i in 0..num_qubits {
				if tableau.get_x_row(row).get(i) != PauliLetter::I {
					qubit = i;
					break;
				}
			}
			add_gate_to_circuit_and_tableau('S', qubit, &mut tableau, &mut qc_out);
			add_gate_to_circuit_and_tableau('S', qubit, &mut tableau, &mut qc_out);
		}
		if tableau.get_z_signs().get(row) {
			let mut qubit: usize = usize::MAX;
			for i in 0..num_qubits {
				if tableau.get_z_row(row).get(i) != PauliLetter::I {
					qubit = i;
					break;
				}
			}
			add_gate_to_circuit_and_tableau('V', qubit, &mut tableau, &mut qc_out);
			add_gate_to_circuit_and_tableau('V', qubit, &mut tableau, &mut qc_out);
		}
	}
	//	println!("Final tableau:");
	//	print_tableau(&tableau);
	qc_out.reverse();
	println!(
		"Circuit and tablau update took {} ms",
		(timer as f64 / 1000.0).round() as i32
	);
	(qc_out, perm_out)
}

fn all_pairs_shortest_paths(tableau_tree: &Graph) -> Vec<Vec<usize>> {
	let num_nodes = tableau_tree.node_count();
	let mut distances = vec![vec![usize::MAX; num_nodes]; num_nodes];

	for start in 0..num_nodes {
		let mut queue = VecDeque::new();
		distances[start][start] = 0;
		queue.push_back(start);

		while let Some(node) = queue.pop_front() {
			for neighbor in tableau_tree.neighbors(node) {
				if distances[start][neighbor] == usize::MAX {
					distances[start][neighbor] = distances[start][node] + 1;
					queue.push_back(neighbor);
				}
			}
		}
	}
	distances
}

fn gadget_pair_score(tableau: &CliffordTableau, tableau_tree: &Graph, pair: usize) -> usize {
	let (gadget_tree, nr_i) = create_gadget_tree_graph(tableau, tableau_tree, pair, pair);
	gadget_tree.nodes().count() + nr_i
}

fn create_gadget_tree_graph(
	tableau: &CliffordTableau,
	tableau_tree: &Graph,
	pair: usize,
	target_qubit: usize,
) -> (Graph, usize) {
	let mut gadget_tree = tableau_tree.clone();
	let mut nr_i: usize = 0;
	let pauli_i = PauliLetter::I;
	let node_list: Vec<usize> = tableau_tree.nodes().collect();
	for q in node_list {
		let paulix = tableau.get_x_row(pair).get(q);
		let pauliy = tableau.get_z_row(pair).get(q);
		if paulix == pauli_i && pauliy == pauli_i {
			gadget_tree.set_node_type(q, 'I');
			nr_i += 1;
		} else if paulix != pauli_i && pauliy == pauli_i {
			gadget_tree.set_node_type(q, '>');
		} else if paulix == pauli_i && pauliy != pauli_i {
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
			.filter(|&q| gadget_tree.degree(q) == Some(1))
			.collect();
		if leafs.len() == 0 {
			break;
		}
		quit = true;
		for q in leafs {
			if q != target_qubit && gadget_tree.get_node_type(q) == Some('I') {
				nr_i -= 1;
				quit = false;
				gadget_tree.remove_node(q);
			}
		}
	}
	(gadget_tree, nr_i)
}

fn reduce_row(
	row: usize,
	target_qubit: usize,
	tableau: &mut CliffordTableau,
	qc_out: &mut Circuit<CNotHSV>,
	tableau_tree: &mut Graph,
	gate_map: &GateMap,
) -> i32 {
	let num_qubits = tableau.size();
	let mut timer = 0;
	loop {
		let (gadget_tree, nr_i) =
			create_gadget_tree_graph(tableau, tableau_tree, row, target_qubit);
		if gadget_tree.node_count() < 2 {
			// Only target_qubit left, we are done
			break;
		}
		for node in gadget_tree.nodes() {
			if node != target_qubit
				&& gadget_tree.degree(node) == Some(1)
				&& gadget_tree.get_node_type(node) == Some('I')
			{
				panic!("ERROR: leaf node is I");
			}
		}

		// Find a leaf and abstract gate sequences to reduce leaf
		let abs_gates_options: Vec<Vec<GateStep>> = min_edge_options(&gadget_tree, target_qubit);
		let abs_gates = &abs_gates_options[0]; // There are several leafs where to proceed, we choose trivially the first one.

		for gate_step in abs_gates {
			let current_paulis = [
				[
					to_int(tableau.get_x_row(row).get(gate_step.edge.0)),
					to_int(tableau.get_x_row(row).get(gate_step.edge.1)),
				],
				[
					to_int(tableau.get_z_row(row).get(gate_step.edge.0)),
					to_int(tableau.get_z_row(row).get(gate_step.edge.1)),
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
			let start = time::Instant::now();
			add_cnot_and_single_qubit_gates(gate_step.edge, circ_gates, tableau, qc_out);
			timer += start.elapsed().as_micros() as i32;
		}
	}

	// Change last qubit to Z/X
	let mut last_qubit = usize::MAX;
	for q in 0..num_qubits {
		if tableau.get_x_row(row).get(q) != PauliLetter::I
			|| tableau.get_z_row(row).get(q) != PauliLetter::I
		{
			last_qubit = q;
			break;
		}
	}
	if tableau.get_x_row(row).get(last_qubit) == PauliLetter::Y {
		add_gate_to_circuit_and_tableau('S', last_qubit, tableau, qc_out);
	} else if tableau.get_x_row(row).get(last_qubit) == PauliLetter::Z {
		add_gate_to_circuit_and_tableau('H', last_qubit, tableau, qc_out);
	}
	if tableau.get_z_row(row).get(last_qubit) == PauliLetter::Y {
		add_gate_to_circuit_and_tableau('V', last_qubit, tableau, qc_out);
	}
	timer
}

fn to_char(letter: PauliLetter) -> char {
	match letter {
		PauliLetter::I => 'I',
		PauliLetter::X => 'X',
		PauliLetter::Y => 'Y',
		PauliLetter::Z => 'Z',
	}
}

fn to_int(letter: PauliLetter) -> u8 {
	match letter {
		PauliLetter::I => 0,
		PauliLetter::X => 1,
		PauliLetter::Y => 3,
		PauliLetter::Z => 2,
	}
}

fn add_gate_to_circuit_and_tableau(
	gate: char,
	qubit: usize,
	tableau: &mut CliffordTableau,
	qc_out: &mut Circuit<CNotHSV>,
) {
	add_single_gate_to_circuit(gate, qubit, qc_out);
	append_single_gate_to_tableau(gate, qubit, tableau);
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

fn append_single_gate_to_tableau(gate: char, qubit: usize, tableau: &mut CliffordTableau) {
	match gate {
		'S' => {
			for _ in 0..3 {
				tableau.merge_clifford(&CNotHSV::S(S { target: qubit }));
			}
		}
		'V' => {
			for _ in 0..3 {
				tableau.merge_clifford(&CNotHSV::V(V { target: qubit }));
			}
		}
		'H' => tableau.merge_clifford(&CNotHSV::H(H { target: qubit })),
		'I' => {} // Do nothing for identity
		_ => panic!("ERROR: Unsupported gate in add_cnot_and_single_qubit_gates"),
	}
}

fn add_cnot_and_single_qubit_gates(
	edge: (usize, usize),
	gates: (char, char),
	tableau: &mut CliffordTableau,
	qc_out: &mut Circuit<CNotHSV>,
) {
	add_gate_to_circuit_and_tableau(gates.0, edge.0, tableau, qc_out);
	add_gate_to_circuit_and_tableau(gates.1, edge.1, tableau, qc_out);

	let cnot = CNot::new(edge.0, edge.1).expect("CNOT control and target must differ");
	qc_out.push(CNotHSV::CNot(cnot));
	tableau.merge_clifford(&CNotHSV::CNot(cnot));
}

fn construct_tableau_tree(removed_qubits: &Vec<bool>, device_tree: &Graph) -> Graph {
	let mut tableau_tree = device_tree.clone();
	let node_list: Vec<usize> = tableau_tree.nodes().collect();
	for qubit in node_list {
		if removed_qubits[qubit] {
			tableau_tree.remove_node(qubit);
		}
	}
	tableau_tree
}

fn print_tableau(tableau: &CliffordTableau) {
	println!("Tableau:");
	for row in 0..tableau.size() {
		let mut row_str = String::new();
		for qubit in 0..tableau.size() {
			row_str.push(to_char(tableau.get_x_row(row).get(qubit)));
		}
		row_str.push(if tableau.get_x_signs().get(row) {
			'-'
		} else {
			'+'
		});
		println!("{}", row_str);
	}
	println!("");
	for row in 0..tableau.size() {
		let mut row_str = String::new();
		for qubit in 0..tableau.size() {
			row_str.push(to_char(tableau.get_z_row(row).get(qubit)));
		}
		row_str.push(if tableau.get_z_signs().get(row) {
			'-'
		} else {
			'+'
		});
		println!("{}", row_str);
	}
}
