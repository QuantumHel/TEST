use rand::{
	RngExt,
	distr::{Distribution, StandardUniform},
};

use crate::RandomGate;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rz<T> {
	pub angle: T,
	pub target: usize,
}

#[cfg(feature = "openqasm2")]
impl<T: From<openqasm2::Value>> crate::openqasm2::OpenQasm2Gate for Rz<T> {
	fn cx(_: usize, _: usize) -> Option<Self> {
		None
	}

	fn u(
		theta: openqasm2::Value,
		phi: openqasm2::Value,
		lambda: openqasm2::Value,
		target: usize,
	) -> Option<Self> {
		if openqasm2::Value::ZERO != theta || openqasm2::Value::ZERO != phi {
			return None;
		}
		Some(Self {
			angle: lambda.into(),
			target,
		})
	}
}

impl<T> RandomGate for Rz<T>
where
	StandardUniform: Distribution<T>,
{
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		Self {
			angle: rng.random(),
			target: rng.random_range(..n_qubits),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct X {
	pub target: usize,
}

impl RandomGate for X {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		Self {
			target: rng.random_range(..n_qubits),
		}
	}
}

#[cfg(feature = "openqasm2")]
impl crate::openqasm2::OpenQasm2Gate for X {
	fn cx(_: usize, _: usize) -> Option<Self> {
		None
	}

	fn u(
		theta: openqasm2::Value,
		phi: openqasm2::Value,
		lambda: openqasm2::Value,
		target: usize,
	) -> Option<Self> {
		if theta == openqasm2::Value::PI
			&& phi == openqasm2::Value::PI_2.checked_neg().unwrap()
			&& lambda == openqasm2::Value::PI
		{
			Some(Self { target })
		} else {
			None
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Y {
	pub target: usize,
}

impl RandomGate for Y {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		Self {
			target: rng.random_range(..n_qubits),
		}
	}
}

#[cfg(feature = "openqasm2")]
impl crate::openqasm2::OpenQasm2Gate for Y {
	fn cx(_: usize, _: usize) -> Option<Self> {
		None
	}

	fn u(
		theta: openqasm2::Value,
		phi: openqasm2::Value,
		lambda: openqasm2::Value,
		target: usize,
	) -> Option<Self> {
		if theta == openqasm2::Value::PI
			&& phi == openqasm2::Value::ZERO
			&& lambda == openqasm2::Value::ZERO
		{
			Some(Self { target })
		} else {
			None
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct H {
	pub target: usize,
}

#[cfg(feature = "openqasm2")]
impl crate::openqasm2::OpenQasm2Gate for H {
	fn cx(_: usize, _: usize) -> Option<Self> {
		None
	}

	fn u(
		theta: openqasm2::Value,
		phi: openqasm2::Value,
		lambda: openqasm2::Value,
		target: usize,
	) -> Option<Self> {
		if theta == openqasm2::Value::PI_2
			&& phi == openqasm2::Value::ZERO
			&& lambda == openqasm2::Value::PI
		{
			Some(Self { target })
		} else {
			None
		}
	}
}

impl RandomGate for H {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		Self {
			target: rng.random_range(..n_qubits),
		}
	}
}

/// # Attention
/// Currently there is nothing stopping you from having control == target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CNot {
	control: usize,
	target: usize,
}

#[cfg(feature = "openqasm2")]
impl crate::openqasm2::OpenQasm2Gate for CNot {
	fn cx(control: usize, target: usize) -> Option<Self> {
		if control == target {
			return None;
		} else {
			Some(Self { control, target })
		}
	}

	fn u(_: openqasm2::Value, _: openqasm2::Value, _: openqasm2::Value, _: usize) -> Option<Self> {
		None
	}
}

impl RandomGate for CNot {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		let control = rng.random_range(..n_qubits);
		let mut target = rng.random_range(..(n_qubits - 1));
		if target >= control {
			target += 1;
		}

		Self { control, target }
	}
}

impl CNot {
	/// Fails if contro == target
	pub fn new(control: usize, target: usize) -> Option<Self> {
		if control != target {
			Some(CNot { control, target })
		} else {
			None
		}
	}

	pub fn target(&self) -> usize {
		self.target
	}

	pub fn control(&self) -> usize {
		self.control
	}

	pub fn reverse(&self) -> Self {
		CNot {
			control: self.target,
			target: self.control,
		}
	}

	/// This should probably be moved to a trait
	pub fn random<R: rand::prelude::Rng>(qubits: usize, rng: &mut R) -> Self {
		use rand::RngExt;

		let control = (qubits as f64 * rng.random::<f64>()).floor() as usize;
		let mut target = ((qubits - 1) as f64 * rng.random::<f64>()).floor() as usize;
		// Need to make sure we get different target
		if target >= control {
			target += 1;
		}
		CNot { control, target }
	}
}
