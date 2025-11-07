use std::fmt::Binary;

mod bit_and;
mod bit_or;
mod bit_xor;

/// A collection of bits that behaves as an "infinite" amounf of bits.
#[derive(Default, Clone)]
pub struct Bits {
	bits: Vec<usize>,
}

const BITS_PER: usize = usize::BITS as usize;

impl Bits {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_capacity(capacity: usize) -> Self {
		let len = capacity.div_ceil(BITS_PER);
		Bits { bits: vec![0, len] }
	}

	pub fn is_all_zero(&self) -> bool {
		for bits in self.bits.iter() {
			if bits != &0 {
				return false;
			}
		}
		true
	}

	pub fn count_ones(&self) -> u32 {
		self.bits.iter().map(|v| v.count_ones()).sum()
	}

	pub fn set(&mut self, index: usize, value: bool) {
		let group_index = index / BITS_PER;
		let bit_index = index % BITS_PER;
		println!("Group {group_index} with index {bit_index}");

		let group = match self.bits.get_mut(group_index) {
			Some(group) => group,
			_ => {
				let n_new = (group_index + 1) - self.bits.len();
				self.bits.append(&mut vec![0; n_new]);

				self.bits.get_mut(group_index).unwrap()
			}
		};

		if value {
			*group |= 2_usize.pow(bit_index as u32);
		} else {
			*group &= !2_usize.pow(bit_index as u32);
		}
	}

	pub fn get(&self, index: usize) -> bool {
		let group_index = index / BITS_PER;
		let bit_index = index % BITS_PER;
		self.bits
			.get(group_index)
			.map(|group| group & 2_usize.pow(bit_index as u32) != 0)
			.unwrap_or(false)
	}
}

impl Binary for Bits {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut string = String::new();
		let mut iterator = self.bits.iter().rev();

		if let Some(bits) = iterator.next() {
			string.push_str(&format!("{bits:b}"));
		}

		for bits in iterator {
			string.push_str(&format!("{bits:0BITS_PER$b}"));
		}

		write!(f, "{string}")
	}
}

impl PartialEq for Bits {
	fn eq(&self, other: &Self) -> bool {
		let mut this = self.bits.iter();
		let mut that = other.bits.iter();
		loop {
			match (this.next(), that.next()) {
				(Some(this), Some(that)) => {
					if this != that {
						return false;
					}
				}
				(Some(this), None) => {
					if *this != 0 {
						return false;
					}
				}
				(None, Some(that)) => {
					if *that != 0 {
						return false;
					}
				}
				(None, None) => break,
			}
		}

		true
	}
}

impl Eq for Bits {}
