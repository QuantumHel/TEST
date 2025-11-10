use bits::{Bits, IterOnes};

use crate::{
	connectivity::{RoutingInstruction, RoutingInstructionTarget},
	misc::NonZeroEvenUsize,
};

use super::PauliLetter;

pub struct LetterIterator<'a> {
	x: IterOnes<'a>,
	z: IterOnes<'a>,
	next_x: Option<usize>,
	next_z: Option<usize>,
}

impl Iterator for LetterIterator<'_> {
	type Item = (usize, PauliLetter);

	fn next(&mut self) -> Option<Self::Item> {
		if self.next_x.is_none() {
			self.next_x = self.x.next()
		}

		if self.next_z.is_none() {
			self.next_z = self.z.next()
		}

		match (self.next_x, self.next_z) {
			(Some(x), Some(z)) => {
				if x == z {
					self.next_x = None;
					self.next_z = None;
					Some((x, PauliLetter::Y))
				} else if x < z {
					self.next_x = None;
					Some((x, PauliLetter::X))
				} else {
					self.next_z = None;
					Some((z, PauliLetter::Z))
				}
			}
			(Some(x), None) => {
				self.next_x = None;
				Some((x, PauliLetter::X))
			}
			(None, Some(z)) => {
				self.next_z = None;
				Some((z, PauliLetter::Z))
			}
			(None, None) => None,
		}
	}
}

/// An collection of [PauliLetter]s in qubit order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PauliString {
	x: Bits,
	z: Bits,
}

impl Default for PauliString {
	fn default() -> Self {
		Self {
			x: Bits::new(),
			z: Bits::new(),
		}
	}
}

impl PauliString {
	pub fn size(&self) -> usize {
		self.start_of_trailin_is()
	}

	pub fn id() -> Self {
		Self::default()
	}

	pub fn id_with_capacity(capacity: usize) -> Self {
		Self {
			x: Bits::with_capacity(capacity),
			z: Bits::with_capacity(capacity),
		}
	}

	pub fn x(index: usize) -> Self {
		let mut x = Bits::with_capacity(index + 1);
		x.set(index, true);
		Self {
			x,
			z: Bits::with_capacity(index + 1),
		}
	}

	pub fn x_with_capacity(index: usize, capacity: usize) -> Self {
		let capacity = capacity.max(index + 1);
		let mut x = Bits::with_capacity(capacity);
		x.set(index, true);
		Self {
			x,
			z: Bits::with_capacity(capacity),
		}
	}

	pub fn z(index: usize) -> Self {
		let mut z = Bits::with_capacity(index + 1);
		z.set(index, true);
		Self {
			x: Bits::with_capacity(index + 1),
			z,
		}
	}

	pub fn z_with_capacity(index: usize, capacity: usize) -> Self {
		let capacity = capacity.max(index + 1);
		let mut z = Bits::with_capacity(capacity);
		z.set(index, true);
		Self {
			x: Bits::with_capacity(capacity),
			z,
		}
	}

	pub fn y(index: usize) -> Self {
		let mut z = Bits::with_capacity(index + 1);
		z.set(index, true);
		Self { x: z.clone(), z }
	}

	pub fn y_with_capacity(index: usize, capacity: usize) -> Self {
		let capacity = capacity.max(index + 1);
		let mut z = Bits::with_capacity(capacity);
		z.set(index, true);
		Self { x: z.clone(), z }
	}

	/// The index after which there are only I letters
	pub fn start_of_trailin_is(&self) -> usize {
		let x = self.x.last_one().map(|v| v + 1).unwrap_or_default();
		let z = self.z.last_one().map(|v| v + 1).unwrap_or_default();
		x.max(z)
	}

	pub fn set(&mut self, index: usize, letter: PauliLetter) {
		match letter {
			PauliLetter::I => {
				self.x.set(index, false);
				self.z.set(index, false);
			}
			PauliLetter::X => {
				self.x.set(index, true);
				self.z.set(index, false);
			}
			PauliLetter::Z => {
				self.x.set(index, false);
				self.z.set(index, true);
			}
			PauliLetter::Y => {
				self.x.set(index, true);
				self.z.set(index, true);
			}
		}
	}

	pub fn get(&self, index: usize) -> PauliLetter {
		match (self.x.get(index), self.z.get(index)) {
			(true, false) => PauliLetter::X,
			(false, true) => PauliLetter::Z,
			(true, true) => PauliLetter::Y,
			_ => PauliLetter::I,
		}
	}

	pub fn targets(&self) -> Vec<usize> {
		(self.x.clone() | &self.z).iter_ones().collect()
	}

	pub fn letters(&self) -> LetterIterator<'_> {
		LetterIterator {
			x: self.x.iter_ones(),
			z: self.z.iter_ones(),
			next_x: None,
			next_z: None,
		}
	}

	pub fn commutes_with(&self, other: &Self) -> bool {
		let x_diff = &self.x ^ &other.x;
		let z_diff = &self.z ^ &other.z;

		let non_i_self = &self.x | &self.z;
		let non_i_other = &other.x | &other.z;
		let anti_comm = non_i_self & non_i_other & (x_diff | z_diff);
		anti_comm.count_ones().is_multiple_of(2)
	}

	pub fn anticommutes_with(&self, other: &Self) -> bool {
		!self.commutes_with(other)
	}

	/// Given the parameters:
	///
	/// s = if neg {-1} else {1}
	///
	/// P = `self`
	///
	/// This function transforms `self` to
	///
	/// $$ e^{si\frac{\pi}{4}O}Pe^{-si\frac{\pi}{4}O} $$
	///
	/// and returs true if the sign changes.
	///
	/// In practice this means that we update `self` to $siOP$ when $P$ and $O$ anticommute.
	///
	/// # Proof:
	/// As shown in for example in
	/// [Picturing Quantum Software Chapter 7](https://github.com/zxcalc/book)
	/// we have
	///
	/// $$
	/// e^{\pm i\frac{\pi}{4}o}
	/// =\text{cos}(\frac{\pi}{4})I\pm\text{sin}(\frac{\pi}{4})O
	/// $$
	///
	/// meaning that
	/// $$
	/// e^{\pm i\frac{\pi}{4}O}
	/// =\text{cos}(\frac{\pi}{4})I\pm\text{sin}(\frac{\pi}{4})O
	/// =\frac{1}{\sqrt2}I\pm i\frac{1}{\sqrt2}O
	/// =\frac{1}{\sqrt2}(I\pm iO).
	/// $$
	///
	/// With this we can write the new `self` as
	///
	/// $$
	/// \frac{1}{\sqrt2}(I\pm iO)P\frac{1}{\sqrt2}(I\mp iO)
	/// =\frac{1}{2}(P\pm iOP\mp iPO+OPO)
	/// $$
	///
	/// where $\pm$ and $\mp$ depend on the given neg (upper choice if false).
	///
	/// Next we will go individually trough the cases where $O$ and $P$ commute and anticommute.
	/// Because $O$ and $P$ are Pauli strings they have to either commute or anticommute which means
	/// that we cover all possible cases.
	///
	/// **When $O$ and $P$ commute** we have $OP=PO$ allowing us to simplify as follows:  
	///
	/// $$
	/// \frac{1}{2}(P\pm iOP\mp iPO+OPO)
	/// =\frac{1}{2}(P\pm iOP\mp iOP+O^2P)
	/// =\frac{1}{2}(P+P)
	/// =P
	/// $$
	///
	/// **When $O$ and $P$ anticommute** we have $OP=-PO$ and we can simplify as follows:
	///
	/// $$
	/// \frac{1}{2}(P\pm iOP\mp iPO+OPO)
	/// =\frac{1}{2}(P\pm iOP\pm iOP-O^2P)
	/// =\frac{1}{2}(P\pm iOP\pm iOP-P)
	/// =\frac{1}{2}(\pm 2iOP)
	/// =\pm iOP
	/// $$
	pub fn pi_over_4_sandwitch(&mut self, neg: bool, #[allow(non_snake_case)] O: &Self) -> bool {
		let new_x = &O.x ^ &self.x;
		let new_z = &O.z ^ &self.z;

		let non_i_self = &self.x | &self.z;
		let non_i_other = &O.x | &O.z;
		let anti_comm = non_i_self & non_i_other & (&new_x | &new_z);
		let n_anti_comm = anti_comm.count_ones();

		// Commutes
		if n_anti_comm.is_multiple_of(2) {
			return false;
		}

		// Sign change fron n_changes and extra i
		let mut sign = match (n_anti_comm + 1) % 4 {
			0 => neg,
			2 => !neg,
			_ => unreachable!(),
		};

		// The amount of results -iAB for pauli matrices A and B.
		//let minuses = (O.z.clone() | self.z.clone())
		//	& !(O.z.clone() ^ self.x.clone())
		//	& !(self.z.clone() ^ new_x.clone());
		let minuses = (&O.x & &self.x).and_not(&(&O.x & &self.z));
		if minuses.count_ones() % 2 == 1 {
			sign = !sign;
		}

		self.x = new_x;
		self.z = new_z;

		sign
	}

	pub fn len(&self) -> usize {
		(&self.x | &self.z).count_ones()
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn as_string(&self) -> String {
		let last = self
			.x
			.last_one()
			.unwrap_or_default()
			.max(self.z.last_one().unwrap_or_default());

		(0..=last)
			.map(|i| match (self.x.get(i), self.z.get(i)) {
				(true, false) => 'X',
				(false, true) => 'Z',
				(true, true) => 'Y',
				(false, false) => 'I',
			})
			.collect()
	}

	/// Private function ised for other len calculations
	fn inner_steps_to_len_one(len: usize, n: usize) -> usize {
		if len == 1 {
			return 0;
		}
		if len < n {
			return if len.is_multiple_of(2) { 3 } else { 2 };
		}

		let len_over = (len - n) as f64;
		let mut k = (len_over / (n - 1) as f64).ceil() as usize;

		// Make sure that k is even if and only if len is
		if k % 2 != len % 2 {
			k += 1
		}

		k + 1
	}

	/// Gives the amount of steps needed to convert the self to lenght 1 by
	/// using gates of a specific size.
	///
	/// When len is at least that of the gate size, the return value is k + 1,
	/// where k is the smallest value for witch the length of exp is smaller or
	/// equal to n + k(n-1) where k is even if len exp is, and uneven if len exp
	/// is uneven
	pub fn steps_to_len_one(&self, gate_size: NonZeroEvenUsize) -> usize {
		let len = self.len();
		let n = gate_size.as_value();
		Self::inner_steps_to_len_one(len, n)
	}

	pub fn steps_to_solve_instruction(
		&self,
		gate_size: NonZeroEvenUsize,
		instruction: &RoutingInstruction,
	) -> usize {
		let len = {
			let mut len = 0;
			for qubit in instruction.qubits.iter() {
				if self.get(*qubit) != PauliLetter::I {
					len += 1;
				}
			}
			len
		};
		let n = gate_size.as_value();

		let basic_steps = Self::inner_steps_to_len_one(len, n);

		let target_is_empty = match &instruction.target {
			RoutingInstructionTarget::Any => false,
			RoutingInstructionTarget::Single(target) => self.get(*target) == PauliLetter::I,
			RoutingInstructionTarget::Multiple(targets) => {
				let mut res = true;
				for target in targets {
					if self.get(*target) != PauliLetter::I {
						res = false;
						break;
					}
				}
				res
			}
		};

		let adjustment = if target_is_empty
			&& (len == 1 || self.len() >= n && (self.len() - n).is_multiple_of(n - 1))
		{
			2
		} else {
			0
		};

		basic_steps + adjustment
	}

	pub fn steps_to_solve_instructions(
		&self,
		gate_size: NonZeroEvenUsize,
		instructions: &[RoutingInstruction],
	) -> usize {
		let mut clone = self.clone();
		let mut total = 0;
		for instruction in instructions.iter() {
			total += clone.steps_to_solve_instruction(gate_size, instruction);
			for qubit in instruction.qubits {
				clone.set(*qubit, PauliLetter::I);
			}
			match &instruction.target {
				RoutingInstructionTarget::Single(target) => {
					clone.set(*target, PauliLetter::X);
				}
				RoutingInstructionTarget::Multiple(targets) => {
					clone.set(*targets.first().unwrap(), PauliLetter::X);
				}
				RoutingInstructionTarget::Any => {
					clone.set(*instruction.qubits.first().unwrap(), PauliLetter::X);
				}
			}
		}

		total
	}
}

#[macro_export]
macro_rules! pauli_string {
	($x:literal) => {{
		let mut string = test_transpiler::pauli::PauliString::id_with_capacity($x.len());
		for (i, c) in $x.chars().enumerate() {
			match c {
				'I' | 'i' => string.set(i, test_transpiler::pauli::PauliLetter::I),
				'X' | 'x' => string.set(i, test_transpiler::pauli::PauliLetter::X),
				'Z' | 'z' => string.set(i, test_transpiler::pauli::PauliLetter::Z),
				'Y' | 'y' => string.set(i, test_transpiler::pauli::PauliLetter::Y),
				_ => panic!("{} is not a pauli letter (IXZYixzy)", c),
			}
		}
		string
	}};
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_steps_to_solve_instruction() {
		let qubits = [0, 1, 2, 3, 4, 5];
		let instruction = RoutingInstruction {
			target: RoutingInstructionTarget::Single(0),
			qubits: &qubits,
		};
		let n = NonZeroEvenUsize::new(4).unwrap();

		assert_eq!(
			pauli_string!("IXIIII").steps_to_solve_instruction(n, &instruction),
			2
		);
		assert_eq!(
			pauli_string!("IXXIII").steps_to_solve_instruction(n, &instruction),
			3
		);
		assert_eq!(
			pauli_string!("IXXXXI").steps_to_solve_instruction(n, &instruction),
			3
		);
		assert_eq!(
			pauli_string!("IXXXXX").steps_to_solve_instruction(n, &instruction),
			2
		);
		assert_eq!(
			pauli_string!("XIIIII").steps_to_solve_instruction(n, &instruction),
			0
		);
		assert_eq!(
			pauli_string!("XXIIII").steps_to_solve_instruction(n, &instruction),
			3
		);
		assert_eq!(
			pauli_string!("XXXXII").steps_to_solve_instruction(n, &instruction),
			1
		);
		assert_eq!(
			pauli_string!("XIXXXX").steps_to_solve_instruction(n, &instruction),
			2
		);
	}

	#[test]
	fn singe_qubit_sandwitch() {
		let x = pauli_string!("X");
		let z = pauli_string!("Z");
		let y = pauli_string!("Y");

		// O = X, P = X => X
		let o = x.clone();
		let mut p = x.clone();
		assert!(!p.pi_over_4_sandwitch(false, &o));
		assert_eq!(p, x);

		// O = X, P = Z => Y
		let o = x.clone();
		let mut p = z.clone();
		assert!(!p.pi_over_4_sandwitch(false, &o));
		assert_eq!(p, y);

		// O = X, P = Y => -Z
		let o = x.clone();
		let mut p = y.clone();
		assert!(p.pi_over_4_sandwitch(false, &o));
		assert_eq!(p, z);
	}

	#[test]
	fn commuting_sandwitch() {
		let o = pauli_string!("XYZXII");
		let mut p = pauli_string!("IZXXYI");

		let p_old = p.clone();
		assert!(!p.pi_over_4_sandwitch(false, &o));
		assert_eq!(p, p_old);
	}

	#[test]
	fn anticommuting_pos_sandwitch() {
		let mut o = PauliString::x(0);
		o.set(1, PauliLetter::Y);
		o.set(2, PauliLetter::Z);
		o.set(3, PauliLetter::X);
		o.set(4, PauliLetter::Y);

		let mut p = PauliString::z(2);
		p.set(3, PauliLetter::X);
		p.set(4, PauliLetter::X);
		p.set(5, PauliLetter::Y);

		assert!(!p.pi_over_4_sandwitch(false, &o));

		let mut res = PauliString::x(0);
		res.set(1, PauliLetter::Y);
		res.set(4, PauliLetter::Z);
		res.set(5, PauliLetter::Y);
		assert_eq!(p, res);
	}

	#[test]
	fn anticommuting_neq_sandwitch() {
		let mut o = PauliString::x(0);
		o.set(1, PauliLetter::Z);
		o.set(2, PauliLetter::Y);
		o.set(3, PauliLetter::Y);
		o.set(4, PauliLetter::Z);
		o.set(5, PauliLetter::Y);

		let mut p = PauliString::x(1);
		p.set(2, PauliLetter::Z);
		p.set(3, PauliLetter::Y);
		p.set(5, PauliLetter::X);
		p.set(6, PauliLetter::X);
		p.set(7, PauliLetter::X);

		assert!(p.pi_over_4_sandwitch(false, &o));

		let mut res = PauliString::x(0);
		res.set(1, PauliLetter::Y);
		res.set(2, PauliLetter::X);
		res.set(4, PauliLetter::Z);
		res.set(5, PauliLetter::Z);
		res.set(6, PauliLetter::X);
		res.set(7, PauliLetter::X);
		assert_eq!(p, res);
	}

	#[test]
	fn neq_sign_sandwitch() {
		let mut o = PauliString::x(0);
		o.set(1, PauliLetter::Y);
		o.set(2, PauliLetter::Z);
		o.set(3, PauliLetter::X);
		o.set(4, PauliLetter::X);

		let mut p = PauliString::z(2);
		p.set(3, PauliLetter::X);
		p.set(4, PauliLetter::Y);
		p.set(5, PauliLetter::Y);

		assert!(!p.pi_over_4_sandwitch(true, &o));

		let mut res = PauliString::x(0);
		res.set(1, PauliLetter::Y);
		res.set(4, PauliLetter::Z);
		res.set(5, PauliLetter::Y);
		assert_eq!(p, res);
	}

	#[test]
	fn correct_steps_to_single_qubit() {
		let n = NonZeroEvenUsize::new(4).unwrap();
		assert_eq!(pauli_string!("X").steps_to_len_one(n), 0);
		assert_eq!(pauli_string!("XXXX").steps_to_len_one(n), 1);
		assert_eq!(pauli_string!("XXX").steps_to_len_one(n), 2);
		assert_eq!(pauli_string!("XX").steps_to_len_one(n), 3);
		assert_eq!(pauli_string!("XXXXX").steps_to_len_one(n), 2);
	}
}
