mod decompose;

use bitvec::vec::BitVec;

use crate::pauli::PauliString;

#[derive(Debug, PartialEq, Eq)]
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

	/// This print exists for exploratory research.
	pub fn info_print(&self, n_rows: usize) {
		for i in 0..N.min(n_rows) {
			println!("X{}:\t{}", i, self.x.get(i).unwrap().as_string());
			println!("Z{}:\t{}", i, self.z.get(i).unwrap().as_string());
		}
	}
}
