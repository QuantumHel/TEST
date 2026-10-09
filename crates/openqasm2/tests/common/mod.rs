use std::{
	num::NonZero,
	ops::{Deref, DerefMut},
};

use test_openqasm2::{
	OpaqueFunction, OpaqueFunctionDefinition, OpenQasm2Config, OpenQasm2Cx, OpenQasm2Frontend,
	VirtualOpenqasmFile,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestGate {
	Cx(usize, usize),
	Ccx(usize, usize, usize),
	H(usize),
	X(usize),
	Y(usize),
	T(usize),
	S(usize),
	Tdg(usize),
	Sdg(usize),
}

#[derive(Debug, Default)]
pub struct TestIR {
	ir: Vec<TestGate>,
}

impl Deref for TestIR {
	type Target = Vec<TestGate>;

	fn deref(&self) -> &Self::Target {
		&self.ir
	}
}

impl DerefMut for TestIR {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.ir
	}
}

pub struct CxxOpaque;

impl OpaqueFunction<TestIR> for CxxOpaque {
	fn definition(&self) -> OpaqueFunctionDefinition {
		OpaqueFunctionDefinition {
			name: String::from("ccx"),
			n_params: 0,
			n_qargs: NonZero::new(3).unwrap(),
		}
	}

	fn type_check(&self, qargs: Vec<usize>) -> Result<(), &'static str> {
		if qargs[0] == qargs[1] || qargs[0] == qargs[2] || qargs[1] == qargs[2] {
			Err("cxx has to have three strictly unique input qubits")
		} else {
			Ok(())
		}
	}

	fn insert_gate(&self, qargs: Vec<usize>, ir: &mut TestIR) {
		ir.ir.push(TestGate::Ccx(qargs[0], qargs[1], qargs[2]));
	}
}

pub struct CxOpaque;

impl OpaqueFunction<TestIR> for CxOpaque {
	fn definition(&self) -> OpaqueFunctionDefinition {
		OpaqueFunctionDefinition {
			name: String::from("cx"),
			n_params: 0,
			n_qargs: NonZero::new(2).unwrap(),
		}
	}

	fn type_check(&self, qargs: Vec<usize>) -> Result<(), &'static str> {
		if qargs[0] == qargs[1] {
			Err("cx has to have two strictly unique input qubits")
		} else {
			Ok(())
		}
	}

	fn insert_gate(&self, qargs: Vec<usize>, ir: &mut TestIR) {
		ir.ir.push(TestGate::Cx(qargs[0], qargs[1]));
	}
}

impl OpenQasm2Cx<TestIR> for CxOpaque {
	fn insert_cnot(&self, control: usize, target: usize, ir: &mut TestIR) {
		ir.push(TestGate::Cx(control, target));
	}
}

macro_rules! single_qubit_gate {
	($name:ident, $opaque:ident, $str:literal) => {
		pub struct $opaque;

		impl OpaqueFunction<TestIR> for $opaque {
			fn definition(&self) -> OpaqueFunctionDefinition {
				OpaqueFunctionDefinition {
					name: String::from($str),
					n_params: 0,
					n_qargs: NonZero::new(1).unwrap(),
				}
			}

			fn type_check(&self, _: Vec<usize>) -> Result<(), &'static str> {
				Ok(())
			}

			fn insert_gate(&self, qargs: Vec<usize>, ir: &mut TestIR) {
				ir.ir.push(TestGate::$name(qargs[0]));
			}
		}
	};
}

single_qubit_gate!(H, HOpaque, "h");
single_qubit_gate!(X, XOpaque, "x");
single_qubit_gate!(Y, YOpaque, "y");
single_qubit_gate!(T, TOpaque, "t");
single_qubit_gate!(S, SOpaque, "s");
single_qubit_gate!(Tdg, TdgOpaque, "tdg");
single_qubit_gate!(Sdg, SdgOpaque, "sdg");

pub fn full_opaques_frontend() -> OpenQasm2Frontend<TestIR> {
	let mut file = VirtualOpenqasmFile::default();
	file.add_opaque(CxOpaque).unwrap();
	file.add_opaque(CxxOpaque).unwrap();
	file.add_opaque(HOpaque).unwrap();
	file.add_opaque(XOpaque).unwrap();
	file.add_opaque(YOpaque).unwrap();
	file.add_opaque(TOpaque).unwrap();
	file.add_opaque(SOpaque).unwrap();
	file.add_opaque(TdgOpaque).unwrap();
	file.add_opaque(SdgOpaque).unwrap();

	OpenQasm2Frontend::new(OpenQasm2Config {
		default_file: Some(file),
		cx: Some(Box::new(CxOpaque)),
		ignore_imports: true,
	})
}

pub fn cxx_decomposition_frontend() -> OpenQasm2Frontend<TestIR> {
	let mut file = VirtualOpenqasmFile::default();
	file.add_opaque(CxOpaque).unwrap();
	file.add_opaque(HOpaque).unwrap();
	file.add_opaque(XOpaque).unwrap();
	file.add_opaque(YOpaque).unwrap();
	file.add_opaque(TOpaque).unwrap();
	file.add_opaque(SOpaque).unwrap();
	file.add_opaque(TdgOpaque).unwrap();
	file.add_opaque(SdgOpaque).unwrap();

	// From figure 13 in https://arxiv.org/pdf/1206.0758
	file.add_text(
		"gate ccx a, b, c {
			h c;
			t a; t b; t c;
			cx b, a;
			cx c, b;
			cx a, c;
			tdg b;
			cx a, b;
			tdg a; tdg b; t c;
			cx c, b;
			cx a, c;
			cx b, a;
			h c;
		}",
	);

	OpenQasm2Frontend::new(OpenQasm2Config {
		default_file: Some(file),
		cx: Some(Box::new(CxOpaque)),
		ignore_imports: true,
	})
}
