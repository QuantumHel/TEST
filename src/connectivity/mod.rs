mod explosion;
pub(crate) mod hypergraph;

use std::collections::BTreeSet;

pub(crate) use crate::connectivity::{explosion::ExplosionNode, hypergraph::HyperGraph};
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
	/// Creates a line connectivity.
	///
	/// ## Panics
	/// if group size is smaller than 2.
	pub fn create_line(group_size: usize, length: usize) -> Self {
		if group_size < 2 {
			panic!("Can not create line connectivity with group_size smalle than 2!");
		}

		if length == 0 {
			return Self::new(0, vec![]).unwrap();
		}

		let qubit_count = 1 + length * (group_size - 1);

		let mut operator_groups: Vec<Vec<usize>> = Vec::new();
		for i in 0..length {
			let start = i * (group_size - 1);
			let group: Vec<usize> = (start..(start + group_size)).collect();
			operator_groups.push(group);
		}

		Self::new(qubit_count, operator_groups).unwrap()
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
			let tree = steiner_tree(&self.explosion, &terminals);
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
