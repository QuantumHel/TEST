use core::{Compiler, MapQubits, OperatorEq, TargetQubits, qubit_map::PartialQubitMap};
use std::collections::{BTreeMap, BTreeSet};

use crate::Circuit;

/// Finds patterns and replaces them with another pattern.
///
/// This [Compiler] ignores the given device, and trusts that the replacement
/// follows restraints.
///
/// # Important!
/// After each replacement, he circuit is inspected again from the beginning,
/// this means that this compiler step can lead to an infinite loop.
///
/// # Known issue (will be fixed when needed):
///
/// We pick gates into the pattern greedily. This means that a pattern
///
///
/// `H(0) H(1) CX(0, 1) H(0) H(1)`
///
/// fails to match on
///
/// `H(0) H(3) H(1) CX(0, 1) H(0) H(1)`
///
/// because we map 3 to 1.
pub struct FindAndReplace<Find, Replace> {
	find: Circuit<Find>,
	replace: Circuit<Replace>,
}

#[derive(Debug)]
pub enum FindAndReplaceError {
	FindIsEmpty,
	ReplaceUsedNonFindQubit { qubit: usize },
}

impl<Find: TargetQubits, Replace: TargetQubits> FindAndReplace<Find, Replace> {
	/// Errors when find is empty, or replace uses qubits not specified in find.
	pub fn new(
		find: Circuit<Find>,
		replace: Circuit<Replace>,
	) -> Result<Self, FindAndReplaceError> {
		if find.is_empty() {
			return Err(FindAndReplaceError::FindIsEmpty);
		}
		let find_qubits: BTreeSet<usize> = find
			.gates
			.iter()
			.flat_map(|g| g.target_qubits())
			.copied()
			.collect();
		for qubit in replace.gates.iter().flat_map(|g| g.target_qubits()) {
			if !find_qubits.contains(qubit) {
				return Err(FindAndReplaceError::ReplaceUsedNonFindQubit { qubit: *qubit });
			}
		}
		Ok(Self { find, replace })
	}
}

enum Edit {
	Before,
	After,
	Remove,
}

impl<T: TargetQubits + OperatorEq + MapQubits, D, Find: Clone + Into<T>, Replace: Clone + Into<T>>
	Compiler<Circuit<T>, Circuit<T>, D> for FindAndReplace<Find, Replace>
{
	fn compile(&self, mut input: Circuit<T>, _: &D) -> Circuit<T> {
		let find: Circuit<T> = Circuit {
			gates: self.find.gates.iter().map(|a| a.clone().into()).collect(),
		};

		let find_start = find.gates.first().unwrap();

		let mut i = 0;
		'outer: while i < input.len() {
			let start_gate = input.gates.get(i).unwrap();

			// We should not need to check target qubit size, but we do to make sure.
			if !start_gate.operator_eq(find_start)
				|| find_start.target_qubits().count() != start_gate.target_qubits().count()
			{
				i += 1;
				continue;
			}

			// map from pattern to real
			let mut mapping = PartialQubitMap::default();
			// Contains qubits in input format. These can not be used for before
			// and not for patterns. Only push after
			let mut blocked: BTreeSet<usize> = BTreeSet::default();
			let mut is_mapped: BTreeSet<usize> = BTreeSet::default();
			// Edits
			let mut edits: BTreeMap<usize, Edit> = BTreeMap::default();
			let mut input_index = i;
			let mut find_index = 0;
			'find_index: while find_index < find.gates.len() {
				let goal = find.gates.get(find_index).unwrap();
				let will_be_used: BTreeSet<usize> = find
					.iter()
					.skip(find_index)
					.flat_map(|g| g.target_qubits())
					.cloned()
					.collect();

				while input_index < input.len() {
					let next = input.gates.get(input_index).unwrap();
					if next.operator_eq(goal)
						&& next.target_qubits().count() == goal.target_qubits().count()
					{
						let mut is_match = true;
						let mut tmp_mapped: BTreeSet<usize> = BTreeSet::default();
						for (next_qubit, goal_qubit) in
							next.target_qubits().zip(goal.target_qubits())
						{
							if blocked.contains(next_qubit) {
								is_match = false;
								break;
							}
							let mapped = if is_mapped.contains(goal_qubit)
								|| tmp_mapped.contains(goal_qubit)
							{
								mapping.map(*goal_qubit).unwrap()
							} else {
								tmp_mapped.insert(*goal_qubit);
								if mapping.map_qubit(*goal_qubit, *next_qubit).is_err() {
									is_match = false;
									break;
								}
								*next_qubit
							};
							if mapped != *next_qubit {
								is_match = false;
								break;
							}
						}

						if is_match {
							for qubit in tmp_mapped.into_iter() {
								is_mapped.insert(qubit);
							}
							edits.insert(input_index, Edit::Remove);
							find_index += 1;
							input_index += 1;
							continue 'find_index;
						} else {
							for qubit in tmp_mapped.into_iter() {
								mapping.reset_qubit(qubit);
							}
						}
					}

					let mut after: bool = false;
					for qubit in next.target_qubits() {
						if let Ok(goal_qubit) = mapping.source(*qubit)
							&& is_mapped.contains(&goal_qubit)
						{
							after = true;
						}

						if blocked.contains(qubit) {
							after = true;
						}
					}

					if after {
						for qubit in next.target_qubits() {
							// if the qubit will ne used we cant put it after
							if let Ok(goal_qubit) = mapping.source(*qubit)
								&& is_mapped.contains(&goal_qubit)
								&& will_be_used.contains(&goal_qubit)
							{
								i += 1;
								continue 'outer;
							}
							blocked.insert(*qubit);
						}

						edits.insert(input_index, Edit::After);
					} else {
						edits.insert(input_index, Edit::Before);
					}

					input_index += 1;
				}
				i += 1;
				continue 'outer;
			}

			let mut replace_with: Vec<_> = self
				.replace
				.gates
				.iter()
				.map(|a| {
					let mut gate = a.clone().into();
					gate.try_map_qubits(&mapping).unwrap();
					gate
				})
				.collect();
			let mut before: Vec<T> = Vec::new();
			let mut after: Vec<T> = Vec::new();

			for (index, edit) in edits.into_iter().rev() {
				let gate = input.gates.remove(index);
				match edit {
					Edit::Before => {
						before.push(gate);
					}
					Edit::After => {
						after.push(gate);
					}
					Edit::Remove => {}
				}
			}

			before.append(&mut replace_with);
			before.append(&mut after);
			input.gates.splice(i..i, before);
			i = 0;
		}

		input
	}
}

#[cfg(test)]
mod tests {
	use core::Compiler;

	use crate::{Circuit, gates::CNot};

	use super::FindAndReplace;

	#[test]
	fn find_and_replace_test_1() {
		let circuit = Circuit {
			gates: vec![
				CNot::new(1, 3).unwrap(),
				CNot::new(0, 2).unwrap(),
				CNot::new(2, 4).unwrap(),
				CNot::new(0, 2).unwrap(),
				CNot::new(1, 3).unwrap(),
			],
		};
		let find_and_replace: FindAndReplace<CNot, CNot> = FindAndReplace::new(
			Circuit {
				gates: vec![CNot::new(0, 1).unwrap(), CNot::new(0, 1).unwrap()],
			},
			Circuit { gates: vec![] },
		)
		.unwrap();

		assert_eq!(
			find_and_replace.compile(circuit, &()).gates,
			vec![
				CNot::new(0, 2).unwrap(),
				CNot::new(2, 4).unwrap(),
				CNot::new(0, 2).unwrap(),
			]
		);
	}

	#[test]
	fn find_and_replace_test_2() {
		let circuit = Circuit {
			gates: vec![
				CNot::new(0, 3).unwrap(),
				CNot::new(1, 4).unwrap(),
				CNot::new(0, 2).unwrap(),
				CNot::new(0, 2).unwrap(),
				CNot::new(0, 3).unwrap(),
			],
		};
		let find_and_replace: FindAndReplace<CNot, CNot> = FindAndReplace::new(
			Circuit {
				gates: vec![CNot::new(0, 1).unwrap(), CNot::new(0, 1).unwrap()],
			},
			Circuit { gates: vec![] },
		)
		.unwrap();

		assert_eq!(
			find_and_replace.compile(circuit, &()).gates,
			vec![CNot::new(1, 4).unwrap()]
		);
	}

	#[test]
	fn find_and_replace_test_3() {
		let circuit = Circuit {
			gates: vec![
				CNot::new(0, 1).unwrap(),
				CNot::new(1, 2).unwrap(),
				CNot::new(2, 3).unwrap(),
				CNot::new(1, 2).unwrap(),
				CNot::new(0, 1).unwrap(),
			],
		};
		let find_and_replace: FindAndReplace<CNot, CNot> = FindAndReplace::new(
			Circuit {
				gates: vec![
					CNot::new(0, 1).unwrap(),
					CNot::new(1, 2).unwrap(),
					CNot::new(0, 1).unwrap(),
				],
			},
			Circuit {
				gates: vec![CNot::new(0, 1).unwrap(), CNot::new(1, 0).unwrap()],
			},
		)
		.unwrap();

		assert_eq!(
			find_and_replace.compile(circuit, &()).gates,
			vec![
				CNot::new(0, 1).unwrap(),
				CNot::new(1, 2).unwrap(),
				CNot::new(2, 1).unwrap(),
				CNot::new(0, 1).unwrap(),
			]
		);
	}

	#[test]
	fn find_and_replace_location() {
		let circuit = Circuit {
			gates: vec![
				CNot::new(0, 1).unwrap(),
				CNot::new(3, 2).unwrap(),
				CNot::new(1, 2).unwrap(),
			],
		};
		let find_and_replace: FindAndReplace<CNot, CNot> = FindAndReplace::new(
			Circuit {
				gates: vec![CNot::new(0, 1).unwrap(), CNot::new(1, 2).unwrap()],
			},
			Circuit {
				gates: vec![CNot::new(0, 2).unwrap()],
			},
		)
		.unwrap();

		assert_eq!(
			find_and_replace.compile(circuit, &()).gates,
			vec![CNot::new(3, 2).unwrap(), CNot::new(0, 2).unwrap()]
		);
	}

	#[test]
	fn find_and_replace_non_trivial_block() {
		let circuit = Circuit {
			gates: vec![
				CNot::new(1, 2).unwrap(),
				CNot::new(1, 0).unwrap(),
				CNot::new(3, 0).unwrap(),
				CNot::new(2, 3).unwrap(),
			],
		};
		let find_and_replace: FindAndReplace<CNot, CNot> = FindAndReplace::new(
			Circuit {
				gates: vec![CNot::new(0, 1).unwrap(), CNot::new(1, 2).unwrap()],
			},
			Circuit { gates: vec![] },
		)
		.unwrap();

		assert_eq!(
			find_and_replace.compile(circuit, &()).gates,
			vec![
				CNot::new(1, 2).unwrap(),
				CNot::new(1, 0).unwrap(),
				CNot::new(3, 0).unwrap(),
				CNot::new(2, 3).unwrap(),
			]
		);
	}
}
