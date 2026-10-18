pub mod connectivity;
pub mod qubit_map;

pub mod prelude {
	pub use super::Compiler;
	pub use super::connectivity::Connectivity;
}

pub trait Compiler<Input, Output, Device = ()>: Sized {
	fn compile(&self, input: Input, device: &Device) -> Output;
}

pub trait TargetQubits {
	/// This lists all qubits with which the Gate interacts.
	///
	/// The order of the gates needs to be consistent. For example in a cnot
	/// gate the order of control and target could be such that control is
	/// **always** first, or such that target is **always** first.
	///
	/// Can be used for optimizaion like detecting trivial commutation, for
	/// making sure that we have the needed qubits available, and for pattern
	/// matching.
	fn target_qubits(&self) -> impl Iterator<Item = &usize>;
}

pub trait OperatorEq {
	/// Returns true `self` and `other` are equal operators (ignoring on which
	/// qubits/bits they act on).
	///
	/// # Example:
	/// - `cnot 1, 2` annd `cnot 5, 3` are equal
	/// - `rz(pi) 4` and `rz(pi/2) 4` are not
	fn operator_eq(&self, other: &Self) -> bool;
}
