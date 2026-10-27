use core::connectivity::{Connectivity, TwoQubitEdge};

use graph::GraphExt;

use crate::ParityMatrix;

use super::{ChooseColumn, ChooseQubit};

/// Chooces the qubit with the smallest average distance to other qubits.
#[derive(Debug, Default)]
pub struct MaximumDistance {
	/// If this is true [ChooseColumn] only chooses from columns that have a 1
	/// at the row location.
	pub select_pivot_only: bool,
}

impl ChooseQubit for MaximumDistance {
	fn choose_qubit(
		&self,
		options: &[usize],
		_: &ParityMatrix,
		connectivity: &Connectivity<TwoQubitEdge>,
	) -> usize {
		options
			.iter()
			.map(|&i| {
				let distances = connectivity.shortest_paths(i);
				let summ: f64 = distances.into_iter().map(Option::unwrap_or_default).sum();
				(i, summ)
			})
			.max_by(|a, b| a.1.total_cmp(&b.1))
			.unwrap()
			.0
	}
}

impl ChooseColumn for MaximumDistance {
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

				let distances = connectivity.shortest_paths(i);
				let summ: f64 = distances.into_iter().map(Option::unwrap_or_default).sum();

				Some((i, summ))
			})
			.max_by(|a, b| a.1.total_cmp(&b.1))
			.unwrap()
			.0
	}
}
