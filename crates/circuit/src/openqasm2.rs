use openqasm2::GateWriter;
use openqasm2::Value;
use std::error::Error;
use std::fmt::Display;

use crate::Circuit;

pub trait OpenQasm2Gate: Sized {
	fn cx(control: usize, target: usize) -> Option<Self>;

	fn u(theta: Value, phi: Value, lambda: Value, target: usize) -> Option<Self>;
}

#[derive(Debug)]
pub enum OpenQasm2Operation {
	CX {
		control: usize,
		target: usize,
	},
	U {
		theta: Value,
		phi: Value,
		lambda: Value,
		target: usize,
	},
}

#[derive(Debug)]
pub enum OQ2ToCircuitError {
	UnsupporteOperation(OpenQasm2Operation),
	BitsUsed,
	BarrierUsed,
	ConditionalUsed,
	MeasureUsed,
	OpaqueUsed,
	ResetUsed,
}

impl Display for OQ2ToCircuitError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::UnsupporteOperation(OpenQasm2Operation::CX { control, target }) => {
				write!(f, "Error: Gate 'cx {control}, {target};' not supported")
			}
			Self::UnsupporteOperation(OpenQasm2Operation::U {
				theta,
				phi,
				lambda,
				target,
			}) => {
				write!(
					f,
					"Error: Gate 'u({theta}, {phi}, {lambda}) {target};' not supported"
				)
			}
			Self::BitsUsed => {
				write!(f, "Error: Classical bits are not supported in circuits.")
			}
			Self::BarrierUsed => {
				write!(f, "Error: barriers are not supported in circuits.")
			}
			Self::ConditionalUsed => {
				write!(f, "Error: conditionals are not supported in circuits.")
			}
			Self::MeasureUsed => {
				write!(f, "Error: measurements are not supported in circuits.")
			}
			Self::OpaqueUsed => {
				write!(f, "Error: opaques are not supported in circuits.")
			}
			Self::ResetUsed => {
				write!(f, "Error: resetting qubits is not supported in circuits.")
			}
		}
	}
}

impl Error for OQ2ToCircuitError {}

impl<T: OpenQasm2Gate> GateWriter for &mut Circuit<T> {
	type Error = OQ2ToCircuitError;

	fn initialize(
		&mut self,
		_: &[openqasm2::Symbol],
		bits: &[openqasm2::Symbol],
	) -> Result<(), Self::Error> {
		if !bits.is_empty() {
			return Err(OQ2ToCircuitError::BitsUsed);
		}
		Ok(())
	}

	fn write_cx(&mut self, copy: usize, xor: usize) -> Result<(), Self::Error> {
		if let Some(gate) = T::cx(copy, xor) {
			self.push(gate);
			Ok(())
		} else {
			Err(OQ2ToCircuitError::UnsupporteOperation(
				OpenQasm2Operation::CX {
					control: copy,
					target: xor,
				},
			))
		}
	}

	fn write_u(
		&mut self,
		theta: Value,
		phi: Value,
		lambda: Value,
		reg: usize,
	) -> Result<(), Self::Error> {
		if let Some(gate) = T::u(theta, phi, lambda, reg) {
			self.push(gate);
			Ok(())
		} else {
			Err(OQ2ToCircuitError::UnsupporteOperation(
				OpenQasm2Operation::U {
					theta,
					phi,
					lambda,
					target: reg,
				},
			))
		}
	}

	fn start_conditional(&mut self, _: usize, _: usize, _: u64) -> Result<(), Self::Error> {
		Err(OQ2ToCircuitError::ConditionalUsed)
	}

	fn end_conditional(&mut self) -> Result<(), Self::Error> {
		Err(OQ2ToCircuitError::ConditionalUsed)
	}

	fn write_barrier(&mut self, _: &[usize]) -> Result<(), Self::Error> {
		Err(OQ2ToCircuitError::BarrierUsed)
	}

	fn write_measure(&mut self, _: usize, _: usize) -> Result<(), Self::Error> {
		Err(OQ2ToCircuitError::MeasureUsed)
	}

	fn write_opaque(
		&mut self,
		_: &openqasm2::Symbol,
		_: &[Value],
		_: &[usize],
	) -> Result<(), Self::Error> {
		Err(OQ2ToCircuitError::OpaqueUsed)
	}

	fn write_reset(&mut self, _: usize) -> Result<(), Self::Error> {
		Err(OQ2ToCircuitError::ResetUsed)
	}
}
