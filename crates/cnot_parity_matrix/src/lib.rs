mod gray_star_synth;
mod parity_matrix;
mod patel_markov_hayes;
mod perm_row_col;
mod rowcol;
mod t_par;
mod xor_span;

use core::connectivity::Connectivity;
use graph::{Cardinality, ConstCardinalityEdge, Edge};
pub use parity_matrix::ParityMatrix;

pub mod algorithm {
	pub use super::gray_star_synth::GrayStarSynth;
	pub use super::patel_markov_hayes::PatelMarkovHayes;
	pub use super::perm_row_col::PermRowCol;
	pub use super::rowcol::RowCol;
	pub use super::t_par::TPar;
}

// FIXME: This needs to be moved elsewehre (also tests in rowcol file)
#[derive(Debug)]
pub struct TwoQubitEdge(pub [usize; 2]);

impl Edge for TwoQubitEdge {
	fn nodes(&self) -> Vec<usize> {
		self.0.to_vec()
	}

	fn weight(&self) -> f64 {
		1.0
	}
}

impl ConstCardinalityEdge for TwoQubitEdge {
	type CARDINALITY = Cardinality<2>;
}

impl TwoQubitEdge {
	pub fn square_lattice(min_qubit_count: usize) -> Connectivity<TwoQubitEdge> {
		let n = (min_qubit_count as f64).sqrt().ceil() as usize;

		let mut connectivity: Connectivity<TwoQubitEdge> = Connectivity::new();
		for y in 0..n {
			for x in 0..n {
				let qubit = (y * n) + x;
				// horizontal
				if x < n - 1 {
					connectivity.add_edge(TwoQubitEdge([qubit, qubit + 1]));
				}

				// vertical
				if y < n - 1 {
					connectivity.add_edge(TwoQubitEdge([qubit, qubit + n]));
				}
			}
		}
		connectivity
	}
}
