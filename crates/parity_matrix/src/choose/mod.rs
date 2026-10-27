mod first;
mod minimum_neighbors;
mod minimum_tree;
mod perm_row_col;

pub use first::First;
pub use minimum_neighbors::MinimumNeighbors;
pub use minimum_tree::MinimumTree;
pub use perm_row_col::OriginalPermRowColChoice;

use crate::ParityMatrix;
use core::connectivity::{Connectivity, TwoQubitEdge};

pub trait ChooseColumn {
	fn choose_column(
		&self,
		selected_row: usize,
		options: &[usize],
		parity_matrix: &ParityMatrix,
		connectivity: &Connectivity<TwoQubitEdge>,
	) -> usize;
}

pub trait ChooseQubit {
	fn choose_qubit(
		&self,
		options: &[usize],
		parity_matrix: &ParityMatrix,
		connectivity: &Connectivity<TwoQubitEdge>,
	) -> usize;
}
