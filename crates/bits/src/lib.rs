//! This crate contains [Bits], a collection of bits that behaves as an "infinite" vector of bits.

use std::{
	fmt::{Binary, Debug},
	ops::{Bound, RangeBounds},
};

mod bit_and;
mod bit_or;
mod bit_xor;

// This should always be usize, but can be changed here for testing purposes.
type BitHolder = usize;
const BITS_PER: usize = BitHolder::BITS as usize;
const TWO: BitHolder = 2;

pub struct IterOnes<'a> {
	bits: &'a Bits,
	group: usize,
	// We keep editing the group. The edited state lives here
	variation: BitHolder,
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
		self.variation &= !TWO.pow(index as u32);
		Some(index + self.group * BITS_PER)
	}
}

/// A collection of bits that behaves as an "infinite" vector of bits.
#[derive(Default, Clone)]
pub struct Bits {
	bits: Vec<BitHolder>,
}

impl Bits {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_capacity(capacity: usize) -> Self {
		let len = capacity.div_ceil(BITS_PER);
		Bits { bits: vec![0; len] }
	}

	/// Creates Bits with true at the given location and false elsewhere.
	pub fn with_one(location: usize) -> Bits {
		let mut bits = Bits::with_capacity(location + 1);
		bits.set(location, true);
		bits
	}

	pub fn get_range<T: RangeBounds<usize>>(&self, range: T) -> Bits {
		let start = match range.start_bound() {
			Bound::Included(start) => *start,
			Bound::Excluded(before) => *before + 1,
			Bound::Unbounded => 0,
		};
		let end = match range.end_bound() {
			Bound::Included(end) => *end,
			Bound::Excluded(limit) => *limit - 1,
			Bound::Unbounded => self.last_one().unwrap_or(0),
		};

		if end < start {
			return Bits::default();
		}

		let start_holder = start / BITS_PER;
		let end_holder = end / BITS_PER;

		// Collect all relevan holders
		let mut bits = Bits::default();
		for i in start_holder..=end_holder {
			if let Some(holder) = self.bits.get(i) {
				bits.bits.push(*holder);
			}
		}

		if bits.bits.is_empty() {
			return bits;
		}

		// Shift bits (including between holders)
		let shift = start % BITS_PER;
		*bits.bits.first_mut().unwrap() >>= shift;
		for i in 1..bits.bits.len() {
			let mut moved_back = *bits.bits.get(i).unwrap();
			moved_back <<= BITS_PER - shift;

			let prevous = bits.bits.get_mut(i - 1).unwrap();
			*prevous += moved_back;

			*bits.bits.get_mut(i).unwrap() >>= shift;
		}

		// remove last holder when pushing last bits out of it
		if shift > end % BITS_PER && bits.bits.len() > end_holder {
			bits.bits.pop();
		}

		// remove last bits
		// We need to use end-1 to change from index to bit count
		let empty_at_end = BITS_PER - (end + 1 - shift) % BITS_PER;
		if empty_at_end != 0 {
			let last = bits.bits.last_mut().unwrap();
			*last <<= empty_at_end;
			*last >>= empty_at_end;
		}

		bits
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
			*group |= TWO.pow(bit_index as u32);
		} else {
			*group &= !TWO.pow(bit_index as u32);
		}
	}

	pub fn get(&self, index: usize) -> bool {
		let group_index = index / BITS_PER;
		let bit_index = index % BITS_PER;
		self.bits
			.get(group_index)
			.map(|group| group & TWO.pow(bit_index as u32) != 0)
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

#[derive(Debug)]
pub struct BitsOutOfRange;

impl TryFrom<Bits> for usize {
	type Error = BitsOutOfRange;

	fn try_from(value: Bits) -> Result<Self, Self::Error> {
		match value.bits.len() {
			0 => Ok(0),
			_ if value.last_one().unwrap_or_default() < BITS_PER => {
				Ok(*value.bits.first().unwrap())
			}
			_ => Err(BitsOutOfRange),
		}
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

#[cfg(test)]
mod test {
	use crate::Bits;

	fn create(bits: &[usize]) -> Bits {
		let mut bits_struct = Bits::default();

		for (i, bit) in bits.iter().enumerate() {
			bits_struct.set(i, *bit != 0);
		}

		bits_struct
	}

	#[test]
	fn first_holder_range_test() {
		let bits = create(&[1, 1, 0, 1, 1]);
		let range = bits.get_range(1..4);
		assert_eq!(range, create(&[1, 0, 1]));
	}

	#[test]
	fn cross_border_range_test() {
		let mut bits = Bits::with_one(62);
		bits.set(65, true);
		let range = bits.get_range(62..66);
		assert_eq!(range, create(&[1, 0, 0, 1]));
	}

	#[test]
	fn partial_over_allocated_range_test() {
		let bits = Bits::with_one(62);
		let range = bits.get_range(62..66);
		assert_eq!(range, create(&[1, 0, 0, 0]));
	}

	#[test]
	fn over_allocated_range_test() {
		let bits = Bits::with_one(62);
		let range = bits.get_range(64..66);
		assert_eq!(range, create(&[0, 0]));
	}

	#[test]
	fn empty_range_test() {
		let bits = Bits::with_one(62);
		assert_eq!(bits.get_range(62..62), Bits::default())
	}

	#[test]
	fn faulty_range_test() {
		let bits = Bits::with_one(100);
		#[allow(clippy::reversed_empty_ranges)]
		let range = bits.get_range(100..30);
		assert_eq!(range, Bits::default())
	}

	#[test]
	fn inclusive_range_test() {
		let bits = create(&[0, 1, 1, 0, 1]);
		let range = bits.get_range(1..=3);
		assert_eq!(range, create(&[1, 1, 0]));
	}

	#[test]
	fn open_end_range_test() {
		let bits = create(&[1, 1, 0, 0, 1]);
		let range = bits.get_range(2..);
		assert_eq!(range, create(&[0, 0, 1]));
	}

	#[test]
	fn open_start_range_test() {
		let bits = create(&[1, 1, 0, 1]);
		let range = bits.get_range(..3);
		assert_eq!(range, create(&[1, 1, 0]));
	}

	#[test]
	fn full_range_test() {
		let bits = create(&[1, 0, 1]);
		let range = bits.get_range(..);
		assert_eq!(range, bits);
	}

	#[test]
	fn bug_when_start_from_0() {
		let bits = create(&[1, 1, 1, 1]);
		let range = bits.get_range(..3);
		assert_eq!(range, create(&[1, 1, 1]));
	}

	#[test]
	fn test_removing_last_holder() {
		let bits = create(&[
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 1, // [63]
			0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0,
			0, 1,
		]);
		let range = bits.get_range(63..66);
		assert_eq!(range, Bits::with_one(0));
	}
}
