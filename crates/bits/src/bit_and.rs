use std::ops::{BitAnd, BitAndAssign};

use crate::Bits;

impl BitAndAssign for Bits {
	fn bitand_assign(&mut self, rhs: Self) {
		let mut this = self.bits.iter_mut();
		let mut that = rhs.bits.into_iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => *this &= that,
				(Some(this), None) => {
					*this = 0;
				}
				(None, Some(_)) | (None, None) => {
					break;
				}
			}
		}
	}
}

impl BitAndAssign<&Self> for Bits {
	fn bitand_assign(&mut self, rhs: &Self) {
		let mut this = self.bits.iter_mut();
		let mut that = rhs.bits.iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => *this &= that,
				(Some(this), None) => {
					*this = 0;
				}
				(None, Some(_)) | (None, None) => {
					break;
				}
			}
		}
	}
}

impl BitAnd<Bits> for Bits {
	type Output = Bits;

	fn bitand(self, rhs: Self) -> Self::Output {
		// The & operation means that we only need to iterate over the shorter iterator.
		Bits {
			bits: self
				.bits
				.into_iter()
				.zip(rhs.bits)
				.map(|(bits, rhs)| bits & rhs)
				.collect(),
		}
	}
}

impl BitAnd<&Bits> for Bits {
	type Output = Bits;

	fn bitand(self, rhs: &Self) -> Self::Output {
		// The & operation means that we only need to iterate over the shorter iterator.
		Bits {
			bits: self
				.bits
				.into_iter()
				.zip(rhs.bits.iter())
				.map(|(bits, rhs)| bits & rhs)
				.collect(),
		}
	}
}

impl BitAnd<Bits> for &Bits {
	type Output = Bits;

	fn bitand(self, rhs: Bits) -> Self::Output {
		// The & operation means that we only need to iterate over the shorter iterator.
		Bits {
			bits: self
				.bits
				.iter()
				.zip(rhs.bits)
				.map(|(bits, rhs)| bits & rhs)
				.collect(),
		}
	}
}

impl BitAnd<&Bits> for &Bits {
	type Output = Bits;

	fn bitand(self, rhs: &Bits) -> Self::Output {
		// The & operation means that we only need to iterate over the shorter iterator.
		Bits {
			bits: self
				.bits
				.iter()
				.zip(rhs.bits.iter())
				.map(|(bits, rhs)| bits & rhs)
				.collect(),
		}
	}
}
