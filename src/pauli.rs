//! This module contains tools for working with Pauli exponentials.

use std::{
	collections::{BTreeMap, btree_map::Entry},
	ops::Neg,
};

/// An enum for telling of sign changes when multiplying Pauli strings.
pub enum Sign {
	Negative,
	Positive,
}

/// An enum with variants that correspond to the $X$, $Y$, and $Z$ Pauli matrices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauliMatrix {
	X,
	Y,
	Z,
}

impl PauliMatrix {
	pub fn commutes_with(&self, other: &Self) -> bool {
		self == other
	}

	pub fn anticommutes_with(&self, other: &Self) -> bool {
		self != other
	}
}

/// An collection of [Pauli]s with qubit numbers attached to them.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PauliString {
	pub letters: BTreeMap<u32, PauliMatrix>,
}

impl PauliString {
	pub fn commutes_with(&self, other: &Self) -> bool {
		let mut commutes = true;
		for (qubit, m1) in self.letters.iter() {
			if let Some(m2) = other.letters.get(qubit) {
				if m1.anticommutes_with(m2) {
					commutes = !commutes;
				}
			}
		}

		commutes
	}

	pub fn anticommutes_with(&self, other: &Self) -> bool {
		!self.commutes_with(other)
	}

	pub fn len(&self) -> usize {
		self.letters.len()
	}
}

/// The angle used by [PauliExp].
///
/// TODO: Change later into Free and Clifford?
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PauliAngle {
	Free(f64),
	PiOver4,
	NeqPiOver4,
}

impl Neg for PauliAngle {
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			PauliAngle::Free(x) => PauliAngle::Free(-x),
			PauliAngle::PiOver4 => PauliAngle::NeqPiOver4,
			PauliAngle::NeqPiOver4 => PauliAngle::PiOver4,
		}
	}
}

/// An Pauli exponential $e^{i\theta P}$ where $\theta$ is a [PauliAngle] and $P$ a [PauliString].
#[derive(Clone, Debug, PartialEq)]
pub struct PauliExp {
	pub string: PauliString,
	pub angle: PauliAngle,
}

impl PauliExp {
	pub fn len(&self) -> usize {
		self.string.len()
	}

	/// Pushes $e^{-i\frac{\pi}{4}O}$ trough `self`.
	///
	/// More precisely that `self` is converted from $e^{i\theta P}$ to $e^{i\theta H}$ by using the
	/// following rule:
	///
	/// $$e^{-i\frac{\pi}{4}O}e^{i\theta P}=e^{i\theta H}e^{-i\frac{\pi}{4}O}.$$
	///
	/// In practice this means that if the PauliString $P$ of `self` commuters with $O$ nothing
	/// happens and if it anticommutes with $O$ `self` is converted to $e^{i\theta PO}$.
	///
	/// # Proof:
	/// We firstly use the property that for a Pauli exponential $e^A$ and Clifford $U$ it holds
	/// that (See [Picturing Quantum Software Chapter 7](https://github.com/zxcalc/book) for
	/// explanation):
	///
	/// $$Ue^A=Ue^AU^\dagger U=e^{UAU^\dagger}U.$$
	///
	/// Because $e^{-i\frac{\pi}{4}O}$ is Clifford with its conjugate transpose being
	/// $e^{i\frac{\pi}{4}O}$ we can use the previous property to see that that `self` has to be
	/// converted to
	///
	/// $$\exp({e^{-i\frac{\pi}{4}O}(i\theta P)e^{i\frac{\pi}{4}O}})$$.
	///
	/// Because $O$ is an Pauli string and therefore $O^2=I$ we have (See [Picturing Quantum
	/// Software Chapter 7](https://github.com/zxcalc/book) for an explanation)
	///
	/// $$
	/// e^{-i\frac{\pi}{4}O}=\text{cos}(-\frac{\pi}{4})I+i\text{sin}(-\frac{\pi}{4})O
	/// =\frac{1}{\sqrt2}I-i\frac{1}{\sqrt2}O=\frac{1}{\sqrt2}(I-iO)
	/// $$
	///
	/// and
	///
	/// $$
	/// e^{i\frac{\pi}{4}O}=\text{cos}(\frac{\pi}{4})I+i\text{sin}(\frac{\pi}{4})O
	/// =\frac{1}{\sqrt2}I+i\frac{1}{\sqrt2}O=\frac{1}{\sqrt2}(I+iO)
	/// $$.
	///
	/// With this we can write the new Pauli exponential as
	///
	/// $$
	/// \text{exp}(\frac{1}{\sqrt2}(I-iO)(i\theta P)\frac{1}{\sqrt2}(I+iO))
	/// =\text{exp}(i\frac{\theta}{2}(P+iPO-iOP+OPO))
	/// $$
	///
	/// Next we will go individually trough the cases where $O$ and $P$ commute and anticommute.
	/// Because $O$ and $P$ are Pauli strings they have to either commute or anticommute which means
	/// that we cover all possible cases.
	///
	/// **When $O$ and $P$ commute** we have $OP=PO$ allowing us to simplify as follows:  
	///
	/// $$
	/// \text{exp}(i\frac{\theta}{2}(P+iPO-iOP+OPO))
	/// =\text{exp}(i\frac{\theta}{2}(P+iPO-iPO+O^2P)
	/// =\text{exp}(i\frac{\theta}{2}(P+IP)
	/// =\text{exp}(i\theta P).
	/// $$
	///
	/// **When $O$ and $P$ anticommute** we have $OP=-PO$ and we can simplify as follows:
	///
	/// $$
	/// \text{exp}(i\frac{\theta}{2}(P+iPO-iOP+OPO))
	/// =\text{exp}(i\frac{\theta}{2}(P+iPO+iOP-O^2P)
	/// =\text{exp}(i\theta PO).
	/// $$
	pub fn push_neq_pi_over_4(&mut self, #[allow(non_snake_case)] O: &PauliString) {
		if self.string.commutes_with(O) {
			return;
		}

		// Counts the amount of i:s as in i^x
		let mut imagination = 1;
		for (qubit, m2) in O.letters.iter() {
			match self.string.letters.entry(*qubit) {
				Entry::Vacant(entry) => {
					entry.insert(*m2);
				}
				Entry::Occupied(mut entry) => {
					use PauliMatrix::*;
					match (entry.get(), m2) {
						(X, X) | (Y, Y) | (Z, Z) => {
							entry.remove();
						}
						(X, Y) => {
							entry.insert(Z);
							imagination += 1;
						}
						(Y, X) => {
							entry.insert(Z);
							imagination += 3;
						}
						(Z, X) => {
							entry.insert(Y);
							imagination += 1;
						}
						(X, Z) => {
							entry.insert(Y);
							imagination += 3;
						}
						(Y, Z) => {
							entry.insert(X);
							imagination += 1;
						}
						(Z, Y) => {
							entry.insert(X);
							imagination += 3;
						}
					}
				}
			}
		}

		match imagination % 4 {
			0 => {}
			2 => {
				self.angle = -self.angle;
			}
			_ => unreachable!(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn red_pi_through_zz() {
		let mut zz = PauliExp {
			angle: PauliAngle::Free(-2.0),
			string: PauliString {
				letters: BTreeMap::from([(0, PauliMatrix::Z), (1, PauliMatrix::Z)]),
			},
		};

		let red = PauliString {
			letters: BTreeMap::from([(0, PauliMatrix::X)]),
		};

		zz.push_neq_pi_over_4(&red);
		zz.push_neq_pi_over_4(&red);

		let zz_string = PauliString {
			letters: BTreeMap::from([(0, PauliMatrix::Z), (1, PauliMatrix::Z)]),
		};
		assert_eq!(PauliAngle::Free(2.0), zz.angle);
		assert_eq!(zz_string, zz.string);
	}
}
