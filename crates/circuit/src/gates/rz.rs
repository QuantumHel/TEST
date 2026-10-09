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
