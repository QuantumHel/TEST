use std::{
	fs::{File, exists},
	io::Write,
};

use crate::pauli::{CliffordPauliAngle, FreePauliAngle};

use super::{PauliAngle, PauliString};

/// An Pauli exponential $e^{i\theta P}$ where $\theta$ is a [PauliAngle] and $P$ a [PauliString].
#[derive(Debug, Clone)]
pub struct PauliExp<const N: usize, T: PauliAngle> {
	pub string: PauliString<N>,
	pub angle: T,
}

impl<const N: usize, T: PauliAngle> PauliExp<N, T> {
	pub fn len(&self) -> usize {
		self.string.len()
	}

	pub fn is_empty(&self) -> bool {
		self.string.is_empty()
	}

	/// Pushes $e^{\pm i\frac{\pi}{4}O}$ trough `self`.
	///
	/// More precisely that `self` is converted from $e^{i\theta P}$ to $e^{i\theta H}$ by using the
	/// following rule:
	///
	/// $$e^{\pm i\frac{\pi}{4}O}e^{i\theta P}=e^{i\theta H}e^{\pm i\frac{\pi}{4}O}.$$
	///
	/// In practice this means that if the PauliString $P$ of `self` commuters with $O$ nothing
	/// happens and if it anticommutes with $O$ `self` is converted to $e^{\pm i\theta OP}$.
	///
	/// # Proof:
	/// We firstly use the property that for a Pauli exponential $e^A$ and Clifford $U$ it holds
	/// that (See [Picturing Quantum Software Chapter 7](https://github.com/zxcalc/book) for
	/// explanation):
	///
	/// $$Ue^A=Ue^AU^\dagger U=e^{UAU^\dagger}U.$$
	///
	/// Because $e^{\pm i\frac{\pi}{4}O}$ is Clifford, we can use this property to see that that
	/// `self` has to be converted to
	///
	/// $$
	/// \exp({e^{\pm i\frac{\pi}{4}O}(i\theta P)e^{\mp i\frac{\pi}{4}O}})
	/// =\exp(i\theta{e^{\pm i\frac{\pi}{4}O}Pe^{\mp i\frac{\pi}{4}O}}).
	/// $$
	///
	/// The remainder of the proof can be solved by using the proof from [PauliString::pi_over_4_sandwitch] on
	///
	/// $${e^{\pm i\frac{\pi}{4}O}Pe^{\mp i\frac{\pi}{4}O}}.$$
	///
	pub fn push_pi_over_4(&mut self, neg: bool, #[allow(non_snake_case)] O: &PauliString<N>) {
		if self.string.pi_over_4_sandwitch(neg, O) {
			self.angle = -self.angle;
		}
	}
}

pub fn as_exp_file<const N: usize, T: PauliAngle>(path: &str, paulis: &Vec<PauliExp<N, T>>) {
	if exists(path).unwrap() {
		panic!("Tried to overwrite a file");
	}

	let mut file = File::create(path).unwrap();
	for pauli in paulis {
		let angle = pauli.angle.multiple_of_pi();
		let string = pauli.string.as_string();
		writeln!(&mut file, "{angle};{string}").unwrap();
	}
}

impl<const N: usize> From<PauliExp<N, CliffordPauliAngle>> for PauliExp<N, FreePauliAngle> {
	fn from(value: PauliExp<N, CliffordPauliAngle>) -> Self {
		PauliExp::<N, FreePauliAngle> {
			angle: value.angle.into(),
			string: value.string,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::super::{FreePauliAngle, PauliLetter};
	use super::*;

	#[test]
	fn red_pi_through_zz() {
		let mut letters = PauliString::<2>::z(0);
		letters.set(1, PauliLetter::Z);
		let mut zz = PauliExp {
			angle: FreePauliAngle::MultipleOfPi(-2.0),
			string: letters,
		};

		let red = PauliString::x(0);

		zz.push_pi_over_4(true, &red);
		zz.push_pi_over_4(true, &red);

		let mut zz_string = PauliString::z(0);
		zz_string.set(1, PauliLetter::Z);

		assert_eq!(FreePauliAngle::MultipleOfPi(2.0), zz.angle);
		assert_eq!(zz_string, zz.string);
	}
}
