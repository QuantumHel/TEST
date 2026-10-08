use bits::{Bits, IterOnes};

use super::PauliLetter;

struct LetterIterator<'a> {
	x: IterOnes<'a>,
	z: IterOnes<'a>,
	next_x: Option<usize>,
	next_z: Option<usize>,
}

impl Iterator for LetterIterator<'_> {
	type Item = (usize, PauliLetter);

	fn next(&mut self) -> Option<Self::Item> {
		if self.next_x.is_none() {
			self.next_x = self.x.next()
		}

		if self.next_z.is_none() {
			self.next_z = self.z.next()
		}

		match (self.next_x, self.next_z) {
			(Some(x), Some(z)) => {
				if x == z {
					self.next_x = None;
					self.next_z = None;
					Some((x, PauliLetter::Y))
				} else if x < z {
					self.next_x = None;
					Some((x, PauliLetter::X))
				} else {
					self.next_z = None;
					Some((z, PauliLetter::Z))
				}
			}
			(Some(x), None) => {
				self.next_x = None;
				Some((x, PauliLetter::X))
			}
			(None, Some(z)) => {
				self.next_z = None;
				Some((z, PauliLetter::Z))
			}
			(None, None) => None,
		}
	}
}

/// An collection of [PauliLetter]s in qubit order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PauliString {
	x: Bits,
	z: Bits,
}

impl Default for PauliString {
	fn default() -> Self {
		Self {
			x: Bits::new(),
			z: Bits::new(),
		}
	}
}

impl PauliString {
	/// The amount of non identity letters
	#[allow(clippy::len_without_is_empty)]
	pub fn len(&self) -> usize {
		(&self.x | &self.z).count_ones()
	}

	pub fn id() -> Self {
		Self::default()
	}

	pub fn x(index: usize) -> Self {
		let mut x = Bits::with_capacity(index + 1);
		x.set(index, true);
		Self {
			x,
			z: Bits::with_capacity(index + 1),
		}
	}

	pub fn z(index: usize) -> Self {
		let mut z = Bits::with_capacity(index + 1);
		z.set(index, true);
		Self {
			x: Bits::with_capacity(index + 1),
			z,
		}
	}

	pub fn y(index: usize) -> Self {
		let mut z = Bits::with_capacity(index + 1);
		z.set(index, true);
		Self { x: z.clone(), z }
	}

	pub fn set(&mut self, index: usize, letter: PauliLetter) {
		match letter {
			PauliLetter::I => {
				self.x.set(index, false);
				self.z.set(index, false);
			}
			PauliLetter::X => {
				self.x.set(index, true);
				self.z.set(index, false);
			}
			PauliLetter::Z => {
				self.x.set(index, false);
				self.z.set(index, true);
			}
			PauliLetter::Y => {
				self.x.set(index, true);
				self.z.set(index, true);
			}
		}
	}

	pub fn get(&self, index: usize) -> PauliLetter {
		match (self.x.get(index), self.z.get(index)) {
			(true, false) => PauliLetter::X,
			(false, true) => PauliLetter::Z,
			(true, true) => PauliLetter::Y,
			_ => PauliLetter::I,
		}
	}

	pub fn non_trivial_indices(&self) -> Vec<usize> {
		(&self.x | &self.z).iter_ones().collect()
	}

	pub fn enumerate_non_trivial(&self) -> impl Iterator<Item = (usize, PauliLetter)> {
		LetterIterator {
			x: self.x.iter_ones(),
			z: self.z.iter_ones(),
			next_x: None,
			next_z: None,
		}
	}

	pub fn commutes_with(&self, other: &Self) -> bool {
		let x_diff = &self.x ^ &other.x;
		let z_diff = &self.z ^ &other.z;

		let non_i_self = &self.x | &self.z;
		let non_i_other = &other.x | &other.z;
		let anti_comm = non_i_self & non_i_other & (x_diff | z_diff);
		anti_comm.count_ones().is_multiple_of(2)
	}

	pub fn anticommutes_with(&self, other: &Self) -> bool {
		!self.commutes_with(other)
	}

	pub fn is_identity(&self) -> bool {
		self.x.is_all_zero() && self.z.is_all_zero()
	}

	/// returns [None] when all trivial
	pub fn highest_non_trivial_index(&self) -> Option<usize> {
		self.x
			.last_one()
			.map(|x| x.max(self.z.last_one().unwrap_or_default()))
			.or_else(|| self.z.last_one())
	}

	pub fn as_string(&self) -> String {
		let last = self
			.x
			.last_one()
			.unwrap_or_default()
			.max(self.z.last_one().unwrap_or_default());

		(0..=last)
			.map(|i| match (self.x.get(i), self.z.get(i)) {
				(true, false) => 'X',
				(false, true) => 'Z',
				(true, true) => 'Y',
				(false, false) => 'I',
			})
			.collect()
	}
}

#[macro_export]
macro_rules! pauli_string {
	($x:literal) => {{
		let mut string = $crate::PauliString::id_with_capacity($x.len());
		for (i, c) in $x.chars().enumerate() {
			match c {
				'I' | 'i' => string.set(i, $crate::PauliLetter::I),
				'X' | 'x' => string.set(i, $crate::PauliLetter::X),
				'Z' | 'z' => string.set(i, $crate::PauliLetter::Z),
				'Y' | 'y' => string.set(i, $crate::PauliLetter::Y),
				_ => panic!("{} is not a pauli letter (IXZYixzy)", c),
			}
		}
		string
	}};
}
