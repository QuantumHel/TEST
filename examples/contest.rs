use test_transpiler::{connectivity::Connectivity, misc::NonZeroEvenUsize};

fn test(group_size: usize, min_qubit_count: usize) {
	let group_size = NonZeroEvenUsize::new(group_size).unwrap();
	println!(
		"gatesize {} wit at least {min_qubit_count} qubits",
		group_size.as_value()
	);
	let c = Connectivity::create_square_grid(group_size, min_qubit_count);

	println!("{c:?}");
	println!();
}

fn main() {
	test(4, 16);
}
