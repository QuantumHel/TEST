use core::{OperatorEq, TargetQubits};
use misc::IterEnum2;
use std::iter;

use circuit::{
	RandomGate,
	gates::{CNot, H, S, V},
};
use pauli::Clifford;
use rand::RngExt;

pub enum CNotHSV {
	CNot(CNot),
	H(H),
	S(S),
	V(V),
}

impl From<CNot> for CNotHSV {
	fn from(value: CNot) -> Self {
		Self::CNot(value)
	}
}

impl From<H> for CNotHSV {
	fn from(value: H) -> Self {
		Self::H(value)
	}
}

impl From<S> for CNotHSV {
	fn from(value: S) -> Self {
		Self::S(value)
	}
}

impl From<V> for CNotHSV {
	fn from(value: V) -> Self {
		Self::V(value)
	}
}

impl Clifford for CNotHSV {
	fn conjugate(&self, pauli_string: &pauli::PauliString) -> (bool, pauli::PauliString) {
		match self {
			CNotHSV::CNot(cnot) => cnot.conjugate(pauli_string),
			CNotHSV::H(h) => h.conjugate(pauli_string),
			CNotHSV::S(s) => s.conjugate(pauli_string),
			CNotHSV::V(v) => v.conjugate(pauli_string),
		}
	}
}

impl TargetQubits for CNotHSV {
	fn target_qubits(&self) -> impl Iterator<Item = &usize> {
		match self {
			CNotHSV::CNot(cnot) => IterEnum2::Iter1(cnot.target_qubits()),
			CNotHSV::H(h) => IterEnum2::Iter2(iter::once(&h.target)),
			CNotHSV::S(s) => IterEnum2::Iter2(iter::once(&s.target)),
			CNotHSV::V(v) => IterEnum2::Iter2(iter::once(&v.target)),
		}
	}
}

impl OperatorEq for CNotHSV {
	fn operator_eq(&self, other: &Self) -> bool {
		matches!(
			(self, other),
			(Self::CNot(_), Self::CNot(_))
				| (Self::H(_), Self::H(_))
				| (Self::S(_), Self::S(_))
				| (Self::V(_), Self::V(_))
		)
	}
}

impl RandomGate for CNotHSV {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		match rng.random_range(0..4) {
			0 => Self::CNot(CNot::random(n_qubits, rng)),
			1 => Self::H(H::random(n_qubits, rng)),
			2 => Self::S(S::random(n_qubits, rng)),
			_ => Self::V(V::random(n_qubits, rng)),
		}
	}
}
