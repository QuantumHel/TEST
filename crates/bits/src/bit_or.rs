use std::ops::{BitOr, BitOrAssign};

use crate::Bits;

impl BitOrAssign for Bits {
	fn bitor_assign(&mut self, mut rhs: Self) {
		self.bits
			.iter_mut()
			.zip(rhs.bits.iter())
			.for_each(|(this, that)| *this |= that);

		self.bits.extend(&mut rhs.bits.drain(self.bits.len()..));
	}
}

impl BitOrAssign<&Self> for Bits {
	fn bitor_assign(&mut self, rhs: &Self) {
		self.bits
			.iter_mut()
			.zip(rhs.bits.iter())
			.for_each(|(this, that)| *this |= that);

		self.bits.extend_from_slice(&rhs.bits[self.bits.len()..]);
	}
}

impl BitOr<Bits> for Bits {
	type Output = Bits;

	fn bitor(self, rhs: Self) -> Self::Output {
		let mut result = Bits {
			bits: Vec::with_capacity(self.bits.len().max(rhs.bits.len())),
		};

		let mut this = self.bits.into_iter();
		let mut that = rhs.bits.into_iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => result.bits.push(this | that),
				(Some(this), None) => result.bits.push(this),
				(None, Some(that)) => result.bits.push(that),
				(None, None) => break,
			}
		}

		result
	}
}

impl BitOr<&Bits> for Bits {
	type Output = Bits;

	fn bitor(self, rhs: &Self) -> Self::Output {
		let mut result = Bits {
			bits: Vec::with_capacity(self.bits.len().max(rhs.bits.len())),
		};

		let mut this = self.bits.into_iter();
		let mut that = rhs.bits.iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => result.bits.push(this | that),
				(Some(this), None) => result.bits.push(this),
				(None, Some(that)) => result.bits.push(*that),
				(None, None) => break,
			}
		}

		result
	}
}

impl BitOr<Bits> for &Bits {
	type Output = Bits;

	fn bitor(self, rhs: Bits) -> Self::Output {
		let mut result = Bits {
			bits: Vec::with_capacity(self.bits.len().max(rhs.bits.len())),
		};

		let mut this = self.bits.iter();
		let mut that = rhs.bits.into_iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => result.bits.push(this | that),
				(Some(this), None) => result.bits.push(*this),
				(None, Some(that)) => result.bits.push(that),
				(None, None) => break,
			}
		}

		result
	}
}

impl BitOr<&Bits> for &Bits {
	type Output = Bits;

	fn bitor(self, rhs: &Bits) -> Self::Output {
		let mut result = Bits {
			bits: Vec::with_capacity(self.bits.len().max(rhs.bits.len())),
		};

		let mut this = self.bits.iter();
		let mut that = rhs.bits.iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => result.bits.push(this | that),
				(Some(this), None) => result.bits.push(*this),
				(None, Some(that)) => result.bits.push(*that),
				(None, None) => break,
			}
		}

		result
	}
}
