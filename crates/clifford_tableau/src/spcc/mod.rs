pub mod gateset;
use core::Compiler;

use circuit::Circuit;

use crate::CliffordTableau;

use self::gateset::CNotHSV;

pub struct SPCC;

impl Compiler<CliffordTableau, Circuit<CNotHSV>> for SPCC {
	fn compile(&self, input: CliffordTableau, device: &()) -> Circuit<CNotHSV> {
		todo!()
	}
}

#[cfg(test)]
mod tests {
	use super::{SPCC, gateset::CNotHSV};

	#[test]
	#[ignore = "not yet implemented"]
	fn random_spcc_test() {
		let compiler = SPCC;
		crate::random_test::random_clifford_tableau_test::<SPCC, CNotHSV, ()>(
			&compiler,
			&(),
			100,
			30,
			300,
		);
	}
}
