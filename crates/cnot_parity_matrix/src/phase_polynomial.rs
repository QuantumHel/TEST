use std::collections::BTreeMap;

use bits::Bits;

use crate::{Angle, CNotRz, ParityMatrix, Rz};

pub struct SumOverPathsTerm {
	pub coefficient: f32,
	pub parity_function: Bits,
}

pub type SumOverPaths = BTreeMap<Bits, f32>;

pub struct PhasePolynomial {
	pub sum_over_paths: BTreeMap<Bits, f32>,
	pub basis_transform: ParityMatrix,
}

impl PhasePolynomial {
	pub fn new(program: &Vec<CNotRz>) -> Self {
		let mut basis_transform = ParityMatrix::default();
		let mut sum_over_paths: BTreeMap<Bits, f32> = BTreeMap::new();
		for operation in program {
			match operation {
				CNotRz::CNot(cnot) => {
					basis_transform.insert_cnot(*cnot);
				}
				CNotRz::Rz(Rz { angle, target }) => {
					let parity_function = basis_transform.get_row(*target);
					// Placeholder
					let Angle::Free(angle) = angle else {
						unreachable!()
					};
					*sum_over_paths.entry(parity_function).or_default() += *angle;
				}
			}
		}

		PhasePolynomial {
			sum_over_paths,
			basis_transform,
		}
	}
}
