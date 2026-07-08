use std::{collections::BTreeMap, ops::BitXorAssign};

use bits::Bits;

#[derive(Debug)]
struct Row<T: XorSpaceElement> {
	/// This contains the Parity that this row represents
	in_real_space: T,
	/// This shows to what the row maps in the news space
	in_target_space: Bits,
}

#[derive(Debug)]
pub struct XorSpan<T: XorSpaceElement> {
	rows: BTreeMap<T::ControlBit, Row<T>>,
}

pub trait XorSpaceElement: for<'a> BitXorAssign<&'a Self> + Clone + 'static {
	type ControlBit: Ord;

	fn control_bit(&self) -> Option<Self::ControlBit>;
}

impl XorSpaceElement for Bits {
	type ControlBit = usize;

	fn control_bit(&self) -> Option<Self::ControlBit> {
		self.first_one()
	}
}

impl<T: XorSpaceElement> XorSpan<T> {
	pub fn new(state: &[T]) -> Self {
		let mut rows: BTreeMap<T::ControlBit, Row<T>> = BTreeMap::default();

		// Basically gaussian elminitaion
		'outer: for (i, parity) in state.iter().enumerate() {
			let mut in_real_space = parity.clone();
			let mut in_target_space = Bits::with_one(i);

			loop {
				let Some(control) = in_real_space.control_bit() else {
					continue 'outer;
				};

				let Some(to_add) = rows.get(&control) else {
					break;
				};
				in_real_space ^= &to_add.in_real_space;
				in_target_space ^= &to_add.in_target_space;
			}

			let Some(control) = in_real_space.control_bit() else {
				continue 'outer;
			};

			rows.insert(
				control,
				Row {
					in_real_space,
					in_target_space,
				},
			);
		}

		Self { rows }
	}

	pub fn span_element(&self, element: &T) -> Option<Bits> {
		let mut in_real_space = element.clone();
		let mut in_target_space = Bits::default();

		loop {
			let Some(control) = in_real_space.control_bit() else {
				return Some(in_target_space);
			};
			let to_add = self.rows.get(&control)?;
			in_real_space ^= &to_add.in_real_space;
			in_target_space ^= &to_add.in_target_space;
		}
	}

	pub fn supports(&self, element: &T) -> bool {
		self.span_element(element).is_some()
	}
}
