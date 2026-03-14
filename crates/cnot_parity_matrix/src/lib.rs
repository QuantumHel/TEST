mod parity_matrix;
mod patel_markov_hayes;

pub use parity_matrix::ParityMatrix;
pub use patel_markov_hayes::PatelMarkovHayes;

/// # Attention
/// Currently there is nothing stopping you from having control == target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CNot {
	pub control: usize,
	pub target: usize,
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
