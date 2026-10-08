use std::ops::Neg;

use crate::clifford::Clifford;

use super::PauliString;

/// An Pauli exponential $e^{i\theta P}$ where $\theta$ is a 'T' and $P$ a [PauliString].
#[derive(Debug, Clone, Default)]
pub struct PauliExp<T> {
	pub string: PauliString,
	pub angle: T,
}

impl<T> PauliExp<T> {
	/// The amount of non identity letters in the [PauliString]
	#[allow(clippy::len_without_is_empty)]
	pub fn len(&self) -> usize {
		self.string.len()
	}
}

impl<T> PauliExp<T>
where
	for<'a> &'a T: Neg<Output = T>,
{
	/// Pushes a [Clifford] trough `self`.
	///
	/// We use the property that for a Pauli exponential $e^A$ and Clifford $U$
	/// it holds that:
	///
	/// $$Ue^A=Ue^AU^\dagger U=e^{UAU^\dagger}U.$$
	///
	/// ([Picturing Quantum Software Chapter 7](https://github.com/zxcalc/book)
	/// contains an explanation)
	pub fn push_cliffor<C: Clifford>(&mut self, clifford: &C) {
		let (sign, string) = clifford.conjugate(&self.string);
		self.string = string;
		if sign {
			self.angle = -&self.angle;
		}
	}
}
