use super::gateset::CNotHSV;

#[derive(Debug, Eq)]
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

	pub fn get_x_row_col(&self, row: usize, col: usize) -> u8 {
		let x = (self.x_x_columns[row / 64][col] & (1 << (row % 64))) != 0;
		let z = (self.z_x_columns[row / 64][col] & (1 << (row % 64))) != 0;
		(x as u8) | ((z as u8) << 1)
	}

	pub fn get_z_row_col(&self, row: usize, col: usize) -> u8 {
		let x = (self.x_z_columns[row / 64][col] & (1 << (row % 64))) != 0;
		let z = (self.z_z_columns[row / 64][col] & (1 << (row % 64))) != 0;
		(x as u8) | ((z as u8) << 1)
	}

	pub fn get_z_phase(&self, row: usize) -> bool {
		(self.z_signs[row / 64] & (1 << (row % 64))) != 0
	}

	pub fn get_x_phase(&self, row: usize) -> bool {
		(self.x_signs[row / 64] & (1 << (row % 64))) != 0
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

	pub fn apply_sdag(&mut self, qubit: usize) {
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

	pub fn apply_vdag(&mut self, qubit: usize) {
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

	pub fn merge_clifford(&mut self, clifford: &CNotHSV) {
		match clifford {
			CNotHSV::CNot(cnot) => self.apply_cnot(*cnot.control(), *cnot.target()),
			CNotHSV::H(qubit) => self.apply_h(qubit.target),
			CNotHSV::S(qubit) => self.apply_s(qubit.target),
			CNotHSV::V(qubit) => self.apply_v(qubit.target),
			CNotHSV::Sdagger(qubit) => self.apply_sdag(qubit.target),
			CNotHSV::Vdagger(qubit) => self.apply_vdag(qubit.target),
		}
	}

	pub fn display(&self) -> String {
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

		result
	}
}

impl PartialEq for CTFast {
	fn eq(&self, other: &Self) -> bool {
		self.x_x_columns == other.x_x_columns
			&& self.x_z_columns == other.x_z_columns
			&& self.z_x_columns == other.z_x_columns
			&& self.z_z_columns == other.z_z_columns
			&& self.x_signs == other.x_signs
			&& self.z_signs == other.z_signs
			&& self.size == other.size
			&& self.row_blocks == other.row_blocks
	}
}
impl Clone for CTFast {
	fn clone(&self) -> Self {
		Self {
			x_x_columns: self.x_x_columns.clone(),
			x_z_columns: self.x_z_columns.clone(),
			z_x_columns: self.z_x_columns.clone(),
			z_z_columns: self.z_z_columns.clone(),
			x_signs: self.x_signs.clone(),
			z_signs: self.z_signs.clone(),
			size: self.size,
			row_blocks: self.row_blocks,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::CTFast;

	#[test]
	fn prints_identity_tableau() {
		let tableau = CTFast::new(4);
		let res =
			"X: X   +\n    X  +\n     X +\n      X+\nZ: Z   +\n    Z  +\n     Z +\n      Z+\n";
		println!("Identity tableau:\n{}", tableau.display());
		assert_eq!(tableau.display(), res);
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
		println!("tableau:\n{}", tableau.display());
		assert_eq!(tableau.display(), res);
		let gatesr: Vec<(char, (usize, usize))> = gates.iter().rev().cloned().collect();
		for (gate, (q1, q2)) in gatesr {
			match gate {
				'V' => tableau.apply_vdag(q1),
				'S' => tableau.apply_sdag(q1),
				'H' => tableau.apply_h(q1),
				'C' => tableau.apply_cnot(q1, q2),
				_ => panic!("Unknown gate"),
			}
		}
		let res =
			"X: X   +\n    X  +\n     X +\n      X+\nZ: Z   +\n    Z  +\n     Z +\n      Z+\n";
		assert_eq!(tableau.display(), res);
	}
}
