use std::ops::BitXorAssign;

use bits::Bits;

use super::xor_span::XorSpan;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Parity {
	pub bit_flip: bool,
	pub qubits: Bits,
	pub hadamards: Bits,
}

impl Parity {
	pub fn for_qubit(qubit: usize) -> Self {
		Self {
			bit_flip: false,
			qubits: Bits::with_one(qubit),
			hadamards: Bits::default(),
		}
	}

	pub fn for_hadamard(hadamard: usize) -> Self {
		Self {
			bit_flip: false,
			qubits: Bits::default(),
			hadamards: Bits::with_one(hadamard),
		}
	}

	pub fn using_span(&self, xor_span: &XorSpan) -> Option<Bits> {
		xor_span.span_parity(self)
	}

	pub fn not(self) -> Self {
		Self {
			bit_flip: !self.bit_flip,
			qubits: self.qubits,
			hadamards: self.hadamards,
		}
	}
}

impl BitXorAssign<Parity> for Parity {
	fn bitxor_assign(&mut self, rhs: Self) {
		if rhs.bit_flip {
			self.bit_flip = !self.bit_flip;
		}

		self.qubits ^= rhs.qubits;
		self.hadamards ^= rhs.hadamards;
	}
}

impl BitXorAssign<&Parity> for Parity {
	fn bitxor_assign(&mut self, rhs: &Self) {
		if rhs.bit_flip {
			self.bit_flip = !self.bit_flip;
		}

		self.qubits ^= &rhs.qubits;
		self.hadamards ^= &rhs.hadamards;
	}
}
