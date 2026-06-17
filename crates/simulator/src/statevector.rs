use crate::Simulatable;
use crate::complex::Complex;

use super::target_state_iterator::TargetStateIterator;
use std::fmt;
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

#[derive(Debug, Clone)]
pub struct Statevector<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy> {
	pub values: Vec<Complex<T>>,
	pub n_qubits: usize,
}

impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy + From<i32>> Statevector<T> {
	pub fn new(n_qubits: usize) -> Self {
		let mut values = vec![
			Complex {
				re: T::from(0),
				im: T::from(0)
			};
			2usize.pow(n_qubits as u32)
		];
		values[0].re = T::from(1);

		Self { values, n_qubits }
	}
}

impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy> Statevector<T> {
	pub fn values(&self) -> &Vec<Complex<T>> {
		&self.values
	}

	pub fn into_values(self) -> Vec<Complex<T>> {
		self.values
	}

	pub fn n_qubits(&self) -> usize {
		self.n_qubits
	}

	/// Gives the states for which to apply the gate.
	///
	/// States should be the ones where the target and controls are 1.
	fn target_states(&self, target: usize, controls: &[usize]) -> TargetStateIterator {
		TargetStateIterator::new(
			controls
				.iter()
				.map(|x| 2usize.pow(*x as u32))
				.sum::<usize>()
				+ 2usize.pow(target as u32),
			self.n_qubits,
		)
	}

	/// TODO
	pub fn apply<O: Simulatable<T>>(&mut self, operation: &O) {
		self.raw_apply(
			operation.matrix(),
			operation.target(),
			&operation.controls(),
		);
	}

	/// The gate matrix is give as (a, b, c, d), where (a, b) is the first row and (c, d) the second.
	fn raw_apply(&mut self, gate_matrix: [Complex<T>; 4], target: usize, controls: &[usize]) {
		let [a, b, c, d] = gate_matrix;
		for bigger_idx in self.target_states(target, controls) {
			let smaller_idx = bigger_idx - 2usize.pow(target as u32);
			let stored_value = self[smaller_idx];

			self[smaller_idx] = self[smaller_idx] * a + self[bigger_idx] * b;

			self[bigger_idx] = stored_value * c + self[bigger_idx] * d;
		}
	}
}

impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy> Index<usize>
	for Statevector<T>
{
	type Output = Complex<T>;

	fn index(&self, index: usize) -> &Self::Output {
		&self.values[index]
	}
}

impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy> IndexMut<usize>
	for Statevector<T>
{
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.values[index]
	}
}

impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy + fmt::Display> fmt::Display
	for Statevector<T>
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.values.is_empty() {
			return f.write_str("");
		}
		for (idx, value) in self.values.iter().enumerate() {
			f.write_str(&format!(
				"\n{:0width$b}: {}",
				idx + 1,
				value,
				width = self.n_qubits
			))?;
		}
		Ok(())
	}
}

impl<
	T: Mul<Output = T>
		+ Add<Output = T>
		+ Sub<Output = T>
		+ Copy
		+ Div<Output = T>
		+ Neg<Output = T>
		+ Eq,
> PartialEq for Statevector<T>
{
	fn eq(&self, other: &Self) -> bool {
		if self.n_qubits != other.n_qubits {
			return false;
		}

		let mut global_phase: Option<Complex<T>> = None;

		for (a, b) in self.values.iter().zip(other.values.iter()) {
			if a.abs_squared() != b.abs_squared() {
				return false;
			}

			if -a.re * a.re == a.im * a.im {
				continue;
			}

			match global_phase.as_ref() {
				Some(global_phase) => {
					if *global_phase != *a / *b {
						return false;
					}
				}
				_ => global_phase = Some(*a / *b),
			}
		}

		true
	}
}

impl<
	T: Mul<Output = T>
		+ Add<Output = T>
		+ Sub<Output = T>
		+ Copy
		+ Div<Output = T>
		+ Neg<Output = T>
		+ Eq,
> Eq for Statevector<T>
{
}
