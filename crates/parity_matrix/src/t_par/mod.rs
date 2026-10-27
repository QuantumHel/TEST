mod gateset;
mod parity;
mod squirrel;
mod state;
mod triplet;

use self::{state::State, triplet::Triplet};
use crate::ParityMatrix;
use bits::Bits;
use circuit::{
	Circuit,
	gates::{CNot, H, Rz, X},
};
use core::{
	Compiler,
	connectivity::{Connectivity, TwoQubitEdge},
	qubit_map::QubitMap,
};
use gateset::CNotRzXYH;
use parity::Parity;
use std::collections::BTreeSet;

pub struct HadamardTransform {
	target: usize,
	input: State,
	output: State,
}

/// Generic (kinda) implementation of TPar in https://arxiv.org/pdf/1303.2042
#[derive(Default)]
pub struct TPar<V, M> {
	visitor: V,
	parity_solver: M,
}

impl<V, M> TPar<V, M> {
	pub fn new(visitor: V, parity_solver: M) -> Self {
		Self {
			visitor,
			parity_solver,
		}
	}
}

struct MapWrapping<'a, M> {
	parity_solver: &'a M,
}

impl<'a, D, M: Compiler<ParityMatrix, Circuit<CNot>, D>>
	Compiler<ParityMatrix, (Circuit<CNot>, QubitMap), D> for MapWrapping<'a, M>
{
	fn compile(&self, input: ParityMatrix, device: &D) -> (Circuit<CNot>, QubitMap) {
		let circuit: Circuit<CNot> = self.parity_solver.compile(input, device);
		(circuit, QubitMap::default())
	}
}

fn internal_compile<
	D,
	V: ParityVisitor<D>,
	M: Compiler<ParityMatrix, (Circuit<CNot>, QubitMap), D>,
>(
	visitor: &V,
	parity_solver: &M,
	n: usize,
	input: Circuit<CNotRzXYH>,
	device: &D,
) -> (Circuit<CNotRzXYH>, QubitMap) {
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

		let cnots = visitor.visit(required_bits, optional_bits, device);
		let mut required: BTreeSet<_> = required.into_iter().map(|(a, _)| a.clone()).collect();
		for cnot in cnots {
			state.apply_cnot(*cnot.control(), *cnot.target());
			output.push(cnot);

			let parity = state.get_cloned(*cnot.target());
			if let Some(angle) = S.remove(&parity) {
				required.remove(&parity);
				output.push(Rz {
					angle,
					target: *cnot.target(),
				});
			}

			let not_parity = state.get_cloned(*cnot.target()).not();
			if let Some(angle) = S.remove(&not_parity) {
				state.apply_x(*cnot.target());
				required.remove(&not_parity);
				output.push(X {
					target: *cnot.target(),
				});
				output.push(Rz {
					angle,
					target: *cnot.target(),
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
		let (cnots, qubit_map) = parity_solver.compile(parity_matrix, device);
		// we need to have a total qubit map so we know where we apply hadamards
		for cnot in cnots.into_iter() {
			state.apply_cnot(*cnot.control(), *cnot.target());
			output.push(cnot);

			let parity = state.get_cloned(*cnot.target());
			if let Some(angle) = S.remove(&parity) {
				output.push(Rz {
					angle,
					target: *cnot.target(),
				});
			}

			let not_parity = state.get_cloned(*cnot.target()).not();
			if let Some(angle) = S.remove(&not_parity) {
				state.apply_x(*cnot.target());
				output.push(X {
					target: *cnot.target(),
				});
				output.push(Rz {
					angle,
					target: *cnot.target(),
				});
			}
		}

		let h_parities = h.input.parities();
		for (i, current) in state.mut_parities().iter_mut().enumerate() {
			let goal_qubit = qubit_map.map(i);
			let goal = h_parities.get(goal_qubit).unwrap();
			if current.bit_flip != goal.bit_flip {
				current.bit_flip = goal.bit_flip;
				output.push(X { target: i });
			}

			assert_eq!(current, goal, "ParityMatrix solving failed");
		}

		output.push(H {
			target: qubit_map.source(h.target),
		});
		state = h.output.create_mapped(&qubit_map.as_reversed());
	}

	// resolve rest. mostly copy pasted from above
	let final_map = {
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

		let cnots = visitor.visit(required_bits, Vec::new(), device);

		for cnot in cnots {
			state.apply_cnot(*cnot.control(), *cnot.target());
			output.push(cnot);

			let parity = state.get_cloned(*cnot.target());
			if let Some(angle) = S.remove(&parity) {
				output.push(Rz {
					angle,
					target: *cnot.target(),
				});
			}

			let not_parity = parity.not();
			if let Some(angle) = S.remove(&not_parity) {
				state.apply_x(*cnot.target());
				output.push(X {
					target: *cnot.target(),
				});
				output.push(Rz {
					angle,
					target: *cnot.target(),
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
		// we get (cnots, mapping)
		// can we just return this mapping?
		let (cnots, final_map) = parity_solver.compile(parity_matrix, device);
		for cnot in cnots.into_iter() {
			state.apply_cnot(*cnot.control(), *cnot.target());
			output.push(cnot);
		}

		let mut indicies = 0..(state.parities().len().max(Q.parities().len()));
		let mut parities = state.mut_parities().iter_mut();
		let goal = Q.create_mapped(&final_map.clone().as_reversed());
		let mut goals = goal.parities().iter();

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

					let goal_qubit = final_map.map(i);

					assert_eq!(
						*current,
						Parity::for_qubit(goal_qubit),
						"ParityMatrix solving failed"
					);
				}
				(Some(i), None, Some(goal)) => {
					let goal_qubit = final_map.map(i);

					if goal.bit_flip {
						output.push(X { target: i });
						assert_eq!(
							*goal,
							Parity::for_qubit(goal_qubit).not(),
							"ParityMatrix solving failed"
						);
					} else {
						assert_eq!(
							*goal,
							Parity::for_qubit(goal_qubit),
							"ParityMatrix solving failed"
						);
					}
				}
				(None, _, _) => {
					break;
				}
				_ => unreachable!(),
			}
		}

		final_map
	};

	(output, final_map)
}

impl<V: ParityVisitor<()>, M: Compiler<ParityMatrix, Circuit<CNot>>>
	Compiler<Circuit<CNotRzXYH>, Circuit<CNotRzXYH>> for TPar<V, M>
{
	fn compile(&self, input: Circuit<CNotRzXYH>, device: &()) -> Circuit<CNotRzXYH> {
		let n = input
			.iter()
			.map(|gate| gate.n_required_qubits())
			.max()
			.unwrap_or_default();

		let (circuit, _) = internal_compile(
			&self.visitor,
			&MapWrapping {
				parity_solver: &self.parity_solver,
			},
			n,
			input,
			device,
		);
		circuit
	}
}

impl<V: ParityVisitor<()>, M: Compiler<ParityMatrix, (Circuit<CNot>, QubitMap)>>
	Compiler<Circuit<CNotRzXYH>, (Circuit<CNotRzXYH>, QubitMap)> for TPar<V, M>
{
	fn compile(&self, input: Circuit<CNotRzXYH>, device: &()) -> (Circuit<CNotRzXYH>, QubitMap) {
		let n = input
			.iter()
			.map(|gate| gate.n_required_qubits())
			.max()
			.unwrap_or_default();

		internal_compile(&self.visitor, &self.parity_solver, n, input, device)
	}
}

impl<
	V: ParityVisitor<Connectivity<TwoQubitEdge>>,
	M: Compiler<ParityMatrix, Circuit<CNot>, Connectivity<TwoQubitEdge>>,
> Compiler<Circuit<CNotRzXYH>, Circuit<CNotRzXYH>, Connectivity<TwoQubitEdge>> for TPar<V, M>
{
	fn compile(
		&self,
		input: Circuit<CNotRzXYH>,
		device: &Connectivity<TwoQubitEdge>,
	) -> Circuit<CNotRzXYH> {
		let n = device.nodes().len();

		let (circuit, _) = internal_compile(
			&self.visitor,
			&MapWrapping {
				parity_solver: &self.parity_solver,
			},
			n,
			input,
			device,
		);
		circuit
	}
}

impl<
	V: ParityVisitor<Connectivity<TwoQubitEdge>>,
	M: Compiler<ParityMatrix, (Circuit<CNot>, QubitMap), Connectivity<TwoQubitEdge>>,
> Compiler<Circuit<CNotRzXYH>, (Circuit<CNotRzXYH>, QubitMap), Connectivity<TwoQubitEdge>>
	for TPar<V, M>
{
	fn compile(
		&self,
		input: Circuit<CNotRzXYH>,
		device: &Connectivity<TwoQubitEdge>,
	) -> (Circuit<CNotRzXYH>, QubitMap) {
		let n = device.nodes().len();

		internal_compile(&self.visitor, &self.parity_solver, n, input, device)
	}
}

pub trait ParityVisitor<Device> {
	fn visit(&self, required: Vec<Bits>, optional: Vec<Bits>, device: &Device) -> Vec<CNot>;
}

#[cfg(test)]
mod tests {
	use circuit::{
		Circuit,
		gates::{CNot, H, Rz},
	};
	use core::{
		Compiler,
		connectivity::{Connectivity, TwoQubitEdge},
	};
	use rand::prelude::*;
	use rand_chacha::ChaCha8Rng;
	use simulator::{Simulatable, Statevector};
	use std::num::NonZeroU32;

	use crate::{
		choose::{First, OriginalPermRowColChoice},
		decompose::{PatelMarkovHayes, PermRowCol, RowCol},
		gray_star_synth::{GrayStarSynth, GrayStarTerminationCriteria},
		t_par::gateset::{CNotRzXYH, QuarterPi},
	};

	use super::{TPar, squirrel::Squirrel};

	#[test]
	#[ignore = "This is for manual testing"]
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
		let tpar = TPar::new(
			GrayStarSynth {
				max_queue_size: None,
				path_termination_criteria: GrayStarTerminationCriteria::QubitRemoved(1),
			},
			PatelMarkovHayes::new(NonZeroU32::new(2).unwrap()),
		);
		let res = tpar.compile(circuit, &());
		dbg!(res);
	}

	fn random_tpar_test(qubits: usize, gates: usize, rounds: usize) {
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for round in 1..=rounds {
			let tpar = TPar::new(
				GrayStarSynth {
					max_queue_size: None,
					path_termination_criteria: GrayStarTerminationCriteria::QubitRemoved(1),
				},
				PatelMarkovHayes::new(NonZeroU32::new(2).unwrap()),
			);

			let circuit: Circuit<CNotRzXYH> = Circuit::random(gates, qubits, &mut rng);
			let compiled = tpar.compile(circuit.clone(), &());

			let mut original: Statevector<Squirrel> = Statevector::new(qubits);
			for gate in circuit.iter() {
				original.apply(gate);
			}

			let mut new: Statevector<Squirrel> = Statevector::new(qubits);
			for gate in compiled.iter() {
				new.apply(gate);
			}

			assert_eq!(original, new);
			println!(
				"Round {round} went from {} to {} CNOT gates",
				circuit.fileter_len(|g| { matches!(g, CNotRzXYH::CNot(_)) }),
				compiled.fileter_len(|g| { matches!(g, CNotRzXYH::CNot(_)) })
			);
		}
	}

	#[test]
	fn short_random_tpar_test() {
		const QUBITS: usize = 10;
		const GATES: usize = 100;
		const ROUNDS: usize = 10;

		random_tpar_test(QUBITS, GATES, ROUNDS);
	}

	#[test]
	#[ignore = "takes too long for normal test runs"]
	fn long_random_tpar_test() {
		const QUBITS: usize = 15;
		const GATES: usize = 100;
		const ROUNDS: usize = 100;

		random_tpar_test(QUBITS, GATES, ROUNDS);
	}

	fn random_connectivity_tpar_test(qubits: usize, gates: usize, rounds: usize) {
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for round in 1..=rounds {
			let device: Connectivity<TwoQubitEdge> = TwoQubitEdge::square_lattice(qubits);
			let tpar = TPar::new(
				GrayStarSynth {
					max_queue_size: None,
					path_termination_criteria: GrayStarTerminationCriteria::QubitRemoved(1),
				},
				RowCol {
					choose_qubit: First::default(),
				},
			);

			let circuit: Circuit<CNotRzXYH> = Circuit::random(gates, qubits, &mut rng);
			let compiled = tpar.compile(circuit.clone(), &device);

			let mut max_qubit = 0;
			for gate in compiled.iter() {
				max_qubit = max_qubit.max(
					gate.controls()
						.iter()
						.max()
						.copied()
						.unwrap_or_default()
						.max(gate.target()),
				);
			}

			let mut original: Statevector<Squirrel> = Statevector::new(max_qubit + 1);
			for gate in circuit.iter() {
				original.apply(gate);
			}

			let mut new: Statevector<Squirrel> = Statevector::new(max_qubit + 1);
			for gate in compiled.iter() {
				if let CNotRzXYH::CNot(cnot) = gate
					&& !device.neighbors(*cnot.target()).contains(cnot.control())
				{
					panic!(
						"Got CNot({}, {}) that is not supported by connectivity",
						cnot.control(),
						cnot.target()
					);
				}

				new.apply(gate);
			}

			assert_eq!(original, new);
			println!(
				"Round {round} went from {} to {} CNOT gates",
				circuit.fileter_len(|g| { matches!(g, CNotRzXYH::CNot(_)) }),
				compiled.fileter_len(|g| { matches!(g, CNotRzXYH::CNot(_)) })
			);
		}
	}

	#[test]
	fn extra_qubit_usage_connectivity_tpar_test() {
		// this will (with the given seed) use 6 qubits sometimes
		const QUBITS: usize = 5;
		const GATES: usize = 100;
		const ROUNDS: usize = 100;

		random_connectivity_tpar_test(QUBITS, GATES, ROUNDS);
	}

	#[test]
	fn short_random_connectivity_tpar_test() {
		const QUBITS: usize = 10;
		const GATES: usize = 100;
		const ROUNDS: usize = 10;

		random_connectivity_tpar_test(QUBITS, GATES, ROUNDS);
	}

	#[test]
	#[ignore = "takes too long for normal test runs"]
	fn long_random_connectivity_tpar_test() {
		const QUBITS: usize = 15;
		const GATES: usize = 100;
		const ROUNDS: usize = 100;

		random_connectivity_tpar_test(QUBITS, GATES, ROUNDS);
	}

	#[test]
	fn tpar_perm_row_col_test() {
		let qubits = 10;
		let gates = 100;
		let rounds = 10;
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for round in 1..=rounds {
			let device: Connectivity<TwoQubitEdge> = TwoQubitEdge::square_lattice(qubits);
			let tpar = TPar::new(
				GrayStarSynth {
					max_queue_size: None,
					path_termination_criteria: GrayStarTerminationCriteria::QubitRemoved(1),
				},
				PermRowCol {
					choose_row: OriginalPermRowColChoice,
					choose_column: OriginalPermRowColChoice,
				},
			);

			let circuit: Circuit<CNotRzXYH> = Circuit::random(gates, qubits, &mut rng);
			let (compiled_circuit, qubit_map) = tpar.compile(circuit.clone(), &device);

			let mut max_qubit = 0;
			for gate in compiled_circuit.iter() {
				max_qubit = max_qubit.max(
					gate.controls()
						.iter()
						.max()
						.copied()
						.unwrap_or_default()
						.max(gate.target()),
				);
			}

			let mut original: Statevector<Squirrel> = Statevector::new(max_qubit + 1);
			for gate in circuit.iter() {
				original.apply(gate);
			}

			let mut new: Statevector<Squirrel> = Statevector::new(max_qubit + 1);
			for gate in compiled_circuit.iter() {
				if let CNotRzXYH::CNot(cnot) = gate
					&& !device.neighbors(*cnot.target()).contains(cnot.control())
				{
					panic!(
						"Got CNot({}, {}) that is not supported by connectivity",
						cnot.control(),
						cnot.target()
					);
				}

				new.apply(gate);
			}
			new.apply_qubit_map(&qubit_map);

			assert_eq!(original, new);
			println!(
				"Round {round} went from {} to {} CNOT gates",
				circuit.fileter_len(|g| { matches!(g, CNotRzXYH::CNot(_)) }),
				compiled_circuit.fileter_len(|g| { matches!(g, CNotRzXYH::CNot(_)) })
			);
		}
	}
}
