use std::ops::{BitXor, BitXorAssign};

use crate::Bits;

impl BitXorAssign for Bits {
	fn bitxor_assign(&mut self, mut rhs: Self) {
		self.bits
			.iter_mut()
			.zip(rhs.bits.iter())
			.for_each(|(this, that)| *this ^= that);

		self.bits.extend(&mut rhs.bits.drain(self.bits.len()..));
	}
}

impl BitXorAssign<&Self> for Bits {
	fn bitxor_assign(&mut self, rhs: &Self) {
		self.bits
			.iter_mut()
			.zip(rhs.bits.iter())
			.for_each(|(this, that)| *this ^= that);

		self.bits.extend_from_slice(&rhs.bits[self.bits.len()..]);
	}
}

impl BitXor<Bits> for Bits {
	type Output = Bits;

	fn bitxor(self, rhs: Self) -> Self::Output {
		let mut result = Bits {
			bits: Vec::with_capacity(self.bits.len().max(rhs.bits.len())),
		};

		let mut this = self.bits.into_iter();
		let mut that = rhs.bits.into_iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => result.bits.push(this ^ that),
				(Some(this), None) => result.bits.push(this),
				(None, Some(that)) => result.bits.push(that),
				(None, None) => break,
			}
		}

		result
	}
}

impl BitXor<&Bits> for Bits {
	type Output = Bits;

	fn bitxor(self, rhs: &Self) -> Self::Output {
		let mut result = Bits {
			bits: Vec::with_capacity(self.bits.len().max(rhs.bits.len())),
		};

		let mut this = self.bits.into_iter();
		let mut that = rhs.bits.iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => result.bits.push(this ^ that),
				(Some(this), None) => result.bits.push(this),
				(None, Some(that)) => result.bits.push(*that),
				(None, None) => break,
			}
		}

		result
	}
}

impl BitXor<Bits> for &Bits {
	type Output = Bits;

	fn bitxor(self, rhs: Bits) -> Self::Output {
		let mut result = Bits {
			bits: Vec::with_capacity(self.bits.len().max(rhs.bits.len())),
		};

		let mut this = self.bits.iter();
		let mut that = rhs.bits.into_iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => result.bits.push(this ^ that),
				(Some(this), None) => result.bits.push(*this),
				(None, Some(that)) => result.bits.push(that),
				(None, None) => break,
			}
		}

		result
	}
}

impl BitXor<&Bits> for &Bits {
	type Output = Bits;

	fn bitxor(self, rhs: &Bits) -> Self::Output {
		let mut result = Bits {
			bits: Vec::with_capacity(self.bits.len().max(rhs.bits.len())),
		};

		let mut this = self.bits.iter();
		let mut that = rhs.bits.iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => result.bits.push(this ^ that),
				(Some(this), None) => result.bits.push(*this),
				(None, Some(that)) => result.bits.push(*that),
				(None, None) => break,
			}
		}

		result
	}
}
