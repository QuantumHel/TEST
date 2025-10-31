#[allow(unused_imports)] // This is for documentation
use super::PauliExp;

pub trait Negate {
	/// Negates the value in place
	fn negate(&mut self);
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

impl Negate for CliffordPauliAngle {
	fn negate(&mut self) {
		*self = match self {
			CliffordPauliAngle::NegPiOver2 => CliffordPauliAngle::PiOver2,
			CliffordPauliAngle::NegPiOver4 => CliffordPauliAngle::PiOver4,
			CliffordPauliAngle::Zero => CliffordPauliAngle::Zero,
			CliffordPauliAngle::PiOver4 => CliffordPauliAngle::NegPiOver4,
			CliffordPauliAngle::PiOver2 => CliffordPauliAngle::NegPiOver2,
		};
	}
}

/// An angle for [PauliExp] that can be whatever.
#[derive(Debug, Clone, PartialEq)]
pub enum PauliAngle {
	MultipleOfPi(f64),
	Clifford(CliffordPauliAngle),
	Parameter { neg: bool, name: String },
}

impl PauliAngle {
	pub fn is_clifford(&self) -> bool {
		matches!(self, PauliAngle::Clifford(_))
	}
}

impl Negate for PauliAngle {
	fn negate(&mut self) {
		match self {
			PauliAngle::MultipleOfPi(v) => *v = -*v,
			PauliAngle::Clifford(v) => v.negate(),
			PauliAngle::Parameter { neg, .. } => *neg = !*neg,
		}
	}
}

impl From<CliffordPauliAngle> for PauliAngle {
	fn from(value: CliffordPauliAngle) -> Self {
		Self::Clifford(value)
	}
}
