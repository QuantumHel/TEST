use std::{
	cmp::Ordering,
	collections::{BTreeSet, BinaryHeap},
};

use bits::Bits;
use circuit::gates::CNot;

use crate::t_par::ParityVisitor;

#[derive(Debug, Clone)]
pub struct QueueItem {
	cnots: Vec<CNot>,
	required: Vec<Bits>,
	optional: Vec<Bits>,
}

impl QueueItem {
	/// Returns ture if required contains the given target bit.
	fn does_not_need(&self, target: usize) -> bool {
		for bits in self.required.iter() {
			if bits.get(target) {
				return false;
			}
		}

		true
	}

	/// Returns the sum of required.len() and optional.len()
	fn total_len(&self) -> usize {
		self.required.len() + self.optional.len()
	}
}

impl PartialEq for QueueItem {
	fn eq(&self, other: &Self) -> bool {
		if self.cnots.len() != other.cnots.len() {
			return false;
		}

		if self.total_len() != other.total_len() {
			return false;
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

		self_ones == other_ones
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
			return Ordering::Greater;
		} else if other.cnots.len() < self.cnots.len() {
			return Ordering::Less;
		}

		if self.total_len() < other.total_len() {
			return Ordering::Greater;
		} else if other.total_len() < self.total_len() {
			return Ordering::Less;
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

		other_ones.cmp(&self_ones)
	}
}

fn find_path(unsolved: &BTreeSet<usize>, item: QueueItem) -> (usize, QueueItem) {
	let mut queue = BinaryHeap::from([item]);

	while let Some(item) = queue.pop() {
		for target in unsolved.iter() {
			if item.does_not_need(*target) {
				return (*target, item);
			}
		}

		for target in unsolved.iter() {
			for control in unsolved.iter() {
				if *control == *target {
					continue;
				}

				let cnot = CNot::new(*control, *target).unwrap();
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

				let mut cnots = item.cnots.clone();
				cnots.push(cnot);

				queue.push(QueueItem {
					cnots,
					required,
					optional,
				});
			}
		}
	}

	unreachable!()
}

pub struct GrayStar;

impl ParityVisitor<()> for GrayStar {
	fn visit(&self, mut required: Vec<Bits>, mut optional: Vec<Bits>, _: &()) -> Vec<CNot> {
		let mut unsolved: BTreeSet<usize> = BTreeSet::new();
		// TODO: Check if this should only contain required
		for bits in required.iter().chain(optional.iter()) {
			for i in bits.iter_ones() {
				unsolved.insert(i);
			}
		}
		let mut result: Vec<CNot> = Vec::new();
		while !required.is_empty() {
			assert!(!unsolved.is_empty());
			// (qubit, item)
			let (i, mut item) = find_path(
				&unsolved,
				QueueItem {
					cnots: Vec::new(),
					required: required.clone(),
					optional: optional.clone(),
				},
			);

			result.append(&mut item.cnots);
			required = item.required;
			optional = item.optional;
			unsolved.remove(&i);
		}

		assert!(required.is_empty());
		result
	}
}
