use core::connectivity::{Connectivity, TwoQubitEdge};

use crate::ParityMatrix;

use super::{ChooseColumn, ChooseQubit};

/// An implementation of the original PermRowCol choices for picking the column
/// and row.
///
/// https://arxiv.org/abs/2205.00724v4
#[derive(Debug, Default)]
pub struct OriginalPermRowColChoice;

impl ChooseQubit for OriginalPermRowColChoice {
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

impl ChooseColumn for OriginalPermRowColChoice {
	fn choose_column(
		&self,
		selected_row: usize,
		options: &[usize],
		parity_matrix: &ParityMatrix,
		_: &Connectivity<TwoQubitEdge>,
	) -> usize {
		let parity_matrix = parity_matrix.transpose();
		// for columns with an 1 in the pivor row, pick the one with the least 1s
		options
			.iter()
			.filter_map(|&i| {
				let row = parity_matrix.get_row(i);
				row.get(selected_row).then(|| (i, row.count_ones()))
			})
			.min_by(|a, b| a.1.cmp(&b.1))
			.unwrap()
			.0
	}
}
