mod gray_star_synth;
mod parity_matrix;
mod patel_markov_hayes;
mod rowcol;
mod t_par;
mod xor_span;

pub use parity_matrix::ParityMatrix;
use test_core::connectivity::Edge;

pub mod algorithm {
	pub use super::gray_star_synth::GrayStarSynth;
	pub use super::patel_markov_hayes::PatelMarkovHayes;
	pub use super::rowcol::RowCol;
	pub use super::t_par::TPar;
}

// This needs to be moved elsewehre
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
