use std::ops::Deref;

#[derive(Debug, Clone, Copy)]
pub struct NonZeroEvenUsize {
	value: usize,
}

impl Deref for NonZeroEvenUsize {
	type Target = usize;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl NonZeroEvenUsize {
	pub fn new(value: usize) -> Option<Self> {
		if value == 0 {
			return None;
		}

		match value.is_multiple_of(2) {
			true => Some(Self { value }),
			false => None,
		}
	}

	pub fn as_value(self) -> usize {
		self.value
	}
}

pub mod generic_bounds {
	//! This module is used to force bounds on generic constants. This module
	//! require the use of `#![feature(generic_const_exprs)]`.
	//!
	//! # Example
	//! Asserting that N >= P
	//! ```rust
	//! impl<const N: usize> Connectivity<N> {
	//! 	pub fn something<const P: usize>(string: PauliString<P>)
	//! 	where Assert<{ N >= P}>: IsTrue
	//! 	{
	//! 		todo!()
	//! 	}
	//! }
	//! ```

	pub enum Assert<const C: bool> {}

	pub trait IsTrue {}

	impl IsTrue for Assert<true> {}
}
