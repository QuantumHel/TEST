#[allow(unused_imports)] // This is for documentation
use super::PauliExp;
use std::{fmt::Debug, ops::Neg};

/// A trait for angles used in [PauliExp].
pub trait PauliAngle: Neg<Output = Self> + Debug + Clone + Copy + PartialEq {
	fn multiple_of_pi(&self) -> f64;
}

/// An angle for [PauliExp] that is always Clifford
///
/// TODO: Support all Clifford angles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CliffordPauliAngle {
	PiOver4,
	NeqPiOver4,
}

impl Neg for CliffordPauliAngle {
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			CliffordPauliAngle::PiOver4 => CliffordPauliAngle::NeqPiOver4,
			CliffordPauliAngle::NeqPiOver4 => CliffordPauliAngle::PiOver4,
		}
	}
}

impl PauliAngle for CliffordPauliAngle {
	fn multiple_of_pi(&self) -> f64 {
		match self {
			CliffordPauliAngle::PiOver4 => 0.25,
			CliffordPauliAngle::NeqPiOver4 => -0.25,
		}
	}
}

/// An angle for [PauliExp] that can be whatever.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FreePauliAngle {
	MultipleOfPi(f64),
	Clifford(CliffordPauliAngle),
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
