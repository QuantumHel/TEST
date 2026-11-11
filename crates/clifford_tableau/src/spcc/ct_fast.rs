use super::gateset::CNotHSV;
use core::qubit_map::PartialQubitMap;
use core::qubit_map::QubitMap;
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

	pub fn merge_qubit_map(&mut self, qubit_map: &QubitMap) {
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
		println!("tableau:\n{}", tableau);
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
	fn test_merge_qubit_map() {
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

			let m1 = rand::seq::index::sample(&mut rng, QUBITS, QUBITS);
			let m2 = rand::seq::index::sample(&mut rng, QUBITS, QUBITS);
			let perm_out = m1.iter().zip(m2.iter()).collect::<Vec<_>>();
			let mut partial_mapping = PartialQubitMap::default();
			for (from, to) in perm_out.into_iter() {
				partial_mapping.map_qubit(from, to).unwrap();
			}
			let mapping: QubitMap = partial_mapping.try_into().unwrap();
			tableau.merge_qubit_map(&mapping);
			tableau_fast.merge_qubit_map(&mapping);
			assert_eq!(to_str(&tableau), tableau_fast.to_string());
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
