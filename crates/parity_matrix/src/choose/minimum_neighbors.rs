use core::connectivity::{Connectivity, TwoQubitEdge};

use crate::ParityMatrix;

use super::{ChooseColumn, ChooseQubit};

#[derive(Debug, Default)]
pub struct MinimumNeighbors {
	/// If this is true [ChooseColumn] only chooses from columns that have a 1
	/// at the row location.
	pub select_pivot_only: bool,
}

impl ChooseQubit for MinimumNeighbors {
	fn choose_qubit(
		&self,
		options: &[usize],
		_: &ParityMatrix,
		connectivity: &Connectivity<TwoQubitEdge>,
	) -> usize {
		options
			.iter()
			.map(|&i| (i, connectivity.neighbors(i).len()))
			.min_by(|a, b| a.1.cmp(&b.1))
			.unwrap()
			.0
	}
}

impl ChooseColumn for MinimumNeighbors {
	fn choose_column(
		&self,
		selected_row: usize,
		options: &[usize],
		parity_matrix: &ParityMatrix,
		connectivity: &Connectivity<TwoQubitEdge>,
	) -> usize {
		options
			.iter()
			.filter_map(|&i| {
				if self.select_pivot_only && !parity_matrix.get_row(selected_row).get(i) {
					return None;
				}
				Some((i, connectivity.neighbors(i).len()))
			})
			.min_by(|a, b| a.1.cmp(&b.1))
			.unwrap()
			.0
	}
}
