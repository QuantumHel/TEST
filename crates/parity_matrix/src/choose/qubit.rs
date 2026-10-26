use core::connectivity::{Connectivity, TwoQubitEdge};

use crate::ParityMatrix;

pub trait ChooseQubit {
	fn choose_qubit(
		&self,
		options: &[usize],
		parity_matrix: &ParityMatrix,
		connectivity: &Connectivity<TwoQubitEdge>,
	) -> usize;
}

/// An implementation of the PermRowCol choose row algorithm from
/// https://arxiv.org/abs/2205.00724v4
pub struct OgPermRowColChooseRow;

impl ChooseQubit for OgPermRowColChooseRow {
	fn choose_qubit(
		&self,
		options: &[usize],
		parity_matrix: &ParityMatrix,
		_: &Connectivity<TwoQubitEdge>,
	) -> usize {
		options
			.iter()
			.map(|&i| (i, parity_matrix.get_row(i).count_ones()))
			.min_by(|a, b| a.1.cmp(&b.1))
			.unwrap()
			.0
	}
}

pub struct FirstQubit;

impl ChooseQubit for FirstQubit {
	fn choose_qubit(
		&self,
		options: &[usize],
		_: &ParityMatrix,
		_: &Connectivity<TwoQubitEdge>,
	) -> usize {
		options[0]
	}
}
