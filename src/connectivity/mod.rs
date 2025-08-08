mod explosion;
mod hypergraph;

use bitvec::vec::BitVec;
use petgraph::graph::UnGraph;

use crate::connectivity::{explosion::ExplosionNode, hypergraph::HyperGraph};

pub struct OperationGroupIndex(usize);

pub enum ConnectivityInstructionTarget {
	Single(u32),
	Multiple(Vec<u32>),
	Any,
}

pub struct ConnectivityInstruction {
	pub index: OperationGroupIndex,
	pub target: ConnectivityInstructionTarget,
}

pub struct Connectivity<const N: usize> {
	hypergraph: HyperGraph<usize>,
	explosion: UnGraph<ExplosionNode, ()>,
}

impl<const N: usize> Connectivity<N> {
	pub fn new(operator_groups: Vec<Vec<u32>>) -> Option<Self> {
		todo!()
	}
}
