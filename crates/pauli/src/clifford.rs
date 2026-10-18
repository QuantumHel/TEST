use crate::PauliString;

pub trait Clifford {
	/// Given `self` $U$, and `pauli_string` $P$ returns $UPU^\dagger$.
	///
	/// The return value is a tuple containing a [bool] corresponding to a
	/// negative sign, and the resulting [PauliString] (which does not have an
	/// iternal sign).
	fn conjugate(&self, pauli_string: &PauliString) -> (bool, PauliString);
}
