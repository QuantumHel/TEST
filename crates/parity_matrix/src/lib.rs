pub mod choose;
pub mod decompose;
mod gray_star_synth;
mod parity_matrix;
mod t_par;
mod xor_span;

pub use parity_matrix::ParityMatrix;

// These need to be moved to own crate
pub use gray_star_synth::GrayStarSynth;
pub use t_par::TPar;
