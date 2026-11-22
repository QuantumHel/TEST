use std::{collections::BTreeSet, iter};

use crate::t_par::ParityVisitor;
use bits::Bits;
use circuit::gates::CNot;

/// A [ParityVisitor] following the GraySynth algorithm from Matthew Amy et. al.
/// 2019 Quantum Sci. Technol. 4
///
/// DOI: https://doi.org/10.1088/2058-9565/aad8ca
///
/// Arxiv: https://arxiv.org/abs/1712.01859
pub struct GraySynth;

struct StackItem {
	parities: Vec<Bits>,
	unused: BTreeSet<usize>,
	target: Option<usize>,
}

impl ParityVisitor<()> for GraySynth {
	fn visit(&self, required: Vec<Bits>, _: Vec<Bits>, _: &()) -> Vec<CNot> {
		let Some(n) = required
			.iter()
			.filter_map(Bits::last_one)
			.max()
			.map(|i| i + 1)
		else {
			return Vec::new();
		};
		let mut circuit: Vec<CNot> = Vec::new();
		let mut stack: Vec<StackItem> = vec![StackItem {
			parities: required,
			unused: (0..n).collect(),
			target: None,
		}];

		while let Some(StackItem {
			mut parities,
			unused,
			target,
		}) = stack.pop()
		{
			if parities.is_empty() {
				continue;
			}

			if let Some(target) = target {
				for j in 0..n {
					if j == target || !parities.iter().all(|p| p.get(j)) {
						continue;
					}

					// This is a bit other way around as when changing paritymatrix
					// that is because this is not the state we have, but the state
					// where we "want to go".
					circuit.push(CNot::new(j, target).unwrap());

					for parities in stack
						.iter_mut()
						.map(|s| &mut s.parities)
						.chain(iter::once(&mut parities))
					{
						for parity in parities {
							if parity.get(target) {
								parity.flip(j);
							}
						}
					}
				}
			}

			let Some(j) = unused.iter().copied().max_by_key(|j| {
				let ones = parities.iter().filter(|p| p.get(*j)).count();
				let zeros = parities.len() - ones;
				ones.max(zeros)
			}) else {
				continue;
			};

			let (with_1, with_0): (Vec<_>, Vec<_>) = parities.into_iter().partition(|p| p.get(j));

			let mut new_unused = unused.clone();
			new_unused.remove(&j);

			stack.push(StackItem {
				parities: with_1,
				unused: new_unused.clone(),
				target: Some(target.unwrap_or(j)),
			});
			stack.push(StackItem {
				parities: with_0,
				unused: new_unused,
				target,
			});
		}

		circuit
	}
}
