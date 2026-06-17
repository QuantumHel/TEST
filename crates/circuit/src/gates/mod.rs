use rand::{
	RngExt,
	distr::{Distribution, StandardUniform},
};

use crate::RandomGate;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rz<T> {
	pub angle: T,
	pub target: usize,
}

impl<T> RandomGate for Rz<T>
where
	StandardUniform: Distribution<T>,
{
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		Self {
			angle: rng.random(),
			target: rng.random_range(..n_qubits),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct X {
	pub target: usize,
}

impl RandomGate for X {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		Self {
			target: rng.random_range(..n_qubits),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Y {
	pub target: usize,
}

impl RandomGate for Y {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		Self {
			target: rng.random_range(..n_qubits),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct H {
	pub target: usize,
}

impl RandomGate for H {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		Self {
			target: rng.random_range(..n_qubits),
		}
	}
}

/// # Attention
/// Currently there is nothing stopping you from having control == target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CNot {
	control: usize,
	target: usize,
}

impl RandomGate for CNot {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		let control = rng.random_range(..n_qubits);
		let mut target = rng.random_range(..(n_qubits - 1));
		if target >= control {
			target += 1;
		}

		Self { control, target }
	}
}

impl CNot {
	/// Fails if contro == target
	pub fn new(control: usize, target: usize) -> Option<Self> {
		if control != target {
			Some(CNot { control, target })
		} else {
			None
		}
	}

	pub fn target(&self) -> usize {
		self.target
	}

	pub fn control(&self) -> usize {
		self.control
	}

	pub fn reverse(&self) -> Self {
		CNot {
			control: self.target,
			target: self.control,
		}
	}

	/// This should probably be moved to a trait
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
