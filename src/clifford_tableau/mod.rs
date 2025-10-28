mod decompose;

use bitvec::vec::BitVec;

use crate::pauli::{CliffordPauliAngle, PauliExp, PauliString};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliffordTableau<const N: usize> {
	x: Vec<PauliString<N>>,
	z: Vec<PauliString<N>>,
	x_signs: BitVec,
	z_signs: BitVec,
}

impl<const N: usize> Default for CliffordTableau<N> {
	fn default() -> Self {
		CliffordTableau {
			x: (0..N).map(PauliString::<N>::x).collect(),
			z: (0..N).map(PauliString::<N>::z).collect(),
			x_signs: BitVec::repeat(false, N),
			z_signs: BitVec::repeat(false, N),
		}
	}
}

impl<const N: usize> CliffordTableau<N> {
	pub fn id() -> Self {
		Self::default()
	}

	pub fn is_identity(&mut self) -> bool {
		if self.x_signs.first_one().is_some() {
			return false;
		}

		if self.z_signs.first_one().is_some() {
			return false;
		}

		for (i, x) in self.x.iter().enumerate() {
			if PauliString::<N>::x(i) != *x {
				return false;
			}
		}

		for (i, z) in self.z.iter().enumerate() {
			if PauliString::<N>::z(i) != *z {
				return false;
			}
		}

		true
	}

	/// # merge pi over 4 pauli
	///
	/// Merges a $e^{\pm i\frac{\pi}{4}P}$ Pauli exponential into the tableau (On a
	/// circuit the Pauli would be originally on the right side of the tableau).
	pub fn merge_pi_over_4_pauli(&mut self, neg: bool, string: &PauliString<N>) {
		for (i, x) in self.x.iter_mut().enumerate() {
			if x.pi_over_4_sandwitch(neg, string) {
				let sign = self.x_signs[i];
				self.x_signs.set(i, !sign);
			}
		}

		for (i, z) in self.z.iter_mut().enumerate() {
			if z.pi_over_4_sandwitch(neg, string) {
				let sign = self.z_signs[i];
				self.z_signs.set(i, !sign);
			}
		}
	}

	/// # Merge Clifford
	///
	/// Merges a Clifford Pauli exponential into the tableau (On a
	/// circuit the Pauli would be originally on the right side of the tableau).
	pub fn merge_clifford(&mut self, clifford: PauliExp<N, CliffordPauliAngle>) {
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
						let old = self.x_signs[i];
						self.x_signs.set(i, !old);
					}
				}

				for (i, string) in self.z.iter().enumerate() {
					if string.anticommutes_with(&clifford.string) {
						let old = self.z_signs[i];
						self.z_signs.set(i, !old);
					}
				}
			}
			CliffordPauliAngle::Zero => {}
		}
	}

	pub fn get_x_row(&self, index: usize) -> Option<(PauliString<N>, bool)> {
		if index >= N {
			return None;
		}
		Some((self.x.get(index).unwrap().clone(), self.x_signs[index]))
	}

	pub fn get_z_row(&self, index: usize) -> Option<(PauliString<N>, bool)> {
		if index >= N {
			return None;
		}
		Some((self.z.get(index).unwrap().clone(), self.z_signs[index]))
	}

	pub fn get_x_signs(&self) -> BitVec {
		self.x_signs.clone()
	}

	pub fn get_z_signs(&self) -> BitVec {
		self.z_signs.clone()
	}

	/// This print exists for exploratory research.
	pub fn info_print(&self, n_rows: usize) {
		for i in 0..N.min(n_rows) {
			println!("X{}:\t{}", i, self.x.get(i).unwrap().as_string());
			println!("Z{}:\t{}", i, self.z.get(i).unwrap().as_string());
		}
	}
}
