mod decompose;

use bits::Bits;

use crate::pauli::{CliffordPauliAngle, PauliExp, PauliString};

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

	pub fn id_with_capacity(capacity: usize) -> Self {
		CliffordTableau {
			x: (0..capacity)
				.map(|i| PauliString::x_with_capacity(i, capacity))
				.collect(),
			z: (0..capacity)
				.map(|i| PauliString::z_with_capacity(i, capacity))
				.collect(),
			x_signs: Bits::with_capacity(capacity),
			z_signs: Bits::with_capacity(capacity),
		}
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

	/// # merge pi over 4 pauli
	///
	/// Merges a $e^{\pm i\frac{\pi}{4}P}$ Pauli exponential into the tableau (On a
	/// circuit the Pauli would be originally on the right side of the tableau).
	pub fn merge_pi_over_4_pauli(&mut self, neg: bool, string: &PauliString) {
		let space_needed = string.size().saturating_sub(self.x.len());
		for _ in 0..space_needed {
			let i = self.x.len();
			self.x.push(PauliString::x(i));
			self.z.push(PauliString::z(i));
		}

		for (i, x) in self.x.iter_mut().enumerate() {
			if x.pi_over_4_sandwitch(neg, string) {
				let sign = self.x_signs.get(i);
				self.x_signs.set(i, !sign);
			}
		}

		for (i, z) in self.z.iter_mut().enumerate() {
			if z.pi_over_4_sandwitch(neg, string) {
				let sign = self.z_signs.get(i);
				self.z_signs.set(i, !sign);
			}
		}
	}

	/// # Merge Clifford
	///
	/// Merges a Clifford Pauli exponential into the tableau (On a
	/// circuit the Pauli would be originally on the right side of the tableau).
	pub fn merge_clifford(&mut self, clifford: PauliExp<CliffordPauliAngle>) {
		let space_needed = clifford.string.size().saturating_sub(self.x.len());
		for _ in 0..space_needed {
			let i = self.x.len();
			self.x.push(PauliString::x(i));
			self.z.push(PauliString::z(i));
		}

		match clifford.angle {
			CliffordPauliAngle::NegPiOver4 => {
				self.merge_pi_over_4_pauli(true, &clifford.string);
			}
			CliffordPauliAngle::PiOver4 => {
				self.merge_pi_over_4_pauli(false, &clifford.string);
			}
			// Firstly $e^{\pm i\pi O/2} = \cos(\pi/2)I\pm\sin(\pi/2)O=\pm O$
			// Then we get $e^{\pm i\pi O/2}Pe^{\mp i\pi O/2} = \pm OP(\mp O)=OPO
			// If $O$ and $P$ commute then $OPO=0^2P=P$
			// If $O$ and $P$ anticommute then $OPO=-O^2P=-P$
			CliffordPauliAngle::NegPiOver2 | CliffordPauliAngle::PiOver2 => {
				for (i, string) in self.x.iter().enumerate() {
					if string.anticommutes_with(&clifford.string) {
						let old = self.x_signs.get(i);
						self.x_signs.set(i, !old);
					}
				}

				for (i, string) in self.z.iter().enumerate() {
					if string.anticommutes_with(&clifford.string) {
						let old = self.z_signs.get(i);
						self.z_signs.set(i, !old);
					}
				}
			}
			CliffordPauliAngle::Zero => {}
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
