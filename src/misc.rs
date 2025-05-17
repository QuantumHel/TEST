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

		match value % 2 == 0 {
			true => Some(Self { value }),
			false => None,
		}
	}

	pub fn as_value(self) -> usize {
		self.value
	}
}
