use std::marker::PhantomData;

use circuit::{Circuit, RandomGate};
use rand_chacha::ChaCha8Rng;

pub struct RandomCircuitSource<'a, T: RandomGate> {
	qubit_count: usize,
	gate_count: usize,
	amount: usize,
	i: usize,
	rng: &'a mut ChaCha8Rng,
	_phantom: PhantomData<T>,
}

impl<'a, T: RandomGate> RandomCircuitSource<'a, T> {
	pub fn new(config: RandomCircuitSourceConfig, rng: &'a mut ChaCha8Rng) -> Self {
		Self {
			qubit_count: config.qubit_count,
			gate_count: config.gate_count,
			amount: config.circuit_count,
			i: 0,
			rng,
			_phantom: PhantomData,
		}
	}
}

pub struct RandomCircuitSourceConfig {
	pub qubit_count: usize,
	pub gate_count: usize,
	pub circuit_count: usize,
}

impl<'a, T: RandomGate> Iterator for RandomCircuitSource<'a, T> {
	type Item = (String, Circuit<T>);
	fn next(&mut self) -> Option<Self::Item> {
		if self.i >= self.amount {
			return None;
		}
		self.i += 1;
		Some((
			format!("random_circuit_{}", self.i),
			Circuit::random(self.gate_count, self.qubit_count, &mut self.rng),
		))
	}
}
