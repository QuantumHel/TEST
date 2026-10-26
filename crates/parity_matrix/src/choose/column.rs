use core::connectivity::{Connectivity, TwoQubitEdge};

use crate::ParityMatrix;

pub trait ChooseColumn {
	fn choose_column(
		&self,
		selected_row: usize,
		options: &[usize],
		parity_matrix: &ParityMatrix,
		connectivity: &Connectivity<TwoQubitEdge>,
	) -> usize;
}

/// An implementation of the PermRowCol choose column algorithm from
/// https://arxiv.org/abs/2205.00724v4
pub struct OgPermRowColChooseColumn;

impl ChooseColumn for OgPermRowColChooseColumn {
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
