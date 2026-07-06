mod gateset;
mod parity;
mod squirrel;
mod state;
mod triplet;
mod xor_span;

use std::collections::BTreeSet;

use bits::Bits;
use circuit::{
	Circuit,
	gates::{CNot, H, Rz, X},
};
use test_core::Compiler;

use gateset::CNotRzXYH;
use parity::Parity;

use crate::ParityMatrix;

use self::{state::State, triplet::Triplet};

pub struct HadamardTransform {
	target: usize,
	input: State,
	output: State,
}

/// Generic implementation of TPar in https://arxiv.org/pdf/1303.2042
#[derive(Default)]
pub struct TPar<V: ParityVisitor, M: Compiler<Input = ParityMatrix, Output = Circuit<CNot>>> {
	visitor: V,
	parity_solver: M,
}

impl<V: ParityVisitor, M: Compiler<Input = ParityMatrix, Output = Circuit<CNot>>> TPar<V, M> {
	pub fn new(visitor: V, parity_solver: M) -> Self {
		Self {
			visitor,
			parity_solver,
		}
	}
}

impl<V: ParityVisitor, M: Compiler<Input = ParityMatrix, Output = Circuit<CNot>>> Compiler
	for TPar<V, M>
{
	type Input = Circuit<CNotRzXYH>;
	type Output = Circuit<CNotRzXYH>;

	fn compile(&self, input: Self::Input) -> Self::Output {
		let n = input
			.iter()
			.map(|gate| gate.n_required_qubits())
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
		let mut output = Circuit::new();
		let mut state = State::new(n);

		for i in 0..n {
			if let Some(angle) = S.remove(&Parity::for_qubit(i)) {
				output.push(Rz { angle, target: i });
			}

			if let Some(angle) = S.remove(&Parity::for_qubit(i).not()) {
				state.apply_x(i);
				output.push(X { target: i });
				output.push(Rz { angle, target: i });
			}
		}

		for h in H {
			let mut nots_to_apply = Vec::new();
			for (i, parity) in state.parities().iter().enumerate() {
				if let Some(angle) = S.remove(parity) {
					output.push(Rz { angle, target: i });
				}

				if let Some(angle) = S.remove(&parity.clone().not()) {
					nots_to_apply.push(i);
					output.push(X { target: i });
					output.push(Rz { angle, target: i });
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
				state.apply_cnot(cnot.control(), cnot.target());
				output.push(cnot);

				let parity = state.get_cloned(cnot.target());
				if let Some(angle) = S.remove(&parity) {
					required.remove(&parity);
					output.push(Rz {
						angle,
						target: cnot.target(),
					});
				}

				let not_parity = state.get_cloned(cnot.target()).not();
				if let Some(angle) = S.remove(&not_parity) {
					state.apply_x(cnot.target());
					required.remove(&not_parity);
					output.push(X {
						target: cnot.target(),
					});
					output.push(Rz {
						angle,
						target: cnot.target(),
					});
				}
			}
			assert!(
				required.is_empty(),
				"Visitor does not visit all required parities"
			);

			// Map to h.input
			let state_span = state.create_span();
			let parity_rows: Vec<_> = h
				.input
				.parities()
				.iter()
				.map(|parity| parity.using_span(&state_span).unwrap())
				.collect();
			let parity_matrix = ParityMatrix::standard_from_rows(parity_rows);
			for cnot in self.parity_solver.compile(parity_matrix).into_iter().rev() {
				state.apply_cnot(cnot.control(), cnot.target());
				output.push(cnot);

				let parity = state.get_cloned(cnot.target());
				if let Some(angle) = S.remove(&parity) {
					output.push(Rz {
						angle,
						target: cnot.target(),
					});
				}

				let not_parity = state.get_cloned(cnot.target()).not();
				if let Some(angle) = S.remove(&not_parity) {
					state.apply_x(cnot.target());
					output.push(X {
						target: cnot.target(),
					});
					output.push(Rz {
						angle,
						target: cnot.target(),
					});
				}
			}

			for (i, (current, goal)) in state
				.mut_parities()
				.iter_mut()
				.zip(h.input.parities().iter())
				.enumerate()
			{
				if current.bit_flip != goal.bit_flip {
					current.bit_flip = goal.bit_flip;
					output.push(X { target: i });
				}

				assert_eq!(current, goal, "ParityMatrix solving failed");
			}

			output.push(H { target: h.target });
			state = h.output;
		}

		// resolve rest. mostly copy pasted from above
		{
			let mut nots_to_apply = Vec::new();
			for (i, parity) in state.parities().iter().enumerate() {
				if let Some(angle) = S.remove(parity) {
					output.push(Rz { angle, target: i });
				}

				if let Some(angle) = S.remove(&parity.clone().not()) {
					nots_to_apply.push(i);
					output.push(X { target: i });
					output.push(Rz { angle, target: i });
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
				state.apply_cnot(cnot.control(), cnot.target());
				output.push(cnot);

				let parity = state.get_cloned(cnot.target());
				if let Some(angle) = S.remove(&parity) {
					output.push(Rz {
						angle,
						target: cnot.target(),
					});
				}

				let not_parity = parity.not();
				if let Some(angle) = S.remove(&not_parity) {
					state.apply_x(cnot.target());
					output.push(X {
						target: cnot.target(),
					});
					output.push(Rz {
						angle,
						target: cnot.target(),
					});
				}
			}
			assert!(S.is_empty(), "Visitor does not visit all required parities");

			let state_span = state.create_span();
			let parity_rows: Vec<_> = Q
				.parities()
				.iter()
				.map(|parity| parity.using_span(&state_span).unwrap())
				.collect();
			let parity_matrix = ParityMatrix::standard_from_rows(parity_rows);
			for cnot in self.parity_solver.compile(parity_matrix).into_iter().rev() {
				state.apply_cnot(cnot.control(), cnot.target());
				output.push(cnot);
			}

			let mut indicies = 0..(state.parities().len().max(Q.parities().len()));
			let mut parities = state.mut_parities().iter_mut();
			let mut goals = Q.parities().iter();
			loop {
				match (indicies.next(), parities.next(), goals.next()) {
					(Some(i), Some(current), Some(goal)) => {
						if current.bit_flip != goal.bit_flip {
							current.bit_flip = goal.bit_flip;
							output.push(X { target: i });
						}

						assert_eq!(*current, *goal, "ParityMatrix solving failed");
					}
					(Some(i), Some(current), None) => {
						if current.bit_flip {
							current.bit_flip = false;
							output.push(X { target: i });
						}

						assert_eq!(
							*current,
							Parity::for_qubit(i),
							"ParityMatrix solving failed"
						);
					}
					(Some(i), None, Some(goal)) => {
						if goal.bit_flip {
							output.push(X { target: i });
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

	use circuit::{
		Circuit,
		gates::{CNot, H, Rz},
	};
	use simulator::Statevector;
	use test_core::Compiler;

	use crate::{
		PatelMarkovHayes,
		gra_star_synth::GrayStar,
		t_par::gateset::{CNotRzXYH, QuarterPi},
	};

	use super::{TPar, squirrel::Squirrel};

	#[test]
	fn exploration() {
		let gates: Vec<CNotRzXYH> = vec![
			// --- First Block (Qubits 0, 1, 2) ---
			CNotRzXYH::H(H { target: 2 }),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(1),
				target: 0,
			}),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(1),
				target: 1,
			}),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(1),
				target: 2,
			}),
			CNotRzXYH::CNot(CNot::new(1, 0).unwrap()),
			CNotRzXYH::CNot(CNot::new(2, 1).unwrap()),
			CNotRzXYH::CNot(CNot::new(0, 2).unwrap()),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(7),
				target: 1,
			}),
			CNotRzXYH::CNot(CNot::new(0, 1).unwrap()),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(7),
				target: 0,
			}),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(7),
				target: 1,
			}),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(1),
				target: 2,
			}),
			CNotRzXYH::CNot(CNot::new(2, 1).unwrap()),
			CNotRzXYH::CNot(CNot::new(0, 2).unwrap()),
			CNotRzXYH::CNot(CNot::new(1, 0).unwrap()),
			CNotRzXYH::H(H { target: 2 }),
			// --- Second Block (Qubits 1, 2, 3) ---
			CNotRzXYH::H(H { target: 3 }),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(1),
				target: 1,
			}),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(1),
				target: 2,
			}),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(1),
				target: 3,
			}),
			CNotRzXYH::CNot(CNot::new(2, 1).unwrap()),
			CNotRzXYH::CNot(CNot::new(3, 2).unwrap()),
			CNotRzXYH::CNot(CNot::new(1, 3).unwrap()),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(7),
				target: 2,
			}),
			CNotRzXYH::CNot(CNot::new(1, 2).unwrap()),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(7),
				target: 1,
			}),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(7),
				target: 2,
			}),
			CNotRzXYH::Rz(Rz {
				angle: QuarterPi(1),
				target: 3,
			}),
			CNotRzXYH::CNot(CNot::new(3, 2).unwrap()),
			CNotRzXYH::CNot(CNot::new(1, 3).unwrap()),
			CNotRzXYH::CNot(CNot::new(2, 1).unwrap()),
			CNotRzXYH::H(H { target: 3 }),
		];
		let circuit = Circuit { gates };
		let tpar = TPar::new(GrayStar, PatelMarkovHayes::new(NonZeroU32::new(2).unwrap()));
		let res = tpar.compile(circuit);
		dbg!(res);
	}

	#[test]
	fn random_tpar_test() {
		const QUBITS: usize = 4;
		const GATES: usize = 5;

		let mut rng = rand::rng();
		let tpar = TPar::new(GrayStar, PatelMarkovHayes::new(NonZeroU32::new(2).unwrap()));

		let circuit: Circuit<CNotRzXYH> = Circuit::random(GATES, QUBITS, &mut rng);
		let compiled = tpar.compile(circuit.clone());

		let mut original: Statevector<Squirrel> = Statevector::new(QUBITS);
		for gate in circuit.iter() {
			original.apply(gate);
		}

		let mut new: Statevector<Squirrel> = Statevector::new(QUBITS);
		for gate in compiled.iter() {
			new.apply(gate);
		}

		assert_eq!(original, new);
	}
}
