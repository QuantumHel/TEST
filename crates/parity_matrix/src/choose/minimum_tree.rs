use core::connectivity::{Connectivity, TwoQubitEdge};

use graph::GraphExt;

use crate::ParityMatrix;

use super::{ChooseColumn, ChooseQubit};

#[derive(Debug, Default)]
pub struct MinimumTree {
	/// If this is true [ChooseColumn] only chooses from columns that have a 1
	/// at the row location.
	pub select_pivot_only: bool,
}

impl ChooseQubit for MinimumTree {
	fn choose_qubit(
		&self,
		options: &[usize],
		parity_matrix: &ParityMatrix,
		connectivity: &Connectivity<TwoQubitEdge>,
	) -> usize {
		options
			.iter()
			.map(|&i| {
				let terminals: Vec<_> = parity_matrix.get_row(i).iter_ones().collect();
				let tree_size = connectivity.steiner_tree(&terminals).node_count();

				(i, tree_size)
			})
			.min_by(|a, b| a.1.cmp(&b.1))
			.unwrap()
			.0
	}
}

impl ChooseColumn for MinimumTree {
	fn choose_column(
		&self,
		selected_row: usize,
		options: &[usize],
		parity_matrix: &ParityMatrix,
		connectivity: &Connectivity<TwoQubitEdge>,
	) -> usize {
		let parity_matrix = parity_matrix.transpose();

		options
			.iter()
			.filter_map(|&i| {
				let row = parity_matrix.get_row(i);

				if self.select_pivot_only && !row.get(selected_row) {
					return None;
				}

				let mut terminals: Vec<_> = row.iter_ones().collect();
				terminals.push(selected_row);
				let tree_size = connectivity.steiner_tree(&terminals).node_count();

				Some((i, tree_size))
			})
			.min_by(|a, b| a.1.cmp(&b.1))
			.unwrap()
			.0
	}
}
