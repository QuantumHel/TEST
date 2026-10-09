#[cfg(test)]
mod random_test;
pub mod spcc;

use bits::Bits;

use pauli::{Clifford, PauliString};

#[derive(Clone, Debug, Default, Eq)]
pub struct CliffordTableau {
	x: Vec<PauliString>,
	z: Vec<PauliString>,
	x_signs: Bits,
	z_signs: Bits,
}

impl CliffordTableau {
	pub fn id() -> Self {
		Self::default()
	}

	pub fn size(&self) -> usize {
		self.x.len().max(self.z.len())
	}

	pub fn is_identity(&mut self) -> bool {
		if self.x_signs.last_one().is_some() {
			return false;
		}

		if self.z_signs.last_one().is_some() {
			return false;
		}

		for (i, x) in self.x.iter().enumerate() {
			if PauliString::x(i) != *x {
				return false;
			}
		}

		for (i, z) in self.z.iter().enumerate() {
			if PauliString::z(i) != *z {
				return false;
			}
		}

		true
	}

	pub fn is_identity_qubit(&self, index: usize) -> bool {
		if !self
			.x
			.get(index)
			.map(|r| *r == PauliString::x(index))
			.unwrap_or(true)
		{
			return false;
		}

		if !self
			.z
			.get(index)
			.map(|r| *r == PauliString::z(index))
			.unwrap_or(true)
		{
			return false;
		}

		!(self.x_signs.get(index) || self.z_signs.get(index))
	}

	/// # Merge Clifford
	///
	/// Merges a Clifford into the tableau (On a circuit the Pauli would be
	/// originally on the right side of the tableau).
	pub fn merge_clifford<C: Clifford>(&mut self, clifford: &C) {
		let Some(n_qubits) = clifford.interacting_qubits().max() else {
			return;
		};
		// Makes sure that the tableau is enough big to represent the Clifford
		// operation.
		for _ in 0..(n_qubits - self.x.len()) {
			let i = self.x.len();
			self.x.push(PauliString::x(i));
			self.z.push(PauliString::z(i));
		}

		for (i, x_row) in self.x.iter_mut().enumerate() {
			let (sign, new_row) = clifford.conjugate(x_row);
			*x_row = new_row;
			if sign {
				self.x_signs.set(i, !self.x_signs.get(i));
			}
		}

		for (i, z_row) in self.z.iter_mut().enumerate() {
			let (sign, new_row) = clifford.conjugate(z_row);
			*z_row = new_row;
			if sign {
				self.z_signs.set(i, !self.z_signs.get(i));
			}
		}
	}

	pub fn get_x_row(&self, index: usize) -> PauliString {
		self.x.get(index).cloned().unwrap_or(PauliString::x(index))
	}

	pub fn get_z_row(&self, index: usize) -> PauliString {
		self.z.get(index).cloned().unwrap_or(PauliString::z(index))
	}

	pub fn get_x_signs(&self) -> Bits {
		self.x_signs.clone()
	}

	pub fn get_z_signs(&self) -> Bits {
		self.z_signs.clone()
	}

	/// This print exists for exploratory research.
	pub fn info_print(&self, n_rows: usize) {
		for i in 0..n_rows {
			println!(
				"X{}:\t{}",
				i,
				self.x.get(i).unwrap_or(&PauliString::x(i)).as_string()
			);
			println!(
				"Z{}:\t{}",
				i,
				self.z.get(i).unwrap_or(&PauliString::z(i)).as_string()
			);
		}
	}
}

impl PartialEq for CliffordTableau {
	fn eq(&self, other: &Self) -> bool {
		let mut this = self.x.iter().enumerate();
		let mut that = other.x.iter().enumerate();
		loop {
			match (this.next(), that.next()) {
				(Some((_, this)), Some((_, that))) => {
					if this != that {
						return false;
					}
				}
				(Some((i, this)), None) => {
					if *this != PauliString::x(i) {
						return false;
					}
				}
				(None, Some((i, that))) => {
					if *that != PauliString::x(i) {
						return false;
					}
				}
				(None, None) => break,
			}
		}

		let mut this = self.z.iter().enumerate();
		let mut that = other.z.iter().enumerate();
		loop {
			match (this.next(), that.next()) {
				(Some((_, this)), Some((_, that))) => {
					if this != that {
						return false;
					}
				}
				(Some((i, this)), None) => {
					if *this != PauliString::z(i) {
						return false;
					}
				}
				(None, Some((i, that))) => {
					if *that != PauliString::z(i) {
						return false;
					}
				}
				(None, None) => break,
			}
		}

		if self.x_signs != other.x_signs {
			return false;
		}

		if self.z_signs != other.z_signs {
			return false;
		}

		true
	}
}
