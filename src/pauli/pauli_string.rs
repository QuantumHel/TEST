use bitvec::vec::BitVec;

use crate::misc::NonZeroEvenUsize;

use super::PauliLetter;

/// An collection of [PauliLetter]s in qubit order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct PauliString<const N: usize> {
	x: BitVec,
	z: BitVec,
}

impl<const N: usize> Default for PauliString<N> {
	fn default() -> Self {
		let z = BitVec::repeat(false, N);
		Self { x: z.clone(), z }
	}
}

impl<const N: usize> PauliString<N> {
	pub fn id() -> Self {
		Self::default()
	}

	/// # Panics
	/// If index is out of range.
	pub fn x(index: usize) -> Self {
		let mut x = BitVec::repeat(false, N);
		x.set(index, true);
		Self {
			x,
			z: BitVec::repeat(false, N),
		}
	}

	/// # Panics
	/// If index is out of range.
	pub fn z(index: usize) -> Self {
		let mut z = BitVec::repeat(false, N);
		z.set(index, true);
		Self {
			x: BitVec::repeat(false, N),
			z,
		}
	}

	/// # Panics
	/// If index is out of range.
	pub fn y(index: usize) -> Self {
		let mut z = BitVec::repeat(false, N);
		z.set(index, true);
		Self { x: z.clone(), z }
	}

	/// # Panics
	/// If index is out of range.
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
		match (self.x.get(index).as_deref(), self.z.get(index).as_deref()) {
			(Some(true), Some(false)) => PauliLetter::X,
			(Some(false), Some(true)) => PauliLetter::Z,
			(Some(true), Some(true)) => PauliLetter::Y,
			_ => PauliLetter::I,
		}
	}

	pub fn letters(&self) -> Vec<(usize, PauliLetter)> {
		self.x
			.iter()
			.zip(self.z.iter())
			.enumerate()
			.filter(|(_, (x, z))| **x || **z)
			.map(|(i, (x, z))| match (*x, *z) {
				(true, false) => (i, PauliLetter::X),
				(false, true) => (i, PauliLetter::Z),
				(true, true) => (i, PauliLetter::Y),
				_ => {
					unreachable!()
				}
			})
			.collect()
	}

	pub fn commutes_with(&self, other: &Self) -> bool {
		let x_diff = self.x.clone() ^ other.x.clone();
		let z_diff = self.z.clone() ^ other.z.clone();

		let non_i_self = self.x.clone() | self.z.clone();
		let non_i_other = other.x.clone() | other.z.clone();
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
		let new_x = O.x.clone() ^ &self.x;
		let new_z = O.z.clone() ^ &self.z;

		let non_i_self = self.x.clone() | self.z.clone();
		let non_i_other = O.x.clone() | O.z.clone();
		let anti_comm = non_i_self & non_i_other & (new_x.clone() | new_z.clone());
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
		let minuses = (O.z.clone() | self.z.clone())
			& !(O.z.clone() ^ self.x.clone())
			& !(self.z.clone() ^ new_x.clone());
		if minuses.count_ones() % 2 == 1 {
			sign = !sign;
		}

		self.x = new_x;
		self.z = new_z;

		sign
	}

	pub fn len(&self) -> usize {
		(self.x.clone() | self.z.clone()).count_ones()
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn as_string(&self) -> String {
		self.x
			.iter()
			.zip(self.z.iter())
			.map(|(x, z)| match (*x, *z) {
				(true, false) => 'X',
				(false, true) => 'Z',
				(true, true) => 'Y',
				(false, false) => 'I',
			})
			.collect()
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
		if len == 1 {
			return 0;
		}
		if len < n {
			return if len.is_multiple_of(2) { 3 } else { 2 };
		}

		let len_over = (len - n) as f64;
		let mut k = (len_over / (n - 1) as f64).ceil() as usize;

		// Make sure that k is even if and onfly if len is
		if k % 2 != len % 2 {
			k += 1
		}

		k + 1
	}
}

#[macro_export]
macro_rules! pauli_string {
	($x:literal) => {{
		let mut string = PauliString::<{ $x.len() }>::id();
		for (i, c) in $x.chars().enumerate() {
			match c {
				'I' | 'i' => string.set(i, PauliLetter::I),
				'X' | 'x' => string.set(i, PauliLetter::X),
				'Z' | 'z' => string.set(i, PauliLetter::Z),
				'Y' | 'y' => string.set(i, PauliLetter::Y),
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
		let mut o = PauliString::<6>::x(0);
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
		let mut o = PauliString::<8>::x(0);
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
		let mut o = PauliString::<6>::x(0);
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
