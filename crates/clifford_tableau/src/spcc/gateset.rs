use std::{
	array::IntoIter,
	iter::{self, Once},
};

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

enum CNotHSVTargetIterator<'a> {
	One(Once<&'a usize>),
	Two(IntoIter<&'a usize, 2>),
}

impl<'a> Iterator for CNotHSVTargetIterator<'a> {
	type Item = &'a usize;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::One(iter) => iter.next(),
			Self::Two(iter) => iter.next(),
		}
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

	fn interacting_qubits(&self) -> impl Iterator<Item = &usize> {
		match self {
			CNotHSV::CNot(cnot) => {
				CNotHSVTargetIterator::Two([cnot.control(), cnot.target()].into_iter())
			}
			CNotHSV::H(h) => CNotHSVTargetIterator::One(iter::once(&h.target)),
			CNotHSV::S(s) => CNotHSVTargetIterator::One(iter::once(&s.target)),
			CNotHSV::V(v) => CNotHSVTargetIterator::One(iter::once(&v.target)),
		}
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
