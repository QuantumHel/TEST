use core::qubit_map::QubitMap;

use crate::xor_span::XorSpan;

use super::Parity;

#[derive(Default, Clone, Debug)]
pub struct State {
	qubit_parities: Vec<Parity>,
}

impl State {
	pub fn new(n: usize) -> Self {
		Self {
			qubit_parities: (0..n).map(Parity::for_qubit).collect(),
		}
	}

	pub fn create_mapped(&self, qubit_map: &QubitMap) -> Self {
		let mut new_parities = self.parities().to_vec();
		for (a, b) in qubit_map.non_trivial_mappings() {
			new_parities[b] = self.get_cloned(a);
		}

		Self {
			qubit_parities: new_parities,
		}
	}

	pub fn parities(&self) -> &[Parity] {
		&self.qubit_parities
	}

	pub fn mut_parities(&mut self) -> &mut [Parity] {
		&mut self.qubit_parities
	}

	pub fn get_mut(&mut self, qubit: usize) -> &mut Parity {
		while self.qubit_parities.len() <= qubit {
			self.qubit_parities
				.push(Parity::for_qubit(self.qubit_parities.len()));
		}

		self.qubit_parities.get_mut(qubit).unwrap()
	}

	pub fn get_cloned(&self, qubit: usize) -> Parity {
		self.qubit_parities
			.get(qubit)
			.cloned()
			.unwrap_or(Parity::for_qubit(qubit))
	}

	pub fn create_span(&self) -> XorSpan<Parity> {
		XorSpan::new(&self.qubit_parities)
	}

	pub fn apply_cnot(&mut self, control: usize, target: usize) {
		let control = self.get_cloned(control);
		*self.get_mut(target) ^= control;
	}

	pub fn apply_x(&mut self, target: usize) {
		let target = self.get_mut(target);
		target.bit_flip = !target.bit_flip;
	}
}
