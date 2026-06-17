use core::fmt;
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Complex<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy> {
	pub re: T,
	pub im: T,
}

impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy> Complex<T> {
	pub fn abs_squared(&self) -> T {
		self.re * self.re + self.im * self.im
	}
}

impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy> Mul for Complex<T> {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			re: self.re * rhs.re - self.im * rhs.im,
			im: self.re * rhs.im + self.im * rhs.re,
		}
	}
}

impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy> Add for Complex<T> {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			re: self.re + rhs.re,
			im: self.im + rhs.im,
		}
	}
}

impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy + Div<Output = T>> Div
	for Complex<T>
{
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		Self {
			re: (self.re * rhs.re + self.im * rhs.im) / rhs.abs_squared(),
			im: (self.im * rhs.re - self.re * rhs.im) / rhs.abs_squared(),
		}
	}
}

impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy + fmt::Display> fmt::Display
	for Complex<T>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Re({}) + Im({})", self.re, self.im)
	}
}
