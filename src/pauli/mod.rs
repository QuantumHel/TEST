//! This module contains tools for working with Pauli exponentials.

mod pauli_angle;
mod pauli_exp;
mod pauli_string;

pub use pauli_angle::{CliffordPauliAngle, FreePauliAngle, PauliAngle};
pub use pauli_exp::PauliExp;
pub use pauli_string::PauliString;

/// An enum with variants that correspond to the $X$, $Y$, and $Z$ Pauli matrices and $I$.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauliLetter {
	I,
	X,
	Y,
	Z,
}

impl PauliLetter {
	pub fn commutes_with(&self, other: &Self) -> bool {
		(self == &PauliLetter::I) || (other == &PauliLetter::I) || (self == other)
	}

	pub fn anticommutes_with(&self, other: &Self) -> bool {
		!self.commutes_with(other)
	}

	/// This gives the next pauli matrix according to the (looping) order
	///
	/// X -> Y -> Z -> X ->
	///
	/// As a special case the next matrix after I is X.
	pub fn next(self) -> Self {
		match self {
			PauliLetter::I => PauliLetter::X,
			PauliLetter::X => PauliLetter::Y,
			PauliLetter::Y => PauliLetter::Z,
			PauliLetter::Z => PauliLetter::X,
		}
	}
}
