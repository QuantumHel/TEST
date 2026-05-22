pub mod gates;

pub struct Circuit<T> {
	pub gates: Vec<T>,
}

impl<T> Circuit<T> {
	pub fn push<G: Into<T>>(&mut self, gate: G) {
		self.gates.push(gate.into());
	}
}
