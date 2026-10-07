mod common;

use self::common::TestIR;
use common::TestGate::*;

/// From: https://github.com/meamy/feynman/blob/master/benchmarks/qasm/tof_3.qasm
#[test]
fn tof_3() {
	let src = "OPENQASM 2.0;
include \"qelib1.inc\";
qreg qubits[5];
h qubits[4];
h qubits[4];
ccx qubits[0],qubits[1],qubits[4];
h qubits[4];
h qubits[4];
h qubits[3];
h qubits[3];
ccx qubits[2],qubits[4],qubits[3];
h qubits[3];
h qubits[3];
h qubits[4];
h qubits[4];
ccx qubits[0],qubits[1],qubits[4];
h qubits[4];
h qubits[4];";

	let frontend = common::full_opaques_frontend();
	let mut ir = TestIR::default();
	frontend.compile_str(src, &mut ir).unwrap();

	let expected = vec![
		H(4),
		H(4),
		Ccx(0, 1, 4),
		H(4),
		H(4),
		H(3),
		H(3),
		Ccx(2, 4, 3),
		H(3),
		H(3),
		H(4),
		H(4),
		Ccx(0, 1, 4),
		H(4),
		H(4),
	];

	assert_eq!(*ir, expected);
}

#[test]
fn cxx_decomposition() {
	let src = "OPENQASM 2.0; qreg qubits[3]; ccx qubits[0],qubits[1],qubits[2];";

	let frontend = common::cxx_decomposition_frontend();
	let mut ir = TestIR::default();
	frontend.compile_str(src, &mut ir).unwrap();

	let expected = vec![
		H(2),
		T(0),
		T(1),
		T(2),
		Cx(1, 0),
		Cx(2, 1),
		Cx(0, 2),
		Tdg(1),
		Cx(0, 1),
		Tdg(0),
		Tdg(1),
		T(2),
		Cx(2, 1),
		Cx(0, 2),
		Cx(1, 0),
		H(2),
	];

	assert_eq!(*ir, expected);
}

#[test]
fn tof_3_file() {
	let frontend = common::full_opaques_frontend();
	let mut ir = TestIR::default();
	frontend
		.compile_file("./tests/tof_3.qasm", &mut ir)
		.unwrap();

	let expected = vec![
		H(4),
		H(4),
		Ccx(0, 1, 4),
		H(4),
		H(4),
		H(3),
		H(3),
		Ccx(2, 4, 3),
		H(3),
		H(3),
		H(4),
		H(4),
		Ccx(0, 1, 4),
		H(4),
		H(4),
	];

	assert_eq!(*ir, expected);
}

#[test]
fn quantum_register_gate_expansion() {
	let src =
		"OPENQASM 2.0; gate test a, b { cx a, b; cx b, a;} qreg a[1]; qreg b[2]; test a[0], b;";

	let frontend = common::full_opaques_frontend();
	let mut ir = TestIR::default();
	frontend.compile_str(src, &mut ir).unwrap();

	let expected = vec![Cx(0, 1), Cx(1, 0), Cx(0, 2), Cx(2, 0)];

	assert_eq!(*ir, expected);
}
