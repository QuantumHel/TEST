#[allow(unused_imports)] // This is for documentation
use super::PauliExp;
use std::{fmt::Debug, ops::Neg};

/// A trait for angles used in [PauliExp].
pub trait PauliAngle: Neg<Output = Self> + Debug + Clone + Copy + PartialEq {}

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

impl PauliAngle for CliffordPauliAngle {}

/// An angle for [PauliExp] that can be whatever.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FreePauliAngle {
	Free(f64),
	Clifford(CliffordPauliAngle),
}

impl Neg for FreePauliAngle {
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			FreePauliAngle::Free(v) => FreePauliAngle::Free(-v),
			FreePauliAngle::Clifford(v) => FreePauliAngle::Clifford(-v),
		}
	}
}

impl PauliAngle for FreePauliAngle {}
