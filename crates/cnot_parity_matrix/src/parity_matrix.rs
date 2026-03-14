use std::ops::{Range, RangeBounds};

use bits::Bits;

use crate::CNot;

#[derive(Debug, Default, Clone, Copy)]
pub enum Basis {
	#[default]
	Standard,
	Hadamard,
}

#[derive(Debug, Clone, Default)]
pub struct ParityMatrix {
	rows: Vec<Bits>,
	basis: Basis,
}

impl ParityMatrix {
	pub fn standard_basis() -> Self {
		Self {
			rows: Vec::new(),
			basis: Basis::Standard,
		}
	}

	pub fn hadamard_basis() -> Self {
		Self {
			rows: Vec::new(),
			basis: Basis::Hadamard,
		}
	}

	pub fn basis(&self) -> Basis {
		self.basis
	}

	pub fn get(&self, row: usize, col: usize) -> bool {
		self.rows.get(row).unwrap_or(&Bits::with_one(row)).get(col)
	}

	pub fn get_section<T: RangeBounds<usize>>(&self, row: usize, cols: Range<usize>) -> Bits {
		self.rows
			.get(row)
			.unwrap_or(&Bits::with_one(row))
			.get_range(cols)
	}

	pub fn insert_cnot(&mut self, cnot: CNot) {
		match self.basis {
			Basis::Standard => self.add_row(cnot.control, cnot.target),
			Basis::Hadamard => self.add_row(cnot.target, cnot.control),
		};
	}

	pub fn add_row(&mut self, source: usize, target: usize) -> CNot {
		while self.rows.len() <= target {
			self.rows.push(Bits::with_one(self.rows.len()));
		}

		// We made sure above that we can unwrap on target.
		if let Some(source) = self.rows.get(source) {
			// Compiler probably removes this clone
			let source = source.clone();
			*self.rows.get_mut(target).unwrap() ^= source;
		} else {
			let target = self.rows.get_mut(target).unwrap();
			target.set(source, !target.get(source));
		}

		match self.basis {
			Basis::Standard => CNot {
				control: source,
				target,
			},
			Basis::Hadamard => CNot {
				control: target,
				target: source,
			},
		}
	}

	pub fn transpose(&self) -> Self {
		let size = self.size();
		let mut transpose = ParityMatrix {
			rows: vec![Bits::default(); size],
			basis: self.basis,
		};
		for (i, row) in self.rows.iter().enumerate().rev() {
			for j in row.iter_ones() {
				transpose
					.rows
					.get_mut(j)
					.expect("Size function failed?")
					.set(i, true);
			}
		}

		if size > self.rows.len() {
			for i in self.rows.len()..size {
				transpose
					.rows
					.get_mut(i)
					.expect("Size function failed?")
					.set(i, true);
			}
		}

		transpose
	}

	pub fn size(&self) -> usize {
		let mut size = 0;
		for (i, row) in self.rows.iter().enumerate() {
			size = size.max(
				row.last_one()
					.expect("Should not be able to have empty rows")
					+ 1,
			);
			if *row != Bits::with_one(i) {
				size = size.max(i + 1);
			}
		}

		size
	}

	pub fn is_identity(&self) -> bool {
		for (i, row) in self.rows.iter().enumerate() {
			if *row != Bits::with_one(i) {
				return false;
			}
		}

		true
	}

	/// removes rows that are not needed
	pub fn trim(&mut self) {
		for i in (0..self.rows.len()).rev() {
			if self.rows.get(i).unwrap() == &Bits::with_one(i) {
				self.rows.pop().unwrap();
			} else {
				return;
			}
		}
	}
}

impl std::fmt::Display for ParityMatrix {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let n = self.size();

		if n == 0 {
			return writeln!(f, "Empty ParityMatrix");
		}

		for row in self.rows.iter().take(n) {
			let string = (0..n)
				.map(|i| if row.get(i) { "1 " } else { "0 " })
				.collect::<String>();
			writeln!(f, "{}", string.trim())?;
		}

		Ok(())
	}
}

#[cfg(test)]
mod test {
	use crate::{CNot, ParityMatrix};

	#[test]
	fn manual_testing() {
		let answer = vec![
			CNot::new(4, 3), // Leftmost
			CNot::new(1, 0),
			CNot::new(3, 1),
			CNot::new(5, 2),
			CNot::new(4, 2),
			CNot::new(4, 3),
			CNot::new(5, 4),
			CNot::new(2, 3), // Start of dashed box
			CNot::new(3, 2),
			CNot::new(3, 5),
			CNot::new(2, 4),
			CNot::new(1, 2),
			CNot::new(0, 1),
			CNot::new(0, 4),
			CNot::new(0, 3),
		];

		let mut partiy_matrix = ParityMatrix::default();
		for cnot in answer.iter() {
			partiy_matrix.add_row(cnot.control, cnot.target);
		}
		println!("{partiy_matrix}");
	}
}
