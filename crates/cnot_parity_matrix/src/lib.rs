mod gra_star_synth;
mod parity_matrix;
mod patel_markov_hayes;
mod rowcol;
mod t_par;
mod xor_span;

pub use parity_matrix::ParityMatrix;

pub mod algorithm {
	pub use super::gra_star_synth::GrayStar;
	pub use super::patel_markov_hayes::PatelMarkovHayes;
	pub use super::rowcol::RowCol;
	pub use super::t_par::TPar;
}
