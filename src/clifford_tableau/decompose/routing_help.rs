use petgraph::{Undirected, algo::steiner_tree, graph::UnGraph, prelude::StableGraph};

use crate::{
	clifford_tableau::decompose::{
		QubitProtection, delicate_solver::delicate_solver, simple_solver::simple_solver,
	},
	connectivity::{
		Connectivity, ExplosionNode, RoutingInstruction, RoutingInstructionTarget,
		hypergraph::{HyperEdge, HyperEdgeIndex},
	},
	misc::{NonZeroEvenUsize, enforce_tree},
	pauli::{PauliLetter, PauliString},
	synthesize::handle_instruction,
};

pub(super) fn as_instructions(
	mut steiner_tree: StableGraph<ExplosionNode, usize, Undirected>,
	target: HyperEdgeIndex,
) -> Vec<(HyperEdgeIndex, RoutingInstructionTarget)> {
	let mut result: Vec<(HyperEdgeIndex, RoutingInstructionTarget)> = Vec::new();

	while steiner_tree.node_count() != 0 {
		let indices: Vec<_> = steiner_tree.node_indices().collect();
		for index in indices {
			let neighbors: Vec<_> = steiner_tree.neighbors(index).collect();
			if neighbors.len() == 1 {
				// need to check before remove
				let node = steiner_tree.node_weight(index).unwrap();
				if node.hyper_edges.len() == 1 && *node.hyper_edges.first().unwrap() == target {
					continue;
				}

				let mut node = steiner_tree.remove_node(index).unwrap();
				if node.hyper_edges.len() == 1 {
					// represents and edge
					let edge = node.hyper_edges.pop().unwrap();

					let neighbor_node = steiner_tree
						.node_weight(*neighbors.first().unwrap())
						.unwrap();
					let targets = neighbor_node.hyper_nodes.clone();

					let target = match targets.len() {
						1 => RoutingInstructionTarget::Single(targets[0]),
						_ => RoutingInstructionTarget::Multiple(targets),
					};
					result.push((edge, target));
				}
			} else if neighbors.is_empty() {
				let weight = steiner_tree.node_weight(index).unwrap();
				if weight.hyper_edges.len() == 1 {
					assert_eq!(steiner_tree.node_count(), 1);
					let mut node = steiner_tree.remove_node(index).unwrap();
					let edge = node.hyper_edges.pop().unwrap();
					assert_eq!(edge, target);
				} else {
					steiner_tree.remove_node(index).unwrap();
				}
			}
		}
	}

	result
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_target(
	mut row: PauliString,
	letter: PauliLetter,
	target: usize,
	protection: QubitProtection,
	graph: &StableGraph<ExplosionNode, usize, Undirected, u32>,
	edge_index: HyperEdgeIndex,
	connectivity: &Connectivity,
	gate_size: NonZeroEvenUsize,
	dirty_qubits: &[usize],
	edge: &HyperEdge,
) -> Vec<PauliString> {
	let mut result = Vec::new();

	let graph_clone: UnGraph<_, _> = graph.clone().into();
	let terminals = {
		let mut associated: Vec<usize> = row.targets();

		// make sure that associated contains something from targets
		for qubit in connectivity
			.hypergraph
			.get_edge(edge_index)
			.unwrap()
			.nodes
			.iter()
		{
			if row.get(*qubit) == PauliLetter::I {
				associated.push(*qubit);
			}
		}

		let mut terminals = Vec::new();
		for node_index in graph_clone.node_indices() {
			let node = graph_clone.node_weight(node_index).unwrap();
			for node in node.hyper_nodes.iter() {
				if associated.contains(node) {
					terminals.push(node_index);
					break;
				}
			}
		}

		terminals
	};

	let tree = {
		let mut tree = steiner_tree(&graph_clone, &terminals);
		enforce_tree(&mut tree, &terminals);
		tree
	};

	let instructions =
		as_instructions(tree, edge_index)
			.into_iter()
			.map(|(edge_index, instruction_target)| RoutingInstruction {
				qubits: connectivity
					.hypergraph
					.get_edge(edge_index)
					.unwrap()
					.nodes
					.as_slice(),
				target: instruction_target,
			});

	for instruction in instructions {
		let mut push_strings = handle_instruction(row.clone(), gate_size, instruction);
		for string in push_strings.iter() {
			row.pi_over_4_sandwitch(false, string);
		}
		result.append(&mut push_strings);
	}

	// Check that we only have things left inside this edge
	for occupied in row.targets() {
		assert!(edge.nodes.contains(&occupied))
	}

	let push_strings = if dirty_qubits.len() >= gate_size.as_value() {
		simple_solver(
			row.clone(),
			gate_size,
			target,
			letter,
			dirty_qubits,
			protection,
		)
	} else {
		delicate_solver(&row, gate_size, target, letter, Some(&edge.nodes))
	};

	for string in push_strings {
		row.pi_over_4_sandwitch(false, &string);
		result.push(string);
	}

	assert_eq!(
		row,
		match letter {
			PauliLetter::X => PauliString::x(target),
			PauliLetter::Z => PauliString::z(target),
			_ => unreachable!(),
		}
	);

	result
}
