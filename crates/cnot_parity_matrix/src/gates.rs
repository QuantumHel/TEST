use std::{f32::consts::PI, ops::AddAssign};

/// # Attention
/// Currently there is nothing stopping you from having control == target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CNot {
	pub control: usize,
	pub target: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Angle {
	QuarterPi(u32),
	Free(f32),
}

impl Angle {
	pub fn is_zero(&self) -> bool {
		matches!(self, Angle::QuarterPi(0) | Angle::Free(0.))
	}
}

impl AddAssign for Angle {
	fn add_assign(&mut self, rhs: Self) {
		*self = match (*self, rhs) {
			(Angle::QuarterPi(a), Self::QuarterPi(b)) => Angle::QuarterPi((a + b) % 8),
			(Angle::QuarterPi(a), Angle::Free(b)) => Angle::Free(a as f32 * PI / 4. + b),
			(Angle::Free(a), Angle::QuarterPi(b)) => Angle::Free(a + b as f32 * PI / 4.),
			(Angle::Free(a), Angle::Free(b)) => Angle::Free(a + b),
		};
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rz {
	pub angle: Angle,
	pub target: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct X {
	pub target: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Y {
	pub target: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct H {
	pub target: usize,
}

#[derive(Debug)]
pub enum CNotRzXYH {
	CNot(CNot),
	Rz(Rz),
	X(X),
	Y(Y),
	H(H),
}

/// Will be deprecated
#[derive(Debug, Clone, Copy)]
pub enum CNotRz {
	CNot(CNot),
	Rz(Rz),
}

impl CNot {
	pub fn new(control: usize, target: usize) -> Self {
		CNot { control, target }
	}

	pub fn reverse(&self) -> Self {
		CNot {
			control: self.target,
			target: self.control,
		}
	}

	pub fn random<R: rand::prelude::Rng>(qubits: usize, rng: &mut R) -> Self {
		use rand::RngExt;

		let control = (qubits as f64 * rng.random::<f64>()).floor() as usize;
		let mut target = ((qubits - 1) as f64 * rng.random::<f64>()).floor() as usize;
		// Need to make sure we get different target
		if target >= control {
			target += 1;
		}
		CNot { control, target }
	}
}
