use core::connectivity::{Connectivity, TwoQubitEdge};

use crate::ParityMatrix;

use super::{ChooseColumn, ChooseQubit};

#[derive(Debug, Default)]
pub struct First {
	/// If this is true [ChooseColumn] only chooses from columns that have a 1
	/// at the row location.
	pub select_pivot_only: bool,
}

impl ChooseQubit for First {
	fn choose_qubit(
		&self,
		options: &[usize],
		_: &ParityMatrix,
		_: &Connectivity<TwoQubitEdge>,
	) -> usize {
		options[0]
	}
}

impl ChooseColumn for First {
	fn choose_column(
		&self,
		selected_row: usize,
		options: &[usize],
		parity_matrix: &ParityMatrix,
		_: &Connectivity<TwoQubitEdge>,
	) -> usize {
		if self.select_pivot_only {
			let parity_matrix = parity_matrix.transpose();
			*options
				.iter()
				.find(|&&i| parity_matrix.get_row(i).get(selected_row))
				.unwrap()
		} else {
			options[0]
		}
	}
}
