use core::Compiler;

use circuit::{Circuit, RandomGate};
use pauli::Clifford;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::CliffordTableau;

/// Uses random inputs to test if the given [Compiler] for [CliffordTableau]
/// decomposition gives the correct results.
///
/// # Attention
/// - This test assumes that [Clifford] is implemented correctly for `G`!
/// - This test does not test if the output follows device constraints.
pub fn random_clifford_tableau_test<
	C: Compiler<CliffordTableau, Circuit<G>, D>,
	G: RandomGate + Clifford,
	D,
>(
	compiler: &C,
	device: &D,
	rounds: usize,
	n_qubits: usize,
	n_gates: usize,
) {
	let mut rng = ChaCha8Rng::seed_from_u64(67);
	for _ in 0..rounds {
		let circuit: Circuit<G> = Circuit::random(n_gates, n_qubits, &mut rng);
		let mut tableau = CliffordTableau::id();
		for gate in circuit {
			tableau.merge_clifford(&gate);
		}
		let result = compiler.compile(tableau.clone(), device);
		let mut result_tableau = CliffordTableau::id();
		for gate in result.iter() {
			result_tableau.merge_clifford(gate);
		}

		assert_eq!(result_tableau, tableau);
	}
}
