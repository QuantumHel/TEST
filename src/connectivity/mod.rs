mod explosion;
pub(crate) mod hypergraph;

use std::collections::BTreeSet;

pub(crate) use crate::connectivity::{explosion::ExplosionNode, hypergraph::HyperGraph};
use crate::misc::{NonZeroEvenUsize, enforce_tree};
use petgraph::{algo::steiner_tree, graph::UnGraph};

#[derive(Debug)]
pub enum RoutingInstructionTarget {
	Single(usize),
	Multiple(Vec<usize>),
	Any,
}

#[derive(Debug)]
pub struct RoutingInstruction<'a> {
	pub qubits: &'a [usize],
	pub target: RoutingInstructionTarget,
}

#[derive(Debug)]
pub struct Connectivity {
	pub(crate) hypergraph: HyperGraph,
	pub(crate) explosion: UnGraph<ExplosionNode, usize>,
	max_operator_size: usize,
	qubit_count: usize,
}

#[derive(Debug)]
pub enum ConnectivityCreationError {
	IndexOutOfRange(usize),
	NotFullyConnected,
	DublicateInGroup,
}

impl Connectivity {
	/// Can give error for IndexOutOfRange, NotFullyConnected, and when there is a DublicateInGroup
	pub fn new(
		qubit_count: usize,
		operator_groups: Vec<Vec<usize>>,
	) -> Result<Self, ConnectivityCreationError> {
		let mut max_operator_size = 0;
		let mut hypergraph = HyperGraph::new();
		let mut nodes = Vec::with_capacity(qubit_count);
		for _ in 0..qubit_count {
			nodes.push(hypergraph.add_node());
		}

		for mut operation_group in operator_groups {
			let n = operation_group.len();
			let operation_set: BTreeSet<_> = operation_group.drain(..).collect();
			if n != operation_set.len() {
				return Err(ConnectivityCreationError::DublicateInGroup);
			}

			let mut targets = Vec::with_capacity(operation_set.len());
			for target in operation_set {
				if target >= qubit_count {
					return Err(ConnectivityCreationError::IndexOutOfRange(target));
				}
				targets.push(*nodes.get(target).unwrap());
			}
			max_operator_size = max_operator_size.min(targets.len());
			hypergraph.add_edge(targets).unwrap();
		}

		if !hypergraph.fully_connected() {
			return Err(ConnectivityCreationError::NotFullyConnected);
		}

		let explosion = hypergraph.explode();
		Ok(Self {
			hypergraph,
			explosion,
			max_operator_size,
			qubit_count,
		})
	}

	/// # Create Line
	///
	/// Creates a line connectivity with minimal overlap.
	pub fn create_line(group_size: NonZeroEvenUsize, min_qubit_count: usize) -> Self {
		if min_qubit_count == 0 {
			return Self::new(0, vec![]).unwrap();
		}

		let group_size = group_size.as_value();

		// first has group_size. others have group_size -1
		let attached = min_qubit_count.saturating_sub(group_size);
		let n_groups = (attached + group_size - 2) / (group_size - 1) + 1;

		let qubit_count = 1 + n_groups * (group_size - 1);

		let mut operator_groups: Vec<Vec<usize>> = Vec::new();
		for i in 0..n_groups {
			let start = i * (group_size - 1);
			let group: Vec<usize> = (start..(start + group_size)).collect();
			operator_groups.push(group);
		}

		Self::new(qubit_count, operator_groups).unwrap()
	}

	/// # Create Square Grid
	///
	/// Creates a square grid connectivity with minimal overlap.
	pub fn create_square_grid(group_size: NonZeroEvenUsize, min_qubit_count: usize) -> Self {
		if min_qubit_count == 0 {
			return Self::new(0, vec![]).unwrap();
		}

		let group_size = group_size.as_value();

		let mut layers = 2; // same as rows and columns
		let mut n_qubits = 4 * (group_size - 1);

		let side_add = group_size - 1 + group_size - 2;
		let outer_corner = 2 * (group_size - 1) + group_size - 2; // add 2 times
		let inner_corner = 1 + 2 * (group_size - 2); // add 1 times

		while n_qubits < min_qubit_count {
			n_qubits += 2 * outer_corner;
			n_qubits += inner_corner;
			n_qubits += 2 * (layers - 2) * side_add;
			layers += 1;
		}

		// Layering order
		//  0  1  2  3
		//  4  5  6  7
		//  8  9 10 11
		// 12 13 14 15
		let mut operator_groups: Vec<Vec<usize>> = Vec::new();

		for group in 0..(layers - 1) {
			let offset = group * (group_size - 1);
			let group: Vec<usize> = (offset..(offset + group_size)).collect();
			operator_groups.push(group);
		}

		let qubits_in_row = 1 + (layers - 1) * (group_size - 1);
		let layer_size_in_qubits = qubits_in_row + layers * (group_size - 2);

		for row in 1..layers {
			let row_offset = row * layer_size_in_qubits;
			for group in 0..(layers - 1) {
				let offset = row_offset + group * (group_size - 1);
				let group: Vec<usize> = (offset..(offset + group_size)).collect();
				operator_groups.push(group);
			}

			let last_offset = (row - 1) * layer_size_in_qubits;
			for col in 0..layers {
				let end = row_offset + col * (group_size - 1);
				let start = end - layer_size_in_qubits;
				let between_start = last_offset + qubits_in_row + col * (group_size - 2);
				let between_end = last_offset + qubits_in_row + (col + 1) * (group_size - 2) - 1;

				let mut connections = vec![start];
				connections.append(&mut (between_start..=between_end).collect());
				connections.push(end);
				operator_groups.push(connections);
			}
		}

		Self::new(n_qubits, operator_groups).unwrap()
	}

	pub fn supports_operation_on(&self, targets: &[usize]) -> bool {
		if targets.is_empty() {
			return true;
		}

		let first = *targets.first().unwrap();
		if first >= self.qubit_count {
			return false;
		}

		'options: for option in self
			.hypergraph
			.get_node(first)
			.as_ref()
			.unwrap()
			.edges
			.iter()
		{
			let edge = self.hypergraph.get_edge(*option).unwrap();
			for target in targets.iter() {
				if !edge.nodes.contains(target) {
					continue 'options;
				}
			}
			return true;
		}

		false
	}

	pub fn get_routing_path<'a: 'b, 'b>(
		&'a self,
		targets: &[usize],
	) -> Vec<RoutingInstruction<'b>> {
		let mut terminals = Vec::new();
		for index in self.explosion.node_indices() {
			let weight = self.explosion.node_weight(index).unwrap();
			for node in weight.hyper_nodes.iter() {
				if targets.contains(node) {
					terminals.push(index);
					break;
				}
			}
		}

		// With terminals len 1 the steiner tree algorithm fails for some reason
		// (wrong answer as in giving empty tree)
		let primitive_groups = if terminals.len() == 1 {
			let node_things = self
				.explosion
				.node_weight(*terminals.first().unwrap())
				.unwrap();
			let edge = *node_things.hyper_edges.first().unwrap();

			vec![(edge, None)]
		} else {
			let tree = {
				let mut tree = steiner_tree(&self.explosion, &terminals);
				enforce_tree(&mut tree, &terminals);
				tree
			};
			explosion::as_instructions(tree)
		};

		let mut instructions = Vec::new();
		for (edge, nodes) in primitive_groups {
			#[allow(clippy::unnecessary_unwrap)]
			let target = if nodes.is_none() {
				RoutingInstructionTarget::Any
			} else if nodes.as_ref().unwrap().len() == 1 {
				RoutingInstructionTarget::Single(*nodes.unwrap().first().unwrap())
			} else {
				RoutingInstructionTarget::Multiple(nodes.unwrap().clone())
			};
			let qubits = self.hypergraph.get_edge(edge).unwrap().nodes.as_slice();
			instructions.push(RoutingInstruction { qubits, target });
		}

		if instructions.len() == 1 {
			instructions[0].target = RoutingInstructionTarget::Any;
		}

		instructions
	}

	pub fn max_operator_size(&self) -> usize {
		self.max_operator_size
	}

	pub fn qubit_count(&self) -> usize {
		self.qubit_count
	}
}

#[cfg(test)]
mod test {
	use crate::connectivity::Connectivity;

	#[test]
	fn test_supports_operation_on() {
		let connectivity = Connectivity::new(
			12,
			vec![
				vec![1, 2, 3],
				vec![1, 4, 5, 6],
				vec![2, 4],
				vec![6, 4, 7],
				vec![6, 8, 9, 10],
				vec![1, 11, 0],
			],
		)
		.unwrap();

		assert!(connectivity.supports_operation_on(&[1, 2, 3]));
		assert!(!connectivity.supports_operation_on(&[1, 2, 4]));
		assert!(!connectivity.supports_operation_on(&[12]));
		assert!(connectivity.supports_operation_on(&[1, 5, 4, 6]));
		assert!(connectivity.supports_operation_on(&[]))
	}
}
