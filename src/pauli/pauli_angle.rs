#[allow(unused_imports)] // This is for documentation
use super::PauliExp;
use std::{fmt::Debug, ops::Neg};

/// A trait for angles used in [PauliExp].
pub trait PauliAngle: Neg<Output = Self> + Debug + Clone + Copy + PartialEq {
	fn multiple_of_pi(&self) -> f64;
}

/// An angle for [PauliExp] that is always Clifford
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CliffordPauliAngle {
	NegPiOver2,
	NegPiOver4,
	Zero,
	PiOver4,
	PiOver2,
}

impl Neg for CliffordPauliAngle {
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			CliffordPauliAngle::NegPiOver2 => CliffordPauliAngle::PiOver2,
			CliffordPauliAngle::NegPiOver4 => CliffordPauliAngle::PiOver4,
			CliffordPauliAngle::Zero => CliffordPauliAngle::Zero,
			CliffordPauliAngle::PiOver4 => CliffordPauliAngle::NegPiOver4,
			CliffordPauliAngle::PiOver2 => CliffordPauliAngle::NegPiOver2,
		}
	}
}

impl PauliAngle for CliffordPauliAngle {
	fn multiple_of_pi(&self) -> f64 {
		match self {
			CliffordPauliAngle::NegPiOver2 => -0.5,
			CliffordPauliAngle::NegPiOver4 => -0.25,
			CliffordPauliAngle::Zero => 0.0,
			CliffordPauliAngle::PiOver4 => 0.25,
			CliffordPauliAngle::PiOver2 => 0.5,
		}
	}
}

/// An angle for [PauliExp] that can be whatever.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FreePauliAngle {
	MultipleOfPi(f64),
	Clifford(CliffordPauliAngle),
}

impl FreePauliAngle {
	pub fn is_clifford(&self) -> bool {
		matches!(self, FreePauliAngle::Clifford(_))
	}
}

impl Neg for FreePauliAngle {
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			FreePauliAngle::MultipleOfPi(v) => FreePauliAngle::MultipleOfPi(-v),
			FreePauliAngle::Clifford(v) => FreePauliAngle::Clifford(-v),
		}
	}
}

impl PauliAngle for FreePauliAngle {
	fn multiple_of_pi(&self) -> f64 {
		match self {
			FreePauliAngle::MultipleOfPi(v) => *v,
			FreePauliAngle::Clifford(v) => v.multiple_of_pi(),
		}
	}
}

impl From<CliffordPauliAngle> for FreePauliAngle {
	fn from(value: CliffordPauliAngle) -> Self {
		Self::Clifford(value)
	}
}
