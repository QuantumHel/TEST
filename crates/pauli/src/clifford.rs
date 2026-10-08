use crate::PauliString;

pub type IsNegative = bool;

pub trait Clifford {
	/// Given `self` $U$, and `pauli_string` $P$ returns $UPU^\dagger$.
	///
	/// The return value is a tuple containing a [bool] corresponding to a
	/// negative sign, and the resulting [PauliString] (which does not have an
	/// iternal sign).
	fn conjugate(&self, pauli_string: &PauliString) -> (IsNegative, PauliString);

	/// This lists all qubits with which the [Clifford] interacts.
	///
	/// Can be used for optimizaion like detecting trivial commutation, and
	/// for making sure that we have the needed qubits available.
	fn interacting_qubits(&self) -> impl Iterator<Item = &usize>;
}
