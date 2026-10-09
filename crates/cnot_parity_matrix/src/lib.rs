mod gray_star_synth;
mod parity_matrix;
mod patel_markov_hayes;
mod perm_row_col;
mod rowcol;
mod t_par;
mod xor_span;

pub use parity_matrix::ParityMatrix;

pub mod algorithm {
	pub use super::gray_star_synth::GrayStarSynth;
	pub use super::patel_markov_hayes::PatelMarkovHayes;
	pub use super::perm_row_col::PermRowCol;
	pub use super::rowcol::RowCol;
	pub use super::t_par::TPar;
}
