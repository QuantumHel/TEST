use std::ops::BitXorAssign;

use bits::Bits;

use crate::xor_span::{XorSpaceElement, XorSpan};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ControlBit {
	Qubit(usize),
	Hadamard(usize),
}

impl ControlBit {
	fn new(parity: &Parity) -> Option<Self> {
		#[allow(clippy::manual_map)]
		if let Some(i) = parity.qubits.first_one() {
			Some(ControlBit::Qubit(i))
		} else if let Some(i) = parity.hadamards.first_one() {
			Some(ControlBit::Hadamard(i))
		} else {
			None
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Parity {
	pub bit_flip: bool,
	pub qubits: Bits,
	pub hadamards: Bits,
}

impl XorSpaceElement for Parity {
	type ControlBit = ControlBit;

	fn control_bit(&self) -> Option<Self::ControlBit> {
		ControlBit::new(self)
	}
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

	pub fn using_span(&self, xor_span: &XorSpan<Self>) -> Option<Bits> {
		xor_span.span_element(self)
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
