use std::ops::{Add, Mul, Sub};

mod complex;
mod statevector;
mod target_state_iterator;

pub use complex::Complex;
pub use statevector::Statevector;

pub trait Simulatable<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy> {
	fn matrix(&self) -> [Complex<T>; 4];

	fn target(&self) -> usize;

	fn controls(&self) -> Vec<usize>;
}
