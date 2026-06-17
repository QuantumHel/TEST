pub struct TargetStateIterator {
	min: usize,
	max: usize,
	now: usize,
}

impl TargetStateIterator {
	/// Targets are given as a usize equal to a bit string with ones at the target locations.
	/// Needs change if there are more than 64 qubits.
	pub fn new(targets: usize, n_qubits: usize) -> Self {
		Self {
			min: targets,
			max: 2usize.pow((n_qubits) as u32) - 1,
			now: targets - 1,
		}
	}
}

impl Iterator for TargetStateIterator {
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		self.now += 1;
		self.now |= self.min;
		if self.now > self.max {
			return None;
		}
		Some(self.now)
	}
}
