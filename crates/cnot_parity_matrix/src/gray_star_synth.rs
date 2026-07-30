use std::{
	cmp::Ordering,
	collections::{BTreeMap, BTreeSet, VecDeque, btree_map::Keys},
};

use bits::Bits;
use circuit::gates::CNot;
use test_core::connectivity::{Connectivity, ConnectivityNode, Subgraph, steiner_tree};

use crate::{TwoQubitEdge, t_par::ParityVisitor};

enum IteratorEnum<T1: Iterator<Item = usize>, T2: Iterator<Item = usize>> {
	T1(T1),
	T2(T2),
}

impl<T1: Iterator<Item = usize>, T2: Iterator<Item = usize>> Iterator for IteratorEnum<T1, T2> {
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::T1(t1) => t1.next(),
			Self::T2(t2) => t2.next(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct QueueItem {
	cnots: Vec<CNot>,
	unsolved: UnsolvedQubits,
	required: Vec<Bits>,
	optional: Vec<Bits>,
}

impl PartialEq for QueueItem {
	fn eq(&self, other: &Self) -> bool {
		self.cnots == other.cnots
			&& self.required == other.required
			&& self.optional == other.optional
	}
}

impl Eq for QueueItem {}

impl PartialOrd for QueueItem {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for QueueItem {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		if self.cnots.len() < other.cnots.len() {
			return Ordering::Less;
		} else if other.cnots.len() < self.cnots.len() {
			return Ordering::Greater;
		}

		// TODO: It is not trivial what ordering is best.
		// This should be parametrized.
		// vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv

		if self.optional.len() < other.optional.len() {
			return Ordering::Less;
		} else if other.optional.len() < self.optional.len() {
			return Ordering::Greater;
		}

		if self.required.len() < other.required.len() {
			return Ordering::Less;
		} else if other.required.len() < self.required.len() {
			return Ordering::Greater;
		}

		if self.unsolved.len() < other.unsolved.len() {
			return Ordering::Less;
		} else if other.unsolved.len() < self.unsolved.len() {
			return Ordering::Greater;
		}

		let self_ones: usize = self
			.required
			.iter()
			.chain(self.optional.iter())
			.map(|bits| bits.count_ones())
			.sum();

		let other_ones: usize = other
			.required
			.iter()
			.chain(other.optional.iter())
			.map(|bits| bits.count_ones())
			.sum();

		if self_ones < other_ones {
			return Ordering::Less;
		} else if other_ones < self_ones {
			return Ordering::Greater;
		}

		// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

		// The rest here is just to make sure that we are consistent with Eq
		self.cnots
			.cmp(&other.cnots)
			.then(self.required.cmp(&other.required))
			.then(self.optional.cmp(&other.optional))
	}
}

fn find_path(
	max_queue_size: &Option<usize>,
	termination_criteria: &GrayStarTerminationCriteria,
	connectivity: Option<&Subgraph<'_, ConnectivityNode, TwoQubitEdge>>,
	item: QueueItem,
) -> QueueItem {
	let original_len = item.required.len();
	let original_qubits = item.unsolved.len();
	let mut queue = BTreeSet::from([item]);

	while let Some(item) = queue.pop_first() {
		match termination_criteria {
			GrayStarTerminationCriteria::QubitRemoved(n) => {
				if original_qubits - item.unsolved.len() >= original_qubits.min(*n)
					|| item.required.is_empty()
				{
					return item;
				}
			}
			GrayStarTerminationCriteria::ParitiesVisited(n) => {
				if original_len - item.required.len() >= original_len.min(*n)
					|| item.required.is_empty()
				{
					return item;
				}
			}
			GrayStarTerminationCriteria::None => {
				if item.required.is_empty() {
					return item;
				}
			}
		}

		for target in if connectivity.is_some() {
			// In the case of unsolved we can have unsolved that are 0 and therefor do nothing.
			IteratorEnum::T1(
				item.required
					.iter()
					.flat_map(|parity| parity.iter_ones())
					.collect::<BTreeSet<_>>()
					.into_iter(),
			)
		} else {
			IteratorEnum::T2(item.unsolved.iter().copied())
		} {
			for control in if let Some(connectivity) = connectivity {
				let mut neighbors: Vec<usize> = Vec::new();
				for edge in connectivity.get_node(target).unwrap().edges() {
					let edge = connectivity.get_edge(*edge).unwrap();
					for neighbor in edge.nodes() {
						if *neighbor == target || !item.unsolved.contains(*neighbor) {
							continue;
						}
						neighbors.push(*neighbor);
					}
				}

				IteratorEnum::T1(neighbors.into_iter())
			} else {
				IteratorEnum::T2(item.unsolved.iter().copied())
			} {
				if control == target {
					continue;
				}

				let cnot = CNot::new(control, target).unwrap();
				if let Some(previous) = item.cnots.last()
					&& *previous == cnot
				{
					continue;
				}

				let required: Vec<_> = item
					.required
					.iter()
					.map(|bits| {
						let mut new_bits = bits.clone();
						if bits.get(cnot.target()) {
							new_bits.set(cnot.control(), !bits.get(cnot.control()));
						}
						new_bits
					})
					.filter(|bits| bits.count_ones() > 1)
					.collect();

				let optional: Vec<_> = item
					.optional
					.iter()
					.map(|bits| {
						let mut new_bits = bits.clone();
						if bits.get(cnot.target()) {
							new_bits.set(cnot.control(), !bits.get(cnot.control()));
						}
						new_bits
					})
					.filter(|bits| bits.count_ones() > 1)
					.collect();

				let mut unsolved = item.unsolved.clone();
				let unused: Vec<_> = unsolved
					.iter()
					.filter(|q| {
						for bits in required.iter() {
							if bits.get(**q) {
								return false;
							}
						}

						true
					})
					.copied()
					.collect();
				unsolved.remove_many(&unused);

				let mut cnots = item.cnots.clone();
				cnots.push(cnot);

				queue.insert(QueueItem {
					cnots,
					required,
					optional,
					unsolved,
				});

				if let Some(max_queue_size) = max_queue_size {
					while queue.len() > *max_queue_size {
						queue.pop_last();
					}
				}
			}
		}
	}

	unreachable!()
}

#[derive(Debug, Clone)]
struct UnsolvedQubits {
	/// Stores pairs (qubit, children)
	qubits: BTreeMap<usize, Vec<usize>>,
}

impl UnsolvedQubits {
	fn is_empty(&self) -> bool {
		self.qubits.is_empty()
	}

	fn len(&self) -> usize {
		self.qubits.len()
	}

	fn contains(&self, qubit: usize) -> bool {
		self.qubits.contains_key(&qubit)
	}

	fn iter(&self) -> Keys<'_, usize, Vec<usize>> {
		self.qubits.keys()
	}

	fn remove_many(&mut self, qubits: &[usize]) {
		let mut change = true;
		while change {
			change = false;
			'qubit: for qubit in qubits {
				if !self.qubits.contains_key(qubit) {
					continue;
				}

				let children = self
					.qubits
					.get(qubit)
					.expect("Accessing qubit that is not in unsolved");
				for child in children {
					if self.qubits.contains_key(child) {
						continue 'qubit;
					}
				}
				self.qubits.remove(qubit);
				change = true;
			}
		}
	}
}

pub enum GrayStarTerminationCriteria {
	QubitRemoved(usize),
	ParitiesVisited(usize),
	None,
}

pub struct GrayStarSynth {
	pub max_queue_size: Option<usize>,
	pub path_termination_criteria: GrayStarTerminationCriteria,
}

impl GrayStarSynth {
	fn generic_visit(
		&self,
		unsolved: BTreeMap<usize, Vec<usize>>,
		mut required: Vec<Bits>,
		mut optional: Vec<Bits>,
		connectivity: Option<&Subgraph<'_, ConnectivityNode, TwoQubitEdge>>,
	) -> Vec<CNot> {
		required.retain(|bits| bits.count_ones() > 1);
		optional.retain(|bits| bits.count_ones() > 1);

		let mut unsolved = UnsolvedQubits { qubits: unsolved };
		let mut result: Vec<CNot> = Vec::new();
		while !required.is_empty() {
			assert!(!unsolved.is_empty());
			let mut item = find_path(
				&self.max_queue_size,
				&self.path_termination_criteria,
				connectivity,
				QueueItem {
					unsolved: unsolved.clone(),
					cnots: Vec::new(),
					required: required.clone(),
					optional: optional.clone(),
				},
			);

			result.append(&mut item.cnots);
			unsolved = item.unsolved;
			required = item.required;
			optional = item.optional;
		}

		assert!(required.is_empty());
		result
	}
}

impl ParityVisitor<()> for GrayStarSynth {
	fn visit(&self, required: Vec<Bits>, optional: Vec<Bits>, _: &()) -> Vec<CNot> {
		let mut unsolved: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
		for bits in required.iter() {
			for i in bits.iter_ones() {
				unsolved.insert(i, Vec::new());
			}
		}

		self.generic_visit(unsolved, required, optional, None)
	}
}

impl ParityVisitor<Connectivity<TwoQubitEdge>> for GrayStarSynth {
	fn visit(
		&self,
		required: Vec<Bits>,
		optional: Vec<Bits>,
		device: &Connectivity<TwoQubitEdge>,
	) -> Vec<CNot> {
		if required.is_empty() {
			return Vec::new();
		}

		let mut needed_qubits: BTreeSet<usize> = BTreeSet::new();
		for bits in required.iter() {
			for i in bits.iter_ones() {
				needed_qubits.insert(i);
			}
		}
		if needed_qubits.is_empty() {
			return Vec::new();
		}
		let needed_qubits = needed_qubits.into_iter().collect::<Vec<_>>();

		let tree = steiner_tree(&needed_qubits, device);
		let root = *needed_qubits.first().unwrap();
		let mut queue: VecDeque<usize> = VecDeque::from([root]);
		let mut visited: BTreeSet<usize> = BTreeSet::new();
		let mut unsolved: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
		while let Some(node) = queue.pop_front() {
			visited.insert(node);
			let mut children: Vec<usize> = Vec::new();

			for edge in tree.get_node(node).unwrap().edges().iter() {
				let edge = tree.get_edge(*edge).unwrap();
				for child in edge.nodes() {
					// Node is also contained in visited
					if visited.contains(child) {
						continue;
					}
					children.push(*child);
					queue.push_back(*child);
				}
			}

			unsolved.insert(node, children);
		}

		self.generic_visit(unsolved, required, optional, Some(&tree))
	}
}
