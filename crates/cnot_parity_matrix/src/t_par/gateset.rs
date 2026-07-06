use std::ops::AddAssign;

use circuit::{
	RandomGate,
	gates::{CNot, H, Rz, X, Y},
};
use rand::{
	RngExt,
	distr::{Distribution, StandardUniform},
};
use simulator::{Complex, Simulatable};

use super::squirrel::Squirrel;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuarterPi(pub u32);

impl Distribution<QuarterPi> for StandardUniform {
	fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> QuarterPi {
		QuarterPi(rng.random_range(..8))
	}
}

impl AddAssign for QuarterPi {
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0;
		self.0 %= 8;
	}
}

#[derive(Debug, Clone, Copy)]
pub enum CNotRzXYH {
	CNot(CNot),
	Rz(Rz<QuarterPi>),
	X(X),
	Y(Y),
	H(H),
}

impl CNotRzXYH {
	/// Returns the index of the highest used qubit + 1
	pub fn n_required_qubits(&self) -> usize {
		1 + match self {
			Self::CNot(cnot) => cnot.target().max(cnot.control()),
			Self::Rz(rz) => rz.target,
			Self::X(x) => x.target,
			Self::Y(y) => y.target,
			Self::H(h) => h.target,
		}
	}
}

impl Simulatable<Squirrel> for CNotRzXYH {
	fn matrix(&self) -> [Complex<Squirrel>; 4] {
		match &self {
			CNotRzXYH::CNot(cnot) => cnot.matrix(),
			CNotRzXYH::Rz(rz) => rz.matrix(),
			CNotRzXYH::X(x) => x.matrix(),
			CNotRzXYH::Y(y) => y.matrix(),
			CNotRzXYH::H(h) => h.matrix(),
		}
	}

	fn target(&self) -> usize {
		match &self {
			CNotRzXYH::CNot(cnot) => cnot.target(),
			CNotRzXYH::Rz(rz) => rz.target(),
			CNotRzXYH::X(x) => x.target(),
			CNotRzXYH::Y(y) => y.target(),
			CNotRzXYH::H(h) => h.target(),
		}
	}

	fn controls(&self) -> Vec<usize> {
		match &self {
			CNotRzXYH::CNot(cnot) => cnot.controls(),
			CNotRzXYH::Rz(rz) => rz.controls(),
			CNotRzXYH::X(x) => x.controls(),
			CNotRzXYH::Y(y) => y.controls(),
			CNotRzXYH::H(h) => h.controls(),
		}
	}
}

impl RandomGate for CNotRzXYH {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		match rng.random_range(0..5) {
			0 => Self::CNot(CNot::random(n_qubits, rng)),
			1 => Self::Rz(Rz::random(n_qubits, rng)),
			2 => Self::X(X::random(n_qubits, rng)),
			3 => Self::Y(Y::random(n_qubits, rng)),
			_ => Self::H(H::random(n_qubits, rng)),
		}
	}
}

impl From<CNot> for CNotRzXYH {
	fn from(value: CNot) -> Self {
		CNotRzXYH::CNot(value)
	}
}

impl Simulatable<Squirrel> for CNot {
	fn matrix(&self) -> [Complex<Squirrel>; 4] {
		[
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::one(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::one(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
		]
	}

	fn controls(&self) -> Vec<usize> {
		vec![self.control()]
	}

	fn target(&self) -> usize {
		self.target()
	}
}

impl From<Rz<QuarterPi>> for CNotRzXYH {
	fn from(value: Rz<QuarterPi>) -> Self {
		CNotRzXYH::Rz(value)
	}
}

impl Simulatable<Squirrel> for Rz<QuarterPi> {
	fn matrix(&self) -> [Complex<Squirrel>; 4] {
		[
			Complex {
				re: Squirrel::one(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
			Complex {
				re: match self.angle.0 % 8 {
					0 => Squirrel::one(),
					1 | 7 => Squirrel::divided_by_sqrt_2(),
					2 | 6 => Squirrel::zero(),
					3 | 5 => -Squirrel::divided_by_sqrt_2(),
					4 => -Squirrel::one(),
					_ => unreachable!(),
				},
				im: match self.angle.0 % 8 {
					0 | 4 => Squirrel::zero(),
					1 | 3 => Squirrel::divided_by_sqrt_2(),
					2 => Squirrel::one(),
					5 | 7 => -Squirrel::divided_by_sqrt_2(),
					6 => -Squirrel::one(),
					_ => unreachable!(),
				},
			},
		]
	}

	fn controls(&self) -> Vec<usize> {
		Vec::new()
	}

	fn target(&self) -> usize {
		self.target
	}
}

impl From<X> for CNotRzXYH {
	fn from(value: X) -> Self {
		CNotRzXYH::X(value)
	}
}

impl Simulatable<Squirrel> for X {
	fn matrix(&self) -> [Complex<Squirrel>; 4] {
		[
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::one(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::one(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
		]
	}

	fn controls(&self) -> Vec<usize> {
		Vec::new()
	}

	fn target(&self) -> usize {
		self.target
	}
}

impl From<Y> for CNotRzXYH {
	fn from(value: Y) -> Self {
		CNotRzXYH::Y(value)
	}
}

impl Simulatable<Squirrel> for Y {
	fn matrix(&self) -> [Complex<Squirrel>; 4] {
		[
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::zero(),
				im: -Squirrel::one(),
			},
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::one(),
			},
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
		]
	}

	fn controls(&self) -> Vec<usize> {
		Vec::new()
	}

	fn target(&self) -> usize {
		self.target
	}
}

impl From<H> for CNotRzXYH {
	fn from(value: H) -> Self {
		CNotRzXYH::H(value)
	}
}

impl Simulatable<Squirrel> for H {
	fn matrix(&self) -> [Complex<Squirrel>; 4] {
		[
			Complex {
				re: Squirrel::divided_by_sqrt_2(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::divided_by_sqrt_2(),
				im: -Squirrel::zero(),
			},
			Complex {
				re: Squirrel::divided_by_sqrt_2(),
				im: Squirrel::zero(),
			},
			Complex {
				re: -Squirrel::divided_by_sqrt_2(),
				im: Squirrel::zero(),
			},
		]
	}

	fn controls(&self) -> Vec<usize> {
		Vec::new()
	}

	fn target(&self) -> usize {
		self.target
	}
}
