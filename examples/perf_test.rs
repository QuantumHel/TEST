use std::sync::Arc;

use test_transpiler::{connectivity::Connectivity, experiment, misc::NonZeroEvenUsize};

fn main() {
	let gate_size = NonZeroEvenUsize::new(8).unwrap();
	let linear_30 = Arc::new(Some(Connectivity::create_line(gate_size, 30)));

	experiment::from_folder("./datasets/molecules/", gate_size, linear_30, "perf.exp");
}
