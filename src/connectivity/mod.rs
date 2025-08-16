mod explosion;
mod hypergraph;

use std::collections::BTreeSet;

use crate::connectivity::{explosion::ExplosionNode, hypergraph::HyperGraph};
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
	hypergraph: HyperGraph,
	explosion: UnGraph<ExplosionNode, usize>,
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
					continue;
				}
			}
		}

		let tree = steiner_tree(&self.explosion, &terminals);
		let primitive_groups = explosion::as_instructions(tree);
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
