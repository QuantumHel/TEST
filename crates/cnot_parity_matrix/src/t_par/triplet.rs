use crate::{Angle, CNot, CNotRzXYH, H, Rz, X, Y};
use std::collections::BTreeMap;

use super::{HadamardTransform, parity::Parity, state::State};

pub struct Triplet {
	pub s: BTreeMap<Parity, Angle>,
	pub q: State,
	pub h: Vec<HadamardTransform>,
}

impl Triplet {
	pub fn new(n: usize) -> Self {
		Self {
			s: BTreeMap::default(),
			q: State::new(n),
			h: Vec::new(),
		}
	}

	pub fn add_gate(&mut self, gate: CNotRzXYH) {
		match gate {
			CNotRzXYH::CNot(CNot { control, target }) => {
				self.q.apply_cnot(control, target);
			}
			CNotRzXYH::Rz(Rz { angle, target }) => {
				let parity = self.q.get_cloned(target);
				let current = self.s.entry(parity).or_insert(Angle::QuarterPi(0));
				*current += angle;
				if current.is_zero() {
					self.s.remove(&self.q.get_cloned(target));
				}
			}
			CNotRzXYH::X(X { target }) => {
				self.q.apply_x(target);
			}
			CNotRzXYH::Y(Y { target }) => {
				self.add_gate(CNotRzXYH::X(X { target }));
				self.add_gate(CNotRzXYH::Rz(Rz {
					angle: Angle::QuarterPi(4),
					target,
				}));
			}
			CNotRzXYH::H(H { target }) => {
				let input = self.q.clone();
				*self.q.get_mut(target) = Parity::for_hadamard(self.h.len());
				let output = self.q.clone();
				self.h.push(HadamardTransform {
					target,
					input,
					output,
				});
			}
		}
	}
}
