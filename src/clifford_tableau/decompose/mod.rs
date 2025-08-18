mod delicate_solver;
mod simple_solver;

use crate::{
	clifford_tableau::CliffordTableau,
	connectivity::Connectivity,
	misc::NonZeroEvenUsize,
	pauli::{CliffordPauliAngle, PauliExp, PauliLetter, PauliString},
};
use delicate_solver::{delicate_solver, fastest_delicate};
use simple_solver::{fastest, simple_solver};

#[derive(Debug, PartialEq, Eq)]
enum QubitProtection {
	X,
	Z,
	None,
}

impl<const N: usize> CliffordTableau<N> {
	/// # Decompose
	///
	/// Decomposes the tableau into clifford gates.
	pub fn decompose(
		self,
		gate_size: NonZeroEvenUsize,
		connectivity: Option<&Connectivity>,
	) -> Vec<PauliExp<N, CliffordPauliAngle>> {
		match connectivity {
			Some(connectivity) => todo!(),
			_ => self.decompose_full_connectivity(gate_size),
		}
	}

	fn decompose_full_connectivity(
		mut self,
		gate_size: NonZeroEvenUsize,
	) -> Vec<PauliExp<N, CliffordPauliAngle>> {
		let mut decomposition: Vec<PauliExp<N, CliffordPauliAngle>> = Vec::new();
		let mut dirty_qubits: Vec<usize> = (0..N).collect();

		while dirty_qubits.len() >= gate_size.as_value() {
			let (qubit, letter) = fastest(&self, &dirty_qubits, gate_size).unwrap();

			match letter {
				PauliLetter::X => {
					let x_moves = simple_solver(
						self.x.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::X,
						&dirty_qubits,
						QubitProtection::None,
					);
					for string in x_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
					let z_moves = simple_solver(
						self.z.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::Z,
						&dirty_qubits,
						QubitProtection::X,
					);
					for string in z_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
				}
				PauliLetter::Z => {
					let z_moves = simple_solver(
						self.z.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::Z,
						&dirty_qubits,
						QubitProtection::None,
					);
					for string in z_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
					let x_moves = simple_solver(
						self.x.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::X,
						&dirty_qubits,
						QubitProtection::Z,
					);
					for string in x_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
				}
				_ => unreachable!(),
			}

			// Now the qubit is not dirty anymore
			dirty_qubits.retain(|q| *q != qubit);
		}

		// then for remaining use delicate solver
		while !dirty_qubits.is_empty() {
			let (qubit, letter) = fastest_delicate(&self, &dirty_qubits).unwrap();

			match letter {
				PauliLetter::X => {
					let x_moves = delicate_solver(
						self.x.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::X,
					);
					for string in x_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
					let z_moves = delicate_solver(
						self.z.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::Z,
					);
					for string in z_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
				}
				PauliLetter::Z => {
					let z_moves = delicate_solver(
						self.z.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::Z,
					);
					for string in z_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
					let x_moves = delicate_solver(
						self.x.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::X,
					);
					for string in x_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
				}
				_ => unreachable!(),
			}

			// Now the qubit is not dirty anymore
			dirty_qubits.retain(|q| *q != qubit);
		}

		for (i, (x, z)) in self
			.x_signs
			.clone()
			.into_iter()
			.zip(self.z_signs.clone().into_iter())
			.enumerate()
		{
			let string = match (x, z) {
				(true, true) => PauliString::y(i),
				(true, false) => PauliString::z(i),
				(false, true) => PauliString::x(i),
				_ => {
					continue;
				}
			};

			self.merge_pi_over_4_pauli(true, &string);
			self.merge_pi_over_4_pauli(true, &string);
			decomposition.push(PauliExp {
				string: string.clone(),
				angle: CliffordPauliAngle::PiOver4,
			});
			decomposition.push(PauliExp {
				string,
				angle: CliffordPauliAngle::PiOver4,
			});
		}

		assert_eq!(self, CliffordTableau::id());

		decomposition.into_iter().rev().collect()
	}
}
