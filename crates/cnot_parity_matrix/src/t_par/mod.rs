mod parity;
mod state;
mod triplet;
mod xor_span;

use std::collections::BTreeSet;

use bits::Bits;
use test_core::Compiler;

use crate::{CNot, CNotRzXYH, ParityMatrix, Rz, X};
use parity::Parity;

use self::{state::State, triplet::Triplet};

pub struct HadamardTransform {
	target: usize,
	input: State,
	output: State,
}

/// Generic implementation of TPar in https://arxiv.org/pdf/1303.2042
#[derive(Default)]
pub struct TPar<V: ParityVisitor, M: Compiler<Input = ParityMatrix, Output = Vec<CNot>>> {
	visitor: V,
	parity_solver: M,
}

impl<V: ParityVisitor, M: Compiler<Input = ParityMatrix, Output = Vec<CNot>>> TPar<V, M> {
	pub fn new(visitor: V, parity_solver: M) -> Self {
		Self {
			visitor,
			parity_solver,
		}
	}
}

impl<V: ParityVisitor, M: Compiler<Input = ParityMatrix, Output = Vec<CNot>>> Compiler
	for TPar<V, M>
{
	type Input = Vec<CNotRzXYH>;
	type Output = Vec<CNotRzXYH>;

	fn compile(&self, input: Self::Input) -> Self::Output {
		let n = input
			.iter()
			.filter_map(|gate| {
				if let CNotRzXYH::Rz(Rz { target, .. }) = gate {
					Some(target + 1)
				} else {
					None
				}
			})
			.max()
			.unwrap_or_default();

		#[allow(non_snake_case)]
		let Triplet {
			s: mut S,
			q: Q,
			h: H,
		} = {
			let mut triplet = Triplet::new(n);
			for gate in input {
				triplet.add_gate(gate);
			}

			triplet
		};
		let mut output = Vec::new();
		let mut state = State::new(n);

		for i in 0..n {
			if let Some(angle) = S.remove(&Parity::for_qubit(i)) {
				output.push(CNotRzXYH::Rz(Rz { angle, target: i }));
			}

			if let Some(angle) = S.remove(&Parity::for_qubit(i).not()) {
				state.apply_x(i);
				output.push(CNotRzXYH::X(X { target: i }));
				output.push(CNotRzXYH::Rz(Rz { angle, target: i }));
			}
		}

		for h in H {
			let mut nots_to_apply = Vec::new();
			for (i, parity) in state.parities().iter().enumerate() {
				if let Some(angle) = S.remove(parity) {
					output.push(CNotRzXYH::Rz(Rz { angle, target: i }));
				}

				if let Some(angle) = S.remove(&parity.clone().not()) {
					nots_to_apply.push(i);
					output.push(CNotRzXYH::X(X { target: i }));
					output.push(CNotRzXYH::Rz(Rz { angle, target: i }));
				}
			}

			for i in nots_to_apply {
				state.apply_x(i);
			}

			let state_span = state.create_span();
			let supported = S.iter().filter(|(parity, _)| state_span.supports(parity));
			let output_span = h.output.create_span();
			let (optional, required): (Vec<_>, Vec<_>) =
				supported.partition(|(parity, _)| output_span.supports(parity));

			let optional_bits: Vec<_> = optional
				.iter()
				.map(|(parity, _)| parity.using_span(&state_span).unwrap())
				.collect();
			let required_bits: Vec<_> = required
				.iter()
				.map(|(parity, _)| parity.using_span(&state_span).unwrap())
				.collect();

			let cnots = self.visitor.visit(required_bits, optional_bits);
			let mut required: BTreeSet<_> = required.into_iter().map(|(a, _)| a.clone()).collect();
			for cnot in cnots {
				state.apply_cnot(cnot.control, cnot.target);
				output.push(CNotRzXYH::CNot(cnot));

				let parity = state.get_cloned(cnot.target);
				if let Some(angle) = S.remove(&parity) {
					required.remove(&parity);
					output.push(CNotRzXYH::Rz(Rz {
						angle,
						target: cnot.target,
					}));
				}

				let not_parity = state.get_cloned(cnot.target).not();
				if let Some(angle) = S.remove(&not_parity) {
					state.apply_x(cnot.target);
					required.remove(&parity);
					output.push(CNotRzXYH::X(X {
						target: cnot.target,
					}));
					output.push(CNotRzXYH::Rz(Rz {
						angle,
						target: cnot.target,
					}));
				}
			}
			assert!(
				required.is_empty(),
				"Visitor does not visit all required parities"
			);

			let state_span = state.create_span();
			let parity_rows: Vec<_> = h
				.input
				.parities()
				.iter()
				.map(|parity| parity.using_span(&state_span).unwrap())
				.collect();
			let parity_matrix = ParityMatrix::standard_from_rows(parity_rows);
			for cnot in self.parity_solver.compile(parity_matrix) {
				state.apply_cnot(cnot.control, cnot.target);
				output.push(CNotRzXYH::CNot(cnot));

				let parity = state.get_cloned(cnot.target);
				if let Some(angle) = S.remove(&parity) {
					output.push(CNotRzXYH::Rz(Rz {
						angle,
						target: cnot.target,
					}));
				}

				let not_parity = state.get_cloned(cnot.target).not();
				if let Some(angle) = S.remove(&not_parity) {
					state.apply_x(cnot.target);
					output.push(CNotRzXYH::X(X {
						target: cnot.target,
					}));
					output.push(CNotRzXYH::Rz(Rz {
						angle,
						target: cnot.target,
					}));
				}
			}

			for (current, goal) in state
				.mut_parities()
				.iter_mut()
				.zip(h.input.parities().iter())
			{
				if current.bit_flip != goal.bit_flip {
					current.bit_flip = goal.bit_flip
				}

				assert_eq!(current, goal, "ParityMatrix solving failed");
			}

			output.push(CNotRzXYH::H(crate::gates::H { target: h.target }));
			state = h.output;
		}

		// resolve rest. mostly copy pasted from above
		{
			let mut nots_to_apply = Vec::new();
			for (i, parity) in state.parities().iter().enumerate() {
				if let Some(angle) = S.remove(parity) {
					output.push(CNotRzXYH::Rz(Rz { angle, target: i }));
				}

				if let Some(angle) = S.remove(&parity.clone().not()) {
					nots_to_apply.push(i);
					output.push(CNotRzXYH::X(X { target: i }));
					output.push(CNotRzXYH::Rz(Rz { angle, target: i }));
				}
			}

			for i in nots_to_apply {
				state.apply_x(i);
			}

			let state_span = state.create_span();
			let required_bits: Vec<_> = S
				.keys()
				.map(|parity| parity.using_span(&state_span).unwrap())
				.collect();

			let cnots = self.visitor.visit(required_bits, Vec::new());

			for cnot in cnots {
				state.apply_cnot(cnot.control, cnot.target);
				output.push(CNotRzXYH::CNot(cnot));

				let parity = state.get_cloned(cnot.target);
				if let Some(angle) = S.remove(&parity) {
					output.push(CNotRzXYH::Rz(Rz {
						angle,
						target: cnot.target,
					}));
				}

				let not_parity = parity.not();
				if let Some(angle) = S.remove(&not_parity) {
					state.apply_x(cnot.target);
					output.push(CNotRzXYH::X(X {
						target: cnot.target,
					}));
					output.push(CNotRzXYH::Rz(Rz {
						angle,
						target: cnot.target,
					}));
				}
			}
			// HERE
			assert!(S.is_empty(), "Visitor does not visit all required parities");

			let state_span = state.create_span();
			let parity_rows: Vec<_> = Q
				.parities()
				.iter()
				.map(|parity| parity.using_span(&state_span).unwrap())
				.collect();
			let parity_matrix = ParityMatrix::standard_from_rows(parity_rows);
			for cnot in self.parity_solver.compile(parity_matrix) {
				state.apply_cnot(cnot.control, cnot.target);
				output.push(CNotRzXYH::CNot(cnot));
			}

			let mut indicies = 0..(state.parities().len().max(Q.parities().len()));
			let mut parities = state.mut_parities().iter_mut();
			let mut goals = Q.parities().iter();
			loop {
				match (indicies.next(), parities.next(), goals.next()) {
					(Some(i), Some(current), Some(goal)) => {
						if current.bit_flip != goal.bit_flip {
							current.bit_flip = goal.bit_flip;
							output.push(CNotRzXYH::X(X { target: i }));
						}

						assert_eq!(*current, *goal, "ParityMatrix solving failed");
					}
					(Some(i), Some(current), None) => {
						if current.bit_flip {
							current.bit_flip = false;
							output.push(CNotRzXYH::X(X { target: i }));
						}

						assert_eq!(
							*current,
							Parity::for_qubit(i),
							"ParityMatrix solving failed"
						);
					}
					(Some(i), None, Some(goal)) => {
						if goal.bit_flip {
							output.push(CNotRzXYH::X(X { target: i }));
							assert_eq!(
								*goal,
								Parity::for_qubit(i).not(),
								"ParityMatrix solving failed"
							);
						} else {
							assert_eq!(*goal, Parity::for_qubit(i), "ParityMatrix solving failed");
						}
					}
					(None, _, _) => {
						break;
					}
					_ => unreachable!(),
				}
			}
		}

		output
	}
}

pub trait ParityVisitor {
	fn visit(&self, required: Vec<Bits>, optional: Vec<Bits>) -> Vec<CNot>;
}

#[cfg(test)]
mod tests {
	use std::num::NonZeroU32;

	use test_core::Compiler;

	use crate::{Angle, CNot, CNotRzXYH, H, PatelMarkovHayes, Rz, gra_star_synth::GrayStar};

	use super::TPar;

	#[test]
	fn exploration() {
		let circuit: Vec<CNotRzXYH> = vec![
			// --- First Block (Qubits 0, 1, 2) ---
			CNotRzXYH::H(H { target: 2 }),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(1),
				target: 0,
			}),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(1),
				target: 1,
			}),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(1),
				target: 2,
			}),
			CNotRzXYH::CNot(CNot {
				control: 1,
				target: 0,
			}),
			CNotRzXYH::CNot(CNot {
				control: 2,
				target: 1,
			}),
			CNotRzXYH::CNot(CNot {
				control: 0,
				target: 2,
			}),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(7),
				target: 1,
			}),
			CNotRzXYH::CNot(CNot {
				control: 0,
				target: 1,
			}),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(7),
				target: 0,
			}),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(7),
				target: 1,
			}),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(1),
				target: 2,
			}),
			CNotRzXYH::CNot(CNot {
				control: 2,
				target: 1,
			}),
			CNotRzXYH::CNot(CNot {
				control: 0,
				target: 2,
			}),
			CNotRzXYH::CNot(CNot {
				control: 1,
				target: 0,
			}),
			CNotRzXYH::H(H { target: 2 }),
			// --- Second Block (Qubits 1, 2, 3) ---
			CNotRzXYH::H(H { target: 3 }),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(1),
				target: 1,
			}),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(1),
				target: 2,
			}),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(1),
				target: 3,
			}),
			CNotRzXYH::CNot(CNot {
				control: 2,
				target: 1,
			}),
			CNotRzXYH::CNot(CNot {
				control: 3,
				target: 2,
			}),
			CNotRzXYH::CNot(CNot {
				control: 1,
				target: 3,
			}),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(7),
				target: 2,
			}),
			CNotRzXYH::CNot(CNot {
				control: 1,
				target: 2,
			}),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(7),
				target: 1,
			}),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(7),
				target: 2,
			}),
			CNotRzXYH::Rz(Rz {
				angle: Angle::QuarterPi(1),
				target: 3,
			}),
			CNotRzXYH::CNot(CNot {
				control: 3,
				target: 2,
			}),
			CNotRzXYH::CNot(CNot {
				control: 1,
				target: 3,
			}),
			CNotRzXYH::CNot(CNot {
				control: 2,
				target: 1,
			}),
			CNotRzXYH::H(H { target: 3 }),
		];

		let tpar = TPar::new(GrayStar, PatelMarkovHayes::new(NonZeroU32::new(2).unwrap()));
		let res = tpar.compile(circuit);
		dbg!(res);
	}
}
