use std::{
	num::NonZero,
	ops::{Add, Div, Mul, Neg, Sub},
};

const TWO: NonZero<i32> = NonZero::new(2).unwrap();

/// This structure stores a value where given two real values A and B, the value
/// is A + B/sqrt(2).
///
/// This is used to simulate the gateset that t-par uses without floating point
/// errors.
#[derive(Clone, Copy, Debug)]
pub struct Squirrel {
	pub normal: Rational,
	pub divided_by_sqrt_2: Rational,
}

impl Default for Squirrel {
	fn default() -> Self {
		Self {
			normal: Rational::from(0),
			divided_by_sqrt_2: Rational::from(0),
		}
	}
}

impl From<i32> for Squirrel {
	fn from(value: i32) -> Self {
		Self {
			normal: Rational::from(value),
			divided_by_sqrt_2: Rational::from(0),
		}
	}
}

impl Squirrel {
	pub const fn zero() -> Self {
		Self {
			normal: Rational {
				numerator: 0,
				denominator: NonZero::new(1).unwrap(),
			},
			divided_by_sqrt_2: Rational {
				numerator: 0,
				denominator: NonZero::new(1).unwrap(),
			},
		}
	}

	pub const fn one() -> Self {
		Self {
			normal: Rational {
				numerator: 1,
				denominator: NonZero::new(1).unwrap(),
			},
			divided_by_sqrt_2: Rational {
				numerator: 0,
				denominator: NonZero::new(1).unwrap(),
			},
		}
	}

	pub fn divided_by_sqrt_2() -> Self {
		Self {
			normal: Rational {
				numerator: 0,
				denominator: NonZero::new(1).unwrap(),
			},
			divided_by_sqrt_2: Rational {
				numerator: 1,
				denominator: NonZero::new(1).unwrap(),
			},
		}
	}
}

impl Mul for Squirrel {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			normal: self.normal * rhs.normal
				+ (self.divided_by_sqrt_2 * rhs.divided_by_sqrt_2) / TWO,
			divided_by_sqrt_2: self.normal * rhs.divided_by_sqrt_2
				+ self.divided_by_sqrt_2 * rhs.normal,
		}
	}
}

impl Add for Squirrel {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			normal: self.normal + rhs.normal,
			divided_by_sqrt_2: self.divided_by_sqrt_2 + rhs.divided_by_sqrt_2,
		}
	}
}

impl Sub for Squirrel {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			normal: self.normal - rhs.normal,
			divided_by_sqrt_2: self.divided_by_sqrt_2 - rhs.divided_by_sqrt_2,
		}
	}
}

impl Div for Squirrel {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		let div = rhs.normal * rhs.normal - rhs.divided_by_sqrt_2 * rhs.divided_by_sqrt_2 / TWO;
		Self {
			normal: (self.normal * rhs.normal
				- self.divided_by_sqrt_2 * rhs.divided_by_sqrt_2 / TWO)
				/ div,
			divided_by_sqrt_2: (rhs.normal * self.divided_by_sqrt_2
				- self.normal * rhs.divided_by_sqrt_2)
				/ div,
		}
	}
}

impl Neg for Squirrel {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self {
			normal: -self.normal,
			divided_by_sqrt_2: -self.divided_by_sqrt_2,
		}
	}
}

impl PartialEq for Squirrel {
	fn eq(&self, other: &Self) -> bool {
		self.normal == other.normal && self.divided_by_sqrt_2 == other.divided_by_sqrt_2
	}
}

impl Eq for Squirrel {}

// TODO: function to simplify
#[derive(Clone, Copy, Debug)]
pub struct Rational {
	numerator: i32,
	denominator: NonZero<i32>,
}

impl Rational {
	/// Divides numerator and denominator with gcd
	pub fn simplify(&mut self) {
		let gcd = self.gcd();
		self.numerator /= gcd;
		self.denominator = NonZero::new(self.denominator.get() / gcd).unwrap();
	}

	pub fn gcd(&self) -> i32 {
		let mut a = self.numerator;
		let mut b = self.denominator.get();
		while b != 0 {
			let t = b;
			b = a % b;
			a = t;
		}
		a
	}
}

impl From<i32> for Rational {
	fn from(value: i32) -> Self {
		Self {
			numerator: value,
			denominator: NonZero::new(1).unwrap(),
		}
	}
}

impl Mul for Rational {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let mut res = Self {
			numerator: self.numerator * rhs.numerator,
			denominator: NonZero::new(self.denominator.get() * rhs.denominator.get()).unwrap(),
		};
		res.simplify();
		res
	}
}

impl Add for Rational {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		let mut res = Self {
			numerator: self.numerator * rhs.denominator.get()
				+ rhs.numerator * self.denominator.get(),
			denominator: NonZero::new(self.denominator.get() * rhs.denominator.get()).unwrap(),
		};
		res.simplify();
		res
	}
}

impl Sub for Rational {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		let mut res = Self {
			numerator: self.numerator * rhs.denominator.get()
				- rhs.numerator * self.denominator.get(),
			denominator: NonZero::new(self.denominator.get() * rhs.denominator.get()).unwrap(),
		};
		res.simplify();
		res
	}
}

impl Div<NonZero<i32>> for Rational {
	type Output = Self;

	fn div(self, rhs: NonZero<i32>) -> Self::Output {
		#[allow(clippy::suspicious_arithmetic_impl)]
		let mut res = Self {
			numerator: self.numerator,
			denominator: NonZero::new(self.denominator.get() * rhs.get()).unwrap(),
		};
		res.simplify();
		res
	}
}

impl Div for Rational {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		let mut res = Self {
			numerator: self.numerator * rhs.denominator.get(),
			denominator: NonZero::new(self.denominator.get() * rhs.numerator)
				.expect("Division by 0"),
		};
		res.simplify();
		res
	}
}

impl Neg for Rational {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self {
			numerator: -self.numerator,
			denominator: self.denominator,
		}
	}
}

impl PartialEq for Rational {
	fn eq(&self, other: &Self) -> bool {
		self.numerator * other.denominator.get() - other.numerator * self.denominator.get() == 0
	}
}

impl Eq for Rational {}

mod tests {
	use std::num::NonZero;

	use crate::t_par::squirrel::Rational;

	use super::Squirrel;
	use simulator::{Complex, Statevector};

	#[test]
	fn squirrel_statevector_eq_test() {
		fn rat(n: i32, d: i32) -> super::Rational {
			super::Rational {
				numerator: n,
				denominator: std::num::NonZero::new(d).unwrap(),
			}
		}

		fn sq(rn: i32, rd: i32, sn: i32, sd: i32) -> Squirrel {
			Squirrel {
				normal: rat(rn, rd),
				divided_by_sqrt_2: rat(sn, sd),
			}
		}

		let statevector: Statevector<Squirrel> = Statevector {
			values: vec![
				Complex {
					re: sq(1, -8, 1, 8),
					im: sq(0, 1, 1, 8),
				},
				Complex {
					re: sq(1, -8, 1, -8),
					im: sq(0, 1, 1, -8),
				},
				Complex {
					re: sq(1, -8, 3, 8),
					im: sq(1, -4, 1, -8),
				},
				Complex {
					re: sq(1, -8, 1, -8),
					im: sq(0, 1, 1, -8),
				},
				Complex {
					re: sq(1, 8, 3, -8),
					im: sq(1, 4, 1, 8),
				},
				Complex {
					re: sq(1, -8, 1, -8),
					im: sq(0, 1, 1, -8),
				},
				Complex {
					re: sq(1, 8, 1, -8),
					im: sq(0, 1, 1, -8),
				},
				Complex {
					re: sq(1, -8, 1, -8),
					im: sq(0, 1, 1, -8),
				},
				Complex {
					re: sq(1, -8, 3, -8),
					im: sq(1, -4, 1, 8),
				},
				Complex {
					re: sq(1, 8, 1, -8),
					im: sq(0, 1, 1, -8),
				},
				Complex {
					re: sq(1, -8, 1, -8),
					im: sq(0, 1, 1, -8),
				},
				Complex {
					re: sq(1, 8, 1, -8),
					im: sq(0, 1, 1, -8),
				},
				Complex {
					re: sq(1, 8, 1, 8),
					im: sq(0, 1, 1, 8),
				},
				Complex {
					re: sq(1, 8, 1, -8),
					im: sq(0, 1, 1, -8),
				},
				Complex {
					re: sq(1, 8, 3, 8),
					im: sq(1, 4, 1, -8),
				},
				Complex {
					re: sq(1, 8, 1, -8),
					im: sq(0, 1, 1, -8),
				},
			],
			n_qubits: 4,
		};
		assert_eq!(statevector.clone(), statevector)
	}

	#[test]
	fn squirrel_complex_number_division_test() {
		let a = Complex {
			re: Squirrel {
				normal: Rational {
					numerator: 1,
					denominator: NonZero::new(-8).unwrap(),
				},
				divided_by_sqrt_2: Rational {
					numerator: 1,
					denominator: NonZero::new(8).unwrap(),
				},
			},
			im: Squirrel {
				normal: Rational {
					numerator: 0,
					denominator: NonZero::new(1).unwrap(),
				},
				divided_by_sqrt_2: Rational {
					numerator: 1,
					denominator: NonZero::new(8).unwrap(),
				},
			},
		};
		let b = a;
		assert_eq!(
			Complex {
				re: Squirrel::from(1),
				im: Squirrel::from(0)
			},
			a / b
		);
	}
}
