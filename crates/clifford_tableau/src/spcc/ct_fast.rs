use super::gateset::CNotHSV;
use crate::CliffordTableau;
use core::qubit_map::PartialQubitMap;
use core::qubit_map::QubitMap;
use pauli::PauliLetter;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CTFast {
	x_x_columns: Vec<Vec<u64>>,
	x_z_columns: Vec<Vec<u64>>,
	z_x_columns: Vec<Vec<u64>>,
	z_z_columns: Vec<Vec<u64>>,
	x_signs: Vec<u64>,
	z_signs: Vec<u64>,
	size: usize,
	row_blocks: usize,
}

impl CTFast {
	pub fn new(size: usize) -> Self {
		let row_blocks = size / 64 + 1;
		let mut x_x_columns = vec![vec![0; size]; row_blocks];
		let z_x_columns = vec![vec![0; size]; row_blocks];
		let x_z_columns = vec![vec![0; size]; row_blocks];
		let mut z_z_columns = vec![vec![0; size]; row_blocks];
		let x_signs = vec![0; row_blocks];
		let z_signs = vec![0; row_blocks];

		for col in 0..size {
			let row = col / 64;
			x_x_columns[row][col] = 1 << (col % 64);
			z_z_columns[row][col] = 1 << (col % 64);
		}
		Self {
			x_x_columns,
			z_x_columns,
			x_z_columns,
			z_z_columns,
			x_signs,
			z_signs,
			size,
			row_blocks,
		}
	}

	fn translate(ps: PauliLetter) -> u8 {
		match ps {
			PauliLetter::I => 0,
			PauliLetter::X => 1,
			PauliLetter::Z => 2,
			PauliLetter::Y => 3,
		}
	}

	pub fn from_clifford_tableau(tableau: &CliffordTableau) -> Self {
		let mut ct_fast = CTFast::new(tableau.size());
		for row in 0..tableau.size() {
			for col in 0..tableau.size() {
				ct_fast.set_x_row_col(row, col, Self::translate(tableau.get_x_row(row).get(col)));
				ct_fast.set_z_row_col(row, col, Self::translate(tableau.get_z_row(row).get(col)));
			}
			if tableau.get_x_signs().get(row) {
				ct_fast.set_x_phase(row, true);
			}
			if tableau.get_z_signs().get(row) {
				ct_fast.set_z_phase(row, true);
			}
		}
		ct_fast
	}

	pub fn permutation_tableau(size: usize, perm: &QubitMap) -> Self {
		let mut ct_permutation = CTFast::new(size);
		ct_permutation.merge_qubit_map(&perm.clone().as_reversed());
		ct_permutation
	}

	pub fn get_x_row_col(&self, row: usize, col: usize) -> u8 {
		let x = (self.x_x_columns[row / 64][col] & (1 << (row % 64))) != 0;
		let z = (self.z_x_columns[row / 64][col] & (1 << (row % 64))) != 0;
		(x as u8) | ((z as u8) << 1)
	}

	fn set_x_row_col(&mut self, row: usize, col: usize, pauli: u8) {
		let row_block = row / 64;
		let bit_index = row % 64;
		if pauli & 1 != 0 {
			self.x_x_columns[row_block][col] |= 1 << bit_index;
		} else {
			self.x_x_columns[row_block][col] &= !(1 << bit_index);
		}
		if pauli & 2 != 0 {
			self.z_x_columns[row_block][col] |= 1 << bit_index;
		} else {
			self.z_x_columns[row_block][col] &= !(1 << bit_index);
		}
	}

	pub fn get_z_row_col(&self, row: usize, col: usize) -> u8 {
		let x = (self.x_z_columns[row / 64][col] & (1 << (row % 64))) != 0;
		let z = (self.z_z_columns[row / 64][col] & (1 << (row % 64))) != 0;
		(x as u8) | ((z as u8) << 1)
	}

	fn set_z_row_col(&mut self, row: usize, col: usize, pauli: u8) {
		let row_block = row / 64;
		let bit_index = row % 64;
		if pauli & 1 != 0 {
			self.x_z_columns[row_block][col] |= 1 << bit_index;
		} else {
			self.x_z_columns[row_block][col] &= !(1 << bit_index);
		}
		if pauli & 2 != 0 {
			self.z_z_columns[row_block][col] |= 1 << bit_index;
		} else {
			self.z_z_columns[row_block][col] &= !(1 << bit_index);
		}
	}

	pub fn get_z_phase(&self, row: usize) -> bool {
		(self.z_signs[row / 64] & (1 << (row % 64))) != 0
	}

	fn set_z_phase(&mut self, row: usize, phase: bool) {
		let row_block = row / 64;
		let bit_index = row % 64;
		if phase {
			self.z_signs[row_block] |= 1 << bit_index;
		} else {
			self.z_signs[row_block] &= !(1 << bit_index);
		}
	}

	pub fn get_x_phase(&self, row: usize) -> bool {
		(self.x_signs[row / 64] & (1 << (row % 64))) != 0
	}

	fn set_x_phase(&mut self, row: usize, phase: bool) {
		let row_block = row / 64;
		let bit_index = row % 64;
		if phase {
			self.x_signs[row_block] |= 1 << bit_index;
		} else {
			self.x_signs[row_block] &= !(1 << bit_index);
		}
	}

	pub fn size(&self) -> usize {
		self.size
	}

	pub fn apply_h(&mut self, qubit: usize) {
		for row in 0..self.row_blocks {
			self.x_signs[row] ^= self.x_x_columns[row][qubit] & self.z_x_columns[row][qubit];
			self.z_signs[row] ^= self.x_z_columns[row][qubit] & self.z_z_columns[row][qubit];

			std::mem::swap(
				&mut self.x_x_columns[row][qubit],
				&mut self.z_x_columns[row][qubit],
			);
			std::mem::swap(
				&mut self.x_z_columns[row][qubit],
				&mut self.z_z_columns[row][qubit],
			);
		}
	}

	pub fn apply_s(&mut self, qubit: usize) {
		for row in 0..self.row_blocks {
			self.x_signs[row] ^= self.x_x_columns[row][qubit] & self.z_x_columns[row][qubit];
			self.z_signs[row] ^= self.x_z_columns[row][qubit] & self.z_z_columns[row][qubit];

			self.z_x_columns[row][qubit] ^= self.x_x_columns[row][qubit];
			self.z_z_columns[row][qubit] ^= self.x_z_columns[row][qubit];
		}
	}

	pub fn apply_sdg(&mut self, qubit: usize) {
		for row in 0..self.row_blocks {
			self.z_x_columns[row][qubit] ^= self.x_x_columns[row][qubit];
			self.z_z_columns[row][qubit] ^= self.x_z_columns[row][qubit];

			self.x_signs[row] ^= self.x_x_columns[row][qubit] & self.z_x_columns[row][qubit];
			self.z_signs[row] ^= self.x_z_columns[row][qubit] & self.z_z_columns[row][qubit];
		}
	}

	pub fn apply_v(&mut self, qubit: usize) {
		for row in 0..self.row_blocks {
			self.x_x_columns[row][qubit] ^= self.z_x_columns[row][qubit];
			self.x_z_columns[row][qubit] ^= self.z_z_columns[row][qubit];

			self.x_signs[row] ^= self.x_x_columns[row][qubit] & self.z_x_columns[row][qubit];
			self.z_signs[row] ^= self.x_z_columns[row][qubit] & self.z_z_columns[row][qubit];
		}
	}

	pub fn apply_vdg(&mut self, qubit: usize) {
		for row in 0..self.row_blocks {
			self.x_signs[row] ^= self.x_x_columns[row][qubit] & self.z_x_columns[row][qubit];
			self.z_signs[row] ^= self.x_z_columns[row][qubit] & self.z_z_columns[row][qubit];

			self.x_x_columns[row][qubit] ^= self.z_x_columns[row][qubit];
			self.x_z_columns[row][qubit] ^= self.z_z_columns[row][qubit];
		}
	}

	pub fn apply_cnot(&mut self, control: usize, target: usize) {
		for row in 0..self.row_blocks {
			//  if (p1 == 0b01 and p2 == 0b10) or (p1 == 0b11 and p2 == 0b11) --> change
			// ((z[control] ^ 1) & x[control]) & (x[target] ^ 1] & z[target] | ((z{control] & x[control])) & (x[target] & z[target])}))
			// let phase_change = (((cz ^ 1) & cx) & (tz & (tx ^ 1))) | (cz & cx & tz & tx);
			self.x_signs[row] ^= self.x_x_columns[row][control]
				& self.z_x_columns[row][control]
				& self.x_x_columns[row][target]
				& self.z_x_columns[row][target];

			self.x_x_columns[row][target] ^= self.x_x_columns[row][control];
			self.z_x_columns[row][control] ^= self.z_x_columns[row][target];

			self.x_signs[row] ^= self.x_x_columns[row][control]
				& self.z_x_columns[row][control]
				& self.x_x_columns[row][target]
				& self.z_x_columns[row][target];

			self.z_signs[row] ^= self.x_z_columns[row][control]
				& self.z_z_columns[row][control]
				& self.x_z_columns[row][target]
				& self.z_z_columns[row][target];

			self.x_z_columns[row][target] ^= self.x_z_columns[row][control];
			self.z_z_columns[row][control] ^= self.z_z_columns[row][target];

			self.z_signs[row] ^= self.x_z_columns[row][control]
				& self.z_z_columns[row][control]
				& self.x_z_columns[row][target]
				& self.z_z_columns[row][target];
		}
	}

	pub fn inverse(&self) -> Self {
		// First transpose
		let mut x2x = vec![vec![0; self.size]; self.size];
		let mut x2z = vec![vec![0; self.size]; self.size];
		let mut z2x = vec![vec![0; self.size]; self.size];
		let mut z2z = vec![vec![0; self.size]; self.size];
		for row in 0..self.size {
			for col in 0..self.size {
				x2x[row][col] = self.get_x_row_col(col, row) & 1;
				x2z[row][col] = (self.get_x_row_col(col, row) & 2) >> 1;
				z2x[row][col] = self.get_z_row_col(col, row) & 1;
				z2z[row][col] = (self.get_z_row_col(col, row) & 2) >> 1;
			}
		}
		let mut inverse = CTFast::new(self.size);
		for row in 0..self.size {
			for col in 0..self.size {
				let xs = z2z[row][col] + 2 * x2z[row][col];
				inverse.set_x_row_col(row, col, xs);
				let zs = z2x[row][col] + 2 * x2x[row][col];
				inverse.set_z_row_col(row, col, zs);
			}
		}

		// correct signs
		let sign_tableau = self.then(&inverse);
		inverse.then(&sign_tableau)
	}

	pub fn then(&self, other: &Self) -> Self {
		assert_eq!(
			self.size, other.size,
			"cannot compose tableaus with different numbers of qubits"
		);

		let mut result = CTFast::new(self.size);
		for row in 0..self.size {
			for output_is_x in [true, false] {
				// A tableau row is represented as (-1)^sign * product(X_i^x_i Z_i^z_i).
				// Apply `other` to the row by multiplying the images of its X and Z
				// factors in that order. q is the exponent of i in the intermediate
				// (possibly non-Hermitian) Pauli product.
				let mut x = vec![false; self.size];
				let mut z = vec![false; self.size];
				let mut q = if if output_is_x {
					self.get_x_phase(row)
				} else {
					self.get_z_phase(row)
				} {
					2
				} else {
					0
				};
				for col in 0..self.size {
					if (if output_is_x {
						self.get_x_row_col(row, col)
					} else {
						self.get_z_row_col(row, col)
					}) & 3 == 3
					{
						q = (q + 1) % 4;
					}
				}

				for source in 0..self.size {
					let source_pauli = if output_is_x {
						self.get_x_row_col(row, source)
					} else {
						self.get_z_row_col(row, source)
					};
					let x_factor = source_pauli & 1 != 0;
					let z_factor = source_pauli & 2 != 0;
					for (use_factor, factor_is_x) in [(x_factor, true), (z_factor, false)] {
						if !use_factor {
							continue;
						}

						let factor_phase = if factor_is_x {
							other.get_x_phase(source)
						} else {
							other.get_z_phase(source)
						};
						let mut factor_xz = 0;
						for col in 0..self.size {
							let pauli = if factor_is_x {
								other.get_x_row_col(source, col)
							} else {
								other.get_z_row_col(source, col)
							};
							let factor_x = pauli & 1 != 0;
							let factor_z = pauli & 2 != 0;
							q = (q + 2 * (z[col] && factor_x) as u8) % 4;
							x[col] ^= factor_x;
							z[col] ^= factor_z;
							factor_xz += (factor_x && factor_z) as u8;
						}
						q = (q + 2 * factor_phase as u8 + factor_xz % 4) % 4;
					}
				}

				let final_xz = x.iter().zip(&z).map(|(x, z)| (*x && *z) as u8).sum::<u8>() % 4;
				for col in 0..self.size {
					let pauli = x[col] as u8 | ((z[col] as u8) << 1);
					if output_is_x {
						result.set_x_row_col(row, col, pauli);
					} else {
						result.set_z_row_col(row, col, pauli);
					}
				}
				let phase = ((q + 4 - final_xz) % 4) == 2;
				if output_is_x {
					result.set_x_phase(row, phase);
				} else {
					result.set_z_phase(row, phase);
				}
			}
		}
		result
	}

	pub fn merge_clifford(&mut self, clifford: &CNotHSV) {
		match clifford {
			CNotHSV::CNot(cnot) => self.apply_cnot(*cnot.control(), *cnot.target()),
			CNotHSV::H(qubit) => self.apply_h(qubit.target),
			CNotHSV::S(qubit) => self.apply_s(qubit.target),
			CNotHSV::V(qubit) => self.apply_v(qubit.target),
			CNotHSV::Sdagger(qubit) => self.apply_sdg(qubit.target),
			CNotHSV::Vdagger(qubit) => self.apply_vdg(qubit.target),
		}
	}

	fn merge_qubit_map(&mut self, qubit_map: &QubitMap) {
		let old = self.clone();
		for row in 0..self.size {
			for col in 0..self.size {
				self.set_x_row_col(qubit_map.map(row), col, old.get_x_row_col(row, col));
				self.set_z_row_col(qubit_map.map(row), col, old.get_z_row_col(row, col));
			}
			self.set_x_phase(qubit_map.map(row), old.get_x_phase(row));
			self.set_z_phase(qubit_map.map(row), old.get_z_phase(row));
		}
	}

	pub fn merge_partial_qubit_map(&mut self, qubit_map: &PartialQubitMap, new_size: usize) {
		let old = self.clone();
		self.size = new_size;
		self.row_blocks = self.size / 64 + 1;
		self.x_x_columns = vec![vec![0; new_size]; self.row_blocks];
		self.z_x_columns = vec![vec![0; new_size]; self.row_blocks];
		self.x_z_columns = vec![vec![0; new_size]; self.row_blocks];
		self.z_z_columns = vec![vec![0; new_size]; self.row_blocks];
		self.x_signs = vec![0; self.row_blocks];
		self.z_signs = vec![0; self.row_blocks];

		for col in 0..self.size {
			let row = col / 64;
			self.x_x_columns[row][col] = 1 << (col % 64);
			self.z_z_columns[row][col] = 1 << (col % 64);
		}

		for row in 0..old.size {
			let map = qubit_map.map(row);
			if let Ok(map_row) = map {
				for col in 0..old.size {
					let map = qubit_map.map(col);
					if let Ok(map_col) = map {
						self.set_x_row_col(map_row, map_col, old.get_x_row_col(row, col));
						self.set_z_row_col(map_row, map_col, old.get_z_row_col(row, col));
					}
				}
				self.set_x_phase(map_row, old.get_x_phase(row));
				self.set_z_phase(map_row, old.get_z_phase(row));
			}
		}
	}
}

impl Display for CTFast {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		let mut result = String::new();

		for row in 0..self.size {
			if row == 0 {
				result.push_str("X: ");
			} else {
				result.push_str("   ");
			}
			for col in 0..self.size {
				let p = self.get_x_row_col(row, col);
				let pauli = match p {
					0 => ' ',
					1 => 'X',
					2 => 'Z',
					3 => 'Y',
					_ => unreachable!(),
				};
				result.push(pauli);
			}
			if self.get_x_phase(row) {
				result.push('-');
			} else {
				result.push('+');
			}
			result.push('\n');
		}
		for row in 0..self.size {
			if row == 0 {
				result.push_str("Z: ");
			} else {
				result.push_str("   ");
			}
			for col in 0..self.size {
				let p = self.get_z_row_col(row, col);
				let pauli = match p {
					0 => ' ',
					1 => 'X',
					2 => 'Z',
					3 => 'Y',
					_ => unreachable!(),
				};
				result.push(pauli);
			}
			if self.get_z_phase(row) {
				result.push('-');
			} else {
				result.push('+');
			}
			result.push('\n');
		}

		formatter.write_str(&result)
	}
}

#[cfg(test)]
mod tests {
	use super::CTFast;
	use crate::spcc::print_tableau::to_str;
	use circuit::Circuit;
	use circuit::gates::{CNot, H, S};
	use rand::SeedableRng;
	use rand_chacha::ChaCha8Rng;

	use super::CNotHSV;
	use crate::CliffordTableau;
	use core::qubit_map::PartialQubitMap;
	use core::qubit_map::QubitMap;

	#[test]
	fn prints_identity_tableau() {
		let tableau = CTFast::new(4);
		let res =
			"X: X   +\n    X  +\n     X +\n      X+\nZ: Z   +\n    Z  +\n     Z +\n      Z+\n";
		assert_eq!(tableau.to_string(), res);
	}

	#[test]
	fn random_tableau() {
		let gates: Vec<(char, (usize, usize))> = vec![
			('V', (3, 0)),
			('V', (3, 0)),
			('S', (3, 0)),
			('S', (3, 0)),
			('V', (2, 0)),
			('V', (2, 0)),
			('V', (1, 0)),
			('V', (1, 0)),
			('V', (0, 0)),
			('V', (0, 0)),
			('S', (0, 0)),
			('S', (0, 0)),
			('V', (1, 0)),
			('H', (1, 0)),
			('C', (0, 1)),
			('H', (1, 0)),
			('H', (0, 0)),
			('C', (0, 1)),
			('H', (1, 0)),
			('V', (2, 0)),
			('H', (2, 0)),
			('C', (0, 2)),
			('S', (2, 0)),
			('C', (1, 0)),
			('S', (0, 0)),
			('V', (1, 0)),
			('C', (0, 2)),
			('V', (2, 0)),
			('C', (1, 0)),
			('S', (0, 0)),
			('V', (1, 0)),
			('S', (3, 0)),
			('C', (1, 3)),
			('H', (3, 0)),
			('V', (1, 0)),
			('C', (1, 3)),
			('S', (3, 0)),
			('C', (0, 1)),
			('H', (1, 0)),
			('V', (0, 0)),
			('C', (0, 1)),
			('H', (1, 0)),
			('V', (0, 0)),
			('C', (0, 1)),
			('H', (1, 0)),
			('C', (2, 0)),
			('V', (0, 0)),
		];
		let mut tableau = CTFast::new(4);
		for (gate, (q1, q2)) in gates.iter() {
			match gate {
				'V' => tableau.apply_v(*q1),
				'S' => tableau.apply_s(*q1),
				'H' => tableau.apply_h(*q1),
				'C' => tableau.apply_cnot(*q1, *q2),
				_ => panic!("Unknown gate"),
			}
		}
		let res =
			"X:  XXY+\n   XZY +\n    ZYY+\n   Z ZZ-\nZ:   YY+\n     ZZ-\n    YXX+\n   X   -\n";
		assert_eq!(tableau.to_string(), res);
		let gatesr: Vec<(char, (usize, usize))> = gates.iter().rev().cloned().collect();
		for (gate, (q1, q2)) in gatesr {
			match gate {
				'V' => tableau.apply_vdg(q1),
				'S' => tableau.apply_sdg(q1),
				'H' => tableau.apply_h(q1),
				'C' => tableau.apply_cnot(q1, q2),
				_ => panic!("Unknown gate"),
			}
		}
		let res =
			"X: X   +\n    X  +\n     X +\n      X+\nZ: Z   +\n    Z  +\n     Z +\n      Z+\n";
		assert_eq!(tableau.to_string(), res);
	}

	#[test]
	fn test_equality() {
		let tableau = CTFast::new(4);
		let tableau2 = CTFast::new(4);
		assert_eq!(tableau, tableau2);
		let mut tableau3 = CTFast::new(4);
		assert_eq!(tableau, tableau3);
		tableau3.apply_h(0);
		assert_ne!(tableau, tableau3);
	}

	#[test]
	fn test_big_identity() {
		let tableau = CTFast::new(200);
		let tableau2 = CTFast::new(200);
		assert_eq!(tableau, tableau2);
		let mut tableau3 = CTFast::new(200);
		assert_eq!(tableau, tableau3);
		tableau3.apply_h(0);
		assert_ne!(tableau, tableau3);
	}

	#[test]
	fn test_clone() {
		let tableau = CTFast::new(4);
		let tableau2 = tableau.clone();
		assert_eq!(tableau, tableau2);
	}

	#[test]
	fn test_clone_big() {
		let tableau = CTFast::new(100);
		let tableau2 = tableau.clone();
		assert_eq!(tableau, tableau2);
	}

	#[test]
	fn test_then_composes_tableaus() {
		let mut first = CTFast::new(2);
		first.apply_h(0);
		first.apply_s(0);
		first.apply_cnot(0, 1);

		let mut second = CTFast::new(2);
		second.apply_v(1);
		second.apply_h(0);
		second.apply_sdg(1);

		let mut expected = CTFast::new(2);
		expected.apply_h(0);
		expected.apply_s(0);
		expected.apply_cnot(0, 1);
		expected.apply_v(1);
		expected.apply_h(0);
		expected.apply_sdg(1);

		assert!(second.get_z_phase(1));
		assert_eq!(first.then(&second), expected);
	}

	#[test]
	fn test_inverse() {
		let mut tableau = CTFast::new(2);
		tableau.apply_h(0);
		tableau.apply_s(0);
		tableau.apply_cnot(0, 1);
		tableau.apply_v(1);
		tableau.apply_h(0);
		tableau.apply_sdg(1);

		let inverse = tableau.inverse();
		assert_eq!(
			tableau.then(&inverse).to_string(),
			CTFast::new(2).to_string()
		);
	}

	#[test]
	fn test_big() {
		let mut tableau = CTFast::new(200);
		let tableau2 = CTFast::new(200);
		tableau.apply_s(100);
		assert_ne!(tableau, tableau2);
		tableau.apply_sdg(100);
		assert_eq!(tableau, tableau2);
	}

	#[test]
	fn test_to_clifford_tableau_identity_h() {
		const QUBITS: usize = 10;
		let mut tableau = CliffordTableau::id();
		let mut tableau_fast = CTFast::new(QUBITS);
		let gate = &CNotHSV::H(H { target: 9 });
		tableau.merge_clifford(gate);
		tableau_fast.merge_clifford(gate);
		assert_eq!(to_str(&tableau), tableau_fast.to_string());
	}

	#[test]
	fn test_to_clifford_tableau_identity_h_big() {
		const QUBITS: usize = 100;
		let mut tableau = CliffordTableau::id();
		let mut tableau_fast = CTFast::new(QUBITS);
		let gate = &CNotHSV::H(H { target: 99 });
		tableau.merge_clifford(gate);
		tableau_fast.merge_clifford(gate);
		assert_eq!(to_str(&tableau), tableau_fast.to_string());
	}

	#[test]
	fn test_to_clifford_tableau_identity_special() {
		const QUBITS: usize = 2;
		let mut tableau = CliffordTableau::id();
		let mut tableau_fast = CTFast::new(QUBITS);

		let cnot = CNot::new(0, 1).expect("CNOT control and target must differ");
		tableau.merge_clifford(&CNotHSV::CNot(cnot));
		tableau.merge_clifford(&CNotHSV::S(S { target: 0 }));
		tableau.merge_clifford(&CNotHSV::H(H { target: 1 }));
		tableau.merge_clifford(&CNotHSV::CNot(cnot));

		tableau_fast.merge_clifford(&CNotHSV::CNot(cnot));
		tableau_fast.merge_clifford(&CNotHSV::S(S { target: 0 }));
		tableau_fast.merge_clifford(&CNotHSV::H(H { target: 1 }));
		tableau_fast.merge_clifford(&CNotHSV::CNot(cnot));

		assert_eq!(to_str(&tableau), tableau_fast.to_string());
	}

	#[test]
	fn test_to_clifford_tableau() {
		const QUBITS: usize = 10;
		const ROUNDS: usize = 10;
		const GATES: usize = 1000;
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for _i in 0..ROUNDS {
			let mut tableau = CliffordTableau::id();
			let mut tableau_fast = CTFast::new(QUBITS);

			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			for gate in input.iter() {
				tableau.merge_clifford(gate);
				tableau_fast.merge_clifford(gate);
			}
			assert_eq!(to_str(&tableau), tableau_fast.to_string());
		}
	}

	#[test]
	fn test_permutation_map() {
		let mut tableau = CTFast::new(4);
		// Implement mapping 0->2, 2->1, 1->0, 3->3
		tableau.apply_cnot(0, 1);
		tableau.apply_cnot(1, 0);
		tableau.apply_cnot(0, 1);
		tableau.apply_cnot(2, 1);
		tableau.apply_cnot(1, 2);
		tableau.apply_cnot(2, 1);
		// Then we do opposite permutation to get back to identity
		let mut partial_mapping = PartialQubitMap::default();
		let perm_out = vec![1, 2, 0, 3];
		for (from, to) in perm_out.into_iter().enumerate() {
			partial_mapping.map_qubit(from, to).unwrap();
		}
		let mapping: QubitMap = partial_mapping.try_into().unwrap();
		let perm_tableau = CTFast::permutation_tableau(4, &mapping);
		tableau = tableau.then(&perm_tableau);
		assert_eq!(tableau, CTFast::new(4));
	}

	#[test]
	fn test_permutation_map_cliffordtableau() {
		let mut tableau = CliffordTableau::id();
		// Implement mapping 0->2, 2->1, 1->0, 3->3
		tableau.merge_clifford(&CNot::new(0, 1).unwrap());
		tableau.merge_clifford(&CNot::new(1, 0).unwrap());
		tableau.merge_clifford(&CNot::new(0, 1).unwrap());
		tableau.merge_clifford(&CNot::new(2, 1).unwrap());
		tableau.merge_clifford(&CNot::new(1, 2).unwrap());
		tableau.merge_clifford(&CNot::new(2, 1).unwrap());
		// Then we do opposite permutation to get back to identity
		let mut partial_mapping = PartialQubitMap::default();
		let perm_out = vec![1, 2, 0, 3];
		for (from, to) in perm_out.into_iter().enumerate() {
			partial_mapping.map_qubit(from, to).unwrap();
		}
		let mapping: QubitMap = partial_mapping.try_into().unwrap();
		tableau.merge_qubit_map(&mapping);
		assert_eq!(tableau, CliffordTableau::id());
	}

	#[test]
	fn test_merge_qubit_map() {
		const QUBITS: usize = 10;
		const ROUNDS: usize = 10;
		const GATES: usize = 1000;
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for _i in 0..ROUNDS {
			let mut tableau = CliffordTableau::id();
			let mut tableau_fast = CTFast::new(QUBITS);
			let m1 = rand::seq::index::sample(&mut rng, QUBITS, QUBITS);
			let m2 = rand::seq::index::sample(&mut rng, QUBITS, QUBITS);
			let perm_out = m1.iter().zip(m2.iter()).collect::<Vec<_>>();
			let mut partial_mapping = PartialQubitMap::default();
			for (from, to) in perm_out.into_iter() {
				partial_mapping.map_qubit(from, to).unwrap();
			}
			let mapping: QubitMap = partial_mapping.try_into().unwrap();

			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			for gate in input.iter() {
				tableau.merge_clifford(gate);
				tableau_fast.merge_clifford(gate);
			}

			tableau.merge_qubit_map(&mapping);
			let permutation = CTFast::permutation_tableau(QUBITS, &mapping);
			tableau_fast = tableau_fast.then(&permutation);
			assert_eq!(to_str(&tableau), tableau_fast.to_string());
		}
	}

	#[test]
	fn test_then_random_gates() {
		const QUBITS: usize = 10;
		const ROUNDS: usize = 30;
		const GATES: usize = 100;
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for _i in 0..ROUNDS {
			let mut tableau_a = CTFast::new(QUBITS);
			let mut tableau_b = CTFast::new(QUBITS);
			let mut tableau_ab = CTFast::new(QUBITS);

			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			for gate in input.iter() {
				tableau_a.merge_clifford(gate);
				tableau_ab.merge_clifford(gate);
			}

			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			for gate in input.iter() {
				tableau_b.merge_clifford(gate);
				tableau_ab.merge_clifford(gate);
			}
			assert_eq!(tableau_a.then(&tableau_b), tableau_ab);
		}
	}

	#[test]
	fn test_then_random_gates_big() {
		const QUBITS: usize = 100;
		const ROUNDS: usize = 30;
		const GATES: usize = 10000;
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for _i in 0..ROUNDS {
			let mut tableau_a = CTFast::new(QUBITS);
			let mut tableau_b = CTFast::new(QUBITS);
			let mut tableau_ab = CTFast::new(QUBITS);

			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			for gate in input.iter() {
				tableau_a.merge_clifford(gate);
				tableau_ab.merge_clifford(gate);
			}

			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			for gate in input.iter() {
				tableau_b.merge_clifford(gate);
				tableau_ab.merge_clifford(gate);
			}
			assert_eq!(tableau_a.then(&tableau_b), tableau_ab);
		}
	}

	#[test]
	fn test_inverse_random_gates() {
		const QUBITS: usize = 10;
		const ROUNDS: usize = 30;
		const GATES: usize = 100;
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for _i in 0..ROUNDS {
			let mut tableau_a = CTFast::new(QUBITS);

			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			for gate in input.iter() {
				tableau_a.merge_clifford(gate);
			}
			assert_eq!(tableau_a.then(&tableau_a.inverse()), CTFast::new(QUBITS));
		}
	}

	#[test]
	fn test_translate_random_gates() {
		const QUBITS: usize = 10;
		const ROUNDS: usize = 30;
		const GATES: usize = 100;
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for _i in 0..ROUNDS {
			let mut tableau = CliffordTableau::id();
			let mut tableau_a = CTFast::new(QUBITS);
			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			for gate in input.iter() {
				tableau_a.merge_clifford(gate);
				tableau.merge_clifford(gate);
			}
			let tableau_b = CTFast::from_clifford_tableau(&tableau);
			assert_eq!(tableau_a, tableau_b);
		}
	}

	#[test]
	fn test_to_clifford_tableau_big() {
		const QUBITS: usize = 100;
		const ROUNDS: usize = 10;
		const GATES: usize = 10000;
		let mut rng = ChaCha8Rng::seed_from_u64(67);
		for _i in 0..ROUNDS {
			let mut tableau = CliffordTableau::id();
			let mut tableau_fast = CTFast::new(QUBITS);

			let input: Circuit<CNotHSV> = Circuit::random(GATES, QUBITS, &mut rng);
			for gate in input.iter() {
				tableau.merge_clifford(gate);
				tableau_fast.merge_clifford(gate);
			}
			assert_eq!(to_str(&tableau), tableau_fast.to_string());
		}
	}
}
