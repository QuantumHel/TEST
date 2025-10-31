use std::{
	fs::{File, exists},
	io::{self, BufRead, Write},
};

use crate::pauli::{CliffordPauliAngle, PauliAngle, PauliLetter, pauli_angle::Negate};

use super::PauliString;

/// An Pauli exponential $e^{i\theta P}$ where $\theta$ is a [PauliAngle] and $P$ a [PauliString].
#[derive(Debug, Clone)]
pub struct PauliExp<const N: usize, T: Negate> {
	pub string: PauliString<N>,
	pub angle: T,
}

impl<const N: usize, T: Negate> PauliExp<N, T> {
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
			self.angle.negate();
		}
	}
}

impl<const N: usize> PauliExp<N, PauliAngle> {
	pub fn write_exp_file(exps: &Vec<Self>, path: &str) {
		if exists(path).unwrap() {
			panic!("Tried to overwrite a file");
		}

		let mut file = File::create(path).unwrap();
		for pauli in exps {
			let angle = match &pauli.angle {
				PauliAngle::MultipleOfPi(v) => format!("{v}"),
				PauliAngle::Parameter { neg: false, name } => name.clone(),
				PauliAngle::Parameter { neg: true, name } => format!("-{name}"),
				PauliAngle::Clifford(CliffordPauliAngle::PiOver2) => String::from("0.5"),
				PauliAngle::Clifford(CliffordPauliAngle::PiOver4) => String::from("0.25"),
				PauliAngle::Clifford(CliffordPauliAngle::Zero) => String::from("0.0"),
				PauliAngle::Clifford(CliffordPauliAngle::NegPiOver4) => String::from("-0.25"),
				PauliAngle::Clifford(CliffordPauliAngle::NegPiOver2) => String::from("-0.5"),
			};
			let string = pauli.string.as_string();
			writeln!(&mut file, "{angle};{string}").unwrap();
		}
		file.flush().expect("Failed to write to file");
	}

	pub fn read_exp_file(path: &str) -> Vec<Self> {
		let file = File::open(path).expect("Failed to open file");
		io::BufReader::new(file)
			.lines()
			.map(|line| {
				let line = line.expect("Failed to read file");
				let (angle, letters) = line.split_once(';').expect("File format is wrong");
				let angle = match angle.parse::<f64>() {
					Ok(0.5) => PauliAngle::Clifford(CliffordPauliAngle::PiOver2),
					Ok(0.25) => PauliAngle::Clifford(CliffordPauliAngle::PiOver4),
					Ok(0.0) => PauliAngle::Clifford(CliffordPauliAngle::Zero),
					Ok(-0.25) => PauliAngle::Clifford(CliffordPauliAngle::NegPiOver4),
					Ok(-0.5) => PauliAngle::Clifford(CliffordPauliAngle::NegPiOver2),
					Ok(v) => PauliAngle::MultipleOfPi(v),
					Err(_) => match angle.strip_prefix('-') {
						Some(angle) => PauliAngle::Parameter {
							neg: true,
							name: String::from(angle),
						},
						_ => PauliAngle::Parameter {
							neg: false,
							name: String::from(angle),
						},
					},
				};

				if letters.len() != N {
					panic!("File contains an exp that is not of desired length.")
				}

				let mut string = PauliString::id();
				for (i, letter) in letters.chars().enumerate() {
					match letter {
						'X' => string.set(i, PauliLetter::X),
						'Y' => string.set(i, PauliLetter::Y),
						'Z' => string.set(i, PauliLetter::Z),
						'I' => {}
						_ => panic!("File format error, PauliString contains non-valid letters"),
					}
				}

				PauliExp { string, angle }
			})
			.collect()
	}
}

impl<const N: usize> From<PauliExp<N, CliffordPauliAngle>> for PauliExp<N, PauliAngle> {
	fn from(value: PauliExp<N, CliffordPauliAngle>) -> Self {
		PauliExp::<N, PauliAngle> {
			angle: value.angle.into(),
			string: value.string,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::super::PauliLetter;
	use super::*;

	#[test]
	fn red_pi_through_zz() {
		let mut letters = PauliString::<2>::z(0);
		letters.set(1, PauliLetter::Z);
		let mut zz = PauliExp {
			angle: PauliAngle::MultipleOfPi(-2.0),
			string: letters,
		};

		let red = PauliString::x(0);

		zz.push_pi_over_4(true, &red);
		zz.push_pi_over_4(true, &red);

		let mut zz_string = PauliString::z(0);
		zz_string.set(1, PauliLetter::Z);

		assert_eq!(PauliAngle::MultipleOfPi(2.0), zz.angle);
		assert_eq!(zz_string, zz.string);
	}
}
