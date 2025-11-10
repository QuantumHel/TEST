//! This crate contains [Bits], a collection of bits that behaves as an "infinite" vector of bits.

use std::fmt::{Binary, Debug};

mod bit_and;
mod bit_or;
mod bit_xor;

pub struct IterOnes<'a> {
	bits: &'a Bits,
	group: usize,
	// We keep editing the group. The edited state lives here
	variation: usize,
}

impl Iterator for IterOnes<'_> {
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		let index = self.variation.trailing_zeros() as usize;
		if index >= BITS_PER {
			self.group += 1;
			if self.group >= self.bits.bits.len() {
				return None;
			}

			self.variation = self.bits.bits.get(self.group).cloned().unwrap();
			return self.next();
		}
		self.variation &= !2_usize.pow(index as u32);
		Some(index + self.group * BITS_PER)
	}
}

/// A collection of bits that behaves as an "infinite" vector of bits.
#[derive(Default, Clone)]
pub struct Bits {
	bits: Vec<usize>,
}

const BITS_PER: usize = usize::BITS as usize;

impl Bits {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_capacity(capacity: usize) -> Self {
		let len = capacity.div_ceil(BITS_PER);
		Bits { bits: vec![0, len] }
	}

	/// Returns the index of the last '1' bit.
	pub fn last_one(&self) -> Option<usize> {
		for (index, bits) in self.bits.iter().enumerate().rev() {
			let pos = BITS_PER - bits.leading_zeros() as usize;
			if pos > 0 {
				return Some(pos - 1 + index * BITS_PER);
			}
		}
		None
	}

	pub fn iter_ones(&self) -> IterOnes<'_> {
		IterOnes {
			bits: self,
			group: 0,
			variation: self.bits.first().cloned().unwrap_or_default(),
		}
	}

	pub fn is_all_zero(&self) -> bool {
		for bits in self.bits.iter() {
			if bits != &0 {
				return false;
			}
		}
		true
	}

	pub fn count_ones(&self) -> usize {
		self.bits.iter().map(|v| v.count_ones()).sum::<u32>() as usize
	}

	pub fn set(&mut self, index: usize, value: bool) {
		let group_index = index / BITS_PER;
		let bit_index = index % BITS_PER;

		let group = match self.bits.get_mut(group_index) {
			Some(group) => group,
			_ => {
				let n_new = (group_index + 1) - self.bits.len();
				self.bits.append(&mut vec![0; n_new]);

				self.bits.get_mut(group_index).unwrap()
			}
		};

		if value {
			*group |= 2_usize.pow(bit_index as u32);
		} else {
			*group &= !2_usize.pow(bit_index as u32);
		}
	}

	pub fn get(&self, index: usize) -> bool {
		let group_index = index / BITS_PER;
		let bit_index = index % BITS_PER;
		self.bits
			.get(group_index)
			.map(|group| group & 2_usize.pow(bit_index as u32) != 0)
			.unwrap_or(false)
	}

	/// Calculates self & !other
	///
	/// This allows the usage of a ! operator in some cases, even though we cant do a ! operator by itself.
	pub fn and_not(&self, other: &Self) -> Self {
		let mut result = Bits {
			bits: Vec::with_capacity(self.bits.len()),
		};

		let mut this = self.bits.iter();
		let mut that = other.bits.iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => result.bits.push(this & !that),
				(Some(this), None) => result.bits.push(*this),
				(None, Some(_)) | (None, None) => break,
			}
		}

		result
	}
}

impl Binary for Bits {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut string = String::new();
		let mut iterator = self.bits.iter().rev();

		if let Some(bits) = iterator.next() {
			string.push_str(&format!("{bits:b}"));
		}

		for bits in iterator {
			string.push_str(&format!("{bits:0BITS_PER$b}"));
		}

		write!(f, "{string}")
	}
}

impl Debug for Bits {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Bits [")?;
		let chars = format!("{self:b}");
		let mut chars = chars.chars().rev();
		if let Some(first) = chars.next() {
			write!(f, "{first}")?;
		}

		for bit in chars {
			write!(f, ", {bit}")?;
		}

		write!(f, "]")
	}
}

impl PartialEq for Bits {
	fn eq(&self, other: &Self) -> bool {
		let mut this = self.bits.iter();
		let mut that = other.bits.iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => {
					if this != that {
						return false;
					}
				}
				(Some(this), None) => {
					if *this != 0 {
						return false;
					}
				}
				(None, Some(that)) => {
					if *that != 0 {
						return false;
					}
				}
				(None, None) => break,
			}
		}

		true
	}
}

impl Eq for Bits {}
