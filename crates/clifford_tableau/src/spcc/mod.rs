pub mod gateset;
use core::{
	Compiler,
	connectivity::{Connectivity, TwoQubitEdge},
};

use circuit::Circuit;

use crate::CliffordTableau;

use self::gateset::CNotHSV;

pub struct SPCC;

impl Compiler<CliffordTableau, Circuit<CNotHSV>, Connectivity<TwoQubitEdge>> for SPCC {
	fn compile(
		&self,
		input: CliffordTableau,
		device: &Connectivity<TwoQubitEdge>,
	) -> Circuit<CNotHSV> {
		todo!()
	}
}

#[cfg(test)]
mod tests {
	use circuit::Circuit;
	use rand::SeedableRng;
	use rand_chacha::ChaCha8Rng;

	use crate::CliffordTableau;

	use super::{SPCC, gateset::CNotHSV};
	use core::{Compiler, connectivity::TwoQubitEdge};

	#[test]
	#[ignore = "not yet implemented"]
	fn random_spcc_test() {
		const ROUNDS: usize = 100;
		const QUBITS: usize = 30;
		const GATES: usize = QUBITS * 100;
		let compiler = SPCC;
		let connectivity = TwoQubitEdge::square_lattice(QUBITS);

		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for round in 0..ROUNDS {
			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			let mut tableau = CliffordTableau::id();
			for gate in input.iter() {
				tableau.merge_clifford(gate);
			}
			let result = compiler.compile(tableau.clone(), &connectivity);
			let mut result_tableau = CliffordTableau::id();
			for gate in result.iter() {
				if let CNotHSV::CNot(cnot) = gate
					&& !connectivity
						.neighbors(*cnot.target())
						.contains(cnot.control())
				{
					panic!(
						"Got CNot({}, {}) that is not supported by connectivity",
						cnot.control(),
						cnot.target()
					);
				}
				result_tableau.merge_clifford(gate);
			}

			assert_eq!(result_tableau, tableau);
			println!(
				"Round {round} went from {} to {} CNOT gates",
				input.fileter_len(|g| { matches!(g, CNotHSV::CNot(_)) }),
				result.fileter_len(|g| { matches!(g, CNotHSV::CNot(_)) })
			);
		}
	}
}
