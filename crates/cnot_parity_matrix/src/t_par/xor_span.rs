use std::collections::BTreeMap;

use bits::Bits;

use super::{parity::Parity, state::State};

#[derive(Debug)]
struct Row {
	/// This contains the Parity that this row represents
	in_real_space: Parity,
	/// This shows to what the row maps in the news space
	in_target_space: Bits,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum ControlBit {
	Qubit(usize),
	Hadamard(usize),
}

impl ControlBit {
	fn new(parity: &Parity) -> Option<Self> {
		#[allow(clippy::manual_map)]
		if let Some(i) = parity.qubits.first_one() {
			Some(ControlBit::Qubit(i))
		} else if let Some(i) = parity.hadamards.first_one() {
			Some(ControlBit::Hadamard(i))
		} else {
			None
		}
	}
}

pub struct XorSpan {
	rows: BTreeMap<ControlBit, Row>,
}

impl XorSpan {
	pub fn new(state: &State) -> Self {
		let mut rows: BTreeMap<ControlBit, Row> = BTreeMap::default();

		// Basically gaussian elminitaion
		'outer: for (i, parity) in state.parities().iter().enumerate() {
			let mut in_real_space = parity.clone();
			let mut in_target_space = Bits::with_one(i);

			loop {
				let Some(control) = ControlBit::new(&in_real_space) else {
					continue 'outer;
				};

				let Some(to_add) = rows.get(&control) else {
					break;
				};
				in_real_space ^= &to_add.in_real_space;
				in_target_space ^= &to_add.in_target_space;
			}

			let Some(control) = ControlBit::new(&in_real_space) else {
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

	pub fn span_parity(&self, parity: &Parity) -> Option<Bits> {
		let mut in_real_space = parity.clone();
		let mut in_target_space = Bits::default();

		loop {
			let Some(control) = ControlBit::new(&in_real_space) else {
				return Some(in_target_space);
			};
			let to_add = self.rows.get(&control)?;
			in_real_space ^= &to_add.in_real_space;
			in_target_space ^= &to_add.in_target_space;
		}
	}

	pub fn supports(&self, parity: &Parity) -> bool {
		self.span_parity(parity).is_some()
	}
}
