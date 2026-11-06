use crate::spcc::graph::Graph;
use std::collections::HashMap;

pub type Pauli = u8;
pub type Edge = (usize, usize);

pub const GATES1: [char; 3] = ['I', 'H', 'V'];
pub const GATES2: [char; 3] = ['H', 'S', 'V'];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GateStep {
	pub edge: Edge,
	pub old_type: String,
	pub new_type: String,
}

impl GateStep {
	pub fn new(edge: Edge, old_type: &str, new_type: &str) -> Self {
		Self {
			edge,
			old_type: old_type.to_string(),
			new_type: new_type.to_string(),
		}
	}
}

pub type GateSequence = Vec<GateStep>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GateMapKey {
	pub old_state: String,
	pub new_state: String,
	pub operations: [[Pauli; 2]; 2],
}

pub type GateMap = HashMap<GateMapKey, Vec<(char, char)>>;

pub fn apply_cnot(p1: Pauli, p2: Pauli) -> (Pauli, Pauli) {
	let pauli1 = (p1 & 0b01) | ((p1 ^ p2) & 0b10);
	let pauli2 = ((p1 ^ p2) & 0b01) | (p2 & 0b10);
	(pauli1, pauli2)
}

pub fn apply_v(p: Pauli) -> Pauli {
	((0b01 & p) ^ (p >> 1)) | (0b10 & p)
}

pub fn apply_s(p: Pauli) -> Pauli {
	(0b01 & p) | ((0b10 & p) ^ ((p & 0b01) << 1))
}

pub fn apply_i(p: Pauli) -> Pauli {
	p
}

pub fn apply_h(p: Pauli) -> Pauli {
	(p >> 1) | ((p & 0b01) << 1)
}

pub fn apply_single_qubit_gate(p: Pauli, gate_name: char) -> Pauli {
	match gate_name {
		'I' => apply_i(p),
		'H' => apply_h(p),
		'V' => apply_v(p),
		'S' => apply_s(p),
		_ => panic!("unknown single-qubit gate: {gate_name}"),
	}
}

pub fn construct_gate_map() -> GateMap {
	let mut result = GateMap::new();
	let pauli_ops = [0b01, 0b10, 0b11, 0b00];

	for &a0 in &pauli_ops {
		for &a1 in &pauli_ops {
			for &b0 in &pauli_ops {
				for &b1 in &pauli_ops {
					let operations = [[a0, a1], [b0, b1]];
					if operations == [[0, 0], [0, 0]] {
						continue;
					}

					let old_state = state_for_operations(operations);

					for &gate1 in &GATES1 {
						for &gate2 in &GATES2 {
							let mut new_operations = [[0; 2]; 2];

							for i in 0..2 {
								let first = apply_single_qubit_gate(operations[i][0], gate1);
								let second = apply_single_qubit_gate(operations[i][1], gate2);
								let (first, second) = apply_cnot(first, second);
								new_operations[i] = [first, second];
							}

							let new_state = state_for_operations(new_operations);
							if old_state == new_state {
								continue;
							}

							let key = GateMapKey {
								old_state: old_state.clone(),
								new_state,
								operations,
							};
							result.entry(key).or_default().push((gate1, gate2));
						}
					}
				}
			}
		}
	}

	result
}

fn state_for_operations(operations: [[Pauli; 2]; 2]) -> String {
	let mut state = String::with_capacity(2);
	for (&first, &second) in operations[0].iter().zip(operations[1].iter()) {
		let symbol = if first == 0 && second == 0 {
			'I'
		} else if first != 0 && second == 0 {
			'>'
		} else if first == 0 && second != 0 {
			'<'
		} else if first == second {
			'C'
		} else {
			'A'
		};
		state.push(symbol);
	}
	state
}

pub fn leaf(tree: &Graph) -> Option<usize> {
	tree.nodes().find(|&node| tree.degree(node) == Some(1))
}

pub fn get_edge_type(tree: &Graph, edge: Edge) -> String {
	let first = tree.get_node_type(edge.0).expect("node has no type");
	let second = tree.get_node_type(edge.1).expect("node has no type");
	format!("{first}{second}")
}

pub fn resolve_2qubits(qubits: (usize, usize), tree: &Graph) -> (GateSequence, i32, String) {
	let edge = (qubits.0, qubits.1);
	let edge_type = get_edge_type(tree, edge);
	let mut gates = Vec::new();

	let priority = if ["CC", ">>", "<<", "CA", "<A", ">A"].contains(&edge_type.as_str()) {
		gates.push(GateStep::new(
			edge,
			&edge_type,
			&format!("I{}", &edge_type[1..]),
		));
		0
	} else if ["A>", "A<"].contains(&edge_type.as_str()) {
		gates.push(GateStep::new(edge, &edge_type, "CA"));
		gates.push(GateStep::new(edge, "CA", "IA"));
		1
	} else if edge_type == "AC" {
		gates.push(GateStep::new(edge, &edge_type, ">A"));
		gates.push(GateStep::new(edge, ">A", "IA"));
		2
	} else if edge_type == "CI" {
		gates.push(GateStep::new(edge, &edge_type, "CC"));
		gates.push(GateStep::new(edge, "CC", "IC"));
		2
	} else if [">I", "<I"].contains(&edge_type.as_str()) {
		let first = &edge_type[0..1];
		gates.push(GateStep::new(edge, &edge_type, &format!("{first}{first}")));
		gates.push(GateStep::new(
			edge,
			&format!("{first}{first}"),
			&format!("I{first}"),
		));
		2
	} else if edge_type == "AI" {
		gates.push(GateStep::new(edge, &edge_type, "AC"));
		gates.push(GateStep::new(edge, "AC", "<A"));
		gates.push(GateStep::new(edge, "<A", "IA"));
		3
	} else {
		-1
	};

	(gates, priority, edge_type)
}

pub fn resolve_3qubits(qubits: (usize, usize, usize), tree: &Graph) -> (GateSequence, i32, String) {
	let edge1 = (qubits.0, qubits.1);
	let edge2 = (qubits.1, qubits.2);
	let type1 = get_edge_type(tree, edge1);
	let type2 = get_edge_type(tree, edge2);
	let mut gates = Vec::new();
	let priority;

	if type1 == "AA" {
		if type2 == "AA" {
			gates.push(GateStep::new(edge1, "AA", "C<"));
			gates.push(GateStep::new(edge2, "<A", "CA"));
			gates.push(GateStep::new(edge1, "CC", "IC"));
			priority = 3;
		} else if type2 == "A>" {
			gates.push(GateStep::new(edge1, "AA", "<C"));
			gates.push(GateStep::new(edge2, "C>", "<>"));
			gates.push(GateStep::new(edge1, "<<", "I<"));
			priority = 3;
		} else if type2 == "A<" {
			gates.push(GateStep::new(edge1, "AA", ">C"));
			gates.push(GateStep::new(edge2, "C<", "><"));
			gates.push(GateStep::new(edge1, ">>", "I>"));
			priority = 3;
		} else if type2 == "AC" {
			gates.push(GateStep::new(edge2, "AC", ">A"));
			gates.push(GateStep::new(edge1, "A>", "CA"));
			gates.push(GateStep::new(edge1, "CA", "IA"));
			priority = 3;
		} else if type2 == "AI" {
			gates.push(GateStep::new(edge2, "AI", "AC"));
			gates.push(GateStep::new(edge2, "AC", ">A"));
			gates.push(GateStep::new(edge1, "A>", "CA"));
			gates.push(GateStep::new(edge1, "CA", "IA"));
			priority = 4;
		} else {
			priority = -1;
		}
	} else if ["<>", "><"].contains(&type1.as_str()) {
		if ["<A", ">A"].contains(&type2.as_str()) {
			gates.push(GateStep::new(edge2, &type2, "AC"));
			gates.push(GateStep::new(edge1, &format!("{}A", &type1[0..1]), "IA"));
			priority = 2;
		} else if ["<>", "><", "<C", ">C"].contains(&type2.as_str()) {
			gates.push(GateStep::new(edge2, &type2, "AA"));
			gates.push(GateStep::new(edge1, &format!("{}A", &type1[0..1]), "IA"));
			priority = 2;
		} else if ["<I", ">I"].contains(&type2.as_str()) {
			let first = &type2[0..1];
			gates.push(GateStep::new(edge2, &type2, &format!("{first}{first}")));
			gates.push(GateStep::new(
				edge2,
				&format!("{first}{first}"),
				&format!("I{first}"),
			));
			let first1 = &type1[0..1];
			gates.push(GateStep::new(
				edge1,
				&format!("{first1}I"),
				&format!("{first1}{first1}"),
			));
			gates.push(GateStep::new(
				edge1,
				&format!("{first1}{first1}"),
				&format!("I{first1}"),
			));
			priority = 4;
		} else if ["<<", ">>"].contains(&type2.as_str()) {
			gates.push(GateStep::new(edge2, &type2, &format!("I{}", &type2[1..])));
			let first = &type1[0..1];
			gates.push(GateStep::new(
				edge1,
				&format!("{first}I"),
				&format!("{first}{first}"),
			));
			gates.push(GateStep::new(
				edge1,
				&format!("{first}{first}"),
				&format!("I{first}"),
			));
			priority = 3;
		} else {
			priority = -1;
		}
	} else if ["C<", "C>"].contains(&type1.as_str()) {
		if ["<A", ">A"].contains(&type2.as_str()) {
			gates.push(GateStep::new(edge2, &type2, "AC"));
			gates.push(GateStep::new(edge1, "CA", "IA"));
			priority = 2;
		} else if ["<C", ">C", "<>", "><"].contains(&type2.as_str()) {
			gates.push(GateStep::new(edge2, &type2, "AA"));
			gates.push(GateStep::new(edge1, "CA", "IA"));
			priority = 2;
		} else if ["<I", ">I"].contains(&type2.as_str()) {
			let first = &type2[0..1];
			gates.push(GateStep::new(edge2, &type2, &format!("{first}{first}")));
			gates.push(GateStep::new(
				edge2,
				&format!("{first}{first}"),
				&format!("I{first}"),
			));
			gates.push(GateStep::new(edge1, "CI", "CC"));
			gates.push(GateStep::new(edge1, "CC", "IC"));
			priority = 4;
		} else if ["<<", ">>"].contains(&type2.as_str()) {
			gates.push(GateStep::new(edge2, &type2, &format!("I{}", &type2[1..])));
			gates.push(GateStep::new(edge1, "CI", "CC"));
			gates.push(GateStep::new(edge1, "CC", "IC"));
			priority = 3;
		} else {
			priority = -1;
		}
	} else if ["<C", ">C"].contains(&type1.as_str()) {
		let first = &type1[0..1];
		if ["C<", "C>"].contains(&type2.as_str()) {
			gates.push(GateStep::new(edge2, &type2, "AA"));
			gates.push(GateStep::new(edge1, &format!("{first}A"), "IA"));
			priority = 2;
		} else if type2 == "CA" {
			gates.push(GateStep::new(edge2, "CA", &format!("{first}A")));
			gates.push(GateStep::new(
				edge1,
				&format!("{first}{first}"),
				&format!("I{first}"),
			));
			priority = 2;
		} else if type2 == "CC" {
			gates.push(GateStep::new(edge2, "CC", "IC"));
			gates.push(GateStep::new(
				edge1,
				&format!("{first}I"),
				&format!("{first}{first}"),
			));
			gates.push(GateStep::new(
				edge1,
				&format!("{first}{first}"),
				&format!("I{first}"),
			));
			priority = 3;
		} else if type2 == "CI" {
			gates.push(GateStep::new(edge2, "CI", "CC"));
			gates.push(GateStep::new(edge2, "CC", "IC"));
			gates.push(GateStep::new(
				edge1,
				&format!("{first}I"),
				&format!("{first}{first}"),
			));
			gates.push(GateStep::new(
				edge1,
				&format!("{first}{first}"),
				&format!("I{first}"),
			));
			priority = 4;
		} else {
			priority = -1;
		}
	} else {
		priority = -1;
	}

	(gates, priority, format!("{type1}{}", &type2[1..]))
}

pub fn min_edge_options(tree: &Graph, target_qubit: usize) -> Vec<GateSequence> {
	let leaf_qubits = tree
		.nodes()
		.filter(|&node| tree.degree(node) == Some(1) && node != target_qubit);

	let mut options = Vec::new();
	let mut minimum_priority = -1;

	for q1 in leaf_qubits {
		let q2 = tree
			.neighbors(q1)
			.next()
			.ok_or_else(|| format!("leaf {q1} has no neighbor"))
			.unwrap();
		let (gates, priority, _) = resolve_2qubits((q1, q2), tree);
		add_if_minimum(&mut options, &mut minimum_priority, gates, priority);

		let q3s = tree.neighbors(q2).filter(|&q3| q3 != q1);
		for q3 in q3s {
			let (gates, priority, _) = resolve_3qubits((q1, q2, q3), tree);
			add_if_minimum(&mut options, &mut minimum_priority, gates, priority);
		}
	}

	if options.is_empty() {
		panic!("no edge options found for target qubit {target_qubit}");
	}

	options
}

fn add_if_minimum(
	options: &mut Vec<GateSequence>,
	minimum_priority: &mut i32,
	gates: GateSequence,
	priority: i32,
) {
	if priority == -1 {
		return;
	}

	if *minimum_priority == -1 || priority < *minimum_priority {
		options.clear();
		options.push(gates);
		*minimum_priority = priority;
	} else if priority == *minimum_priority {
		options.push(gates);
	}
}
