pub mod measure;
mod random;
pub use random::random_exp;

use std::{
	env,
	fs::{self, File, ReadDir},
	io::Write,
	path::Path,
	sync::{Arc, Mutex},
	thread::{self, JoinHandle},
};

pub use measure::{gate_count, gate_depth, multi_qubit_filter};

use crate::synthesize::synthesize;
use crate::{
	clifford_tableau::CliffordTableau,
	connectivity::Connectivity,
	misc::NonZeroEvenUsize,
	pauli::{CliffordPauliAngle, PauliAngle, PauliExp},
};

pub struct FolderIterator {
	paths: ReadDir,
}

impl Iterator for FolderIterator {
	type Item = (String, Vec<PauliExp<PauliAngle>>);

	fn next(&mut self) -> Option<Self::Item> {
		self.paths.next().map(|path| {
			let path = path.unwrap().path();
			let name = format!("{}", path.display());
			let target = PauliExp::read_exp_file(path);
			(name, target)
		})
	}
}

pub fn from_folder<P: AsRef<Path>>(
	folder: P,
	gate_size: NonZeroEvenUsize,
	connectivity: Arc<Option<Connectivity>>,
	output_file: &str,
) {
	let paths = fs::read_dir(folder).unwrap();
	let targets = FolderIterator { paths };

	run_experiment(targets, gate_size, connectivity, output_file);
}

pub fn run_experiment<T: Iterator<Item = (String, Vec<PauliExp<PauliAngle>>)> + Send + 'static>(
	targets: T,
	gate_size: NonZeroEvenUsize,
	connectivity: Arc<Option<Connectivity>>,
	output_file: &str,
) {
	let mut file = File::options()
		.create_new(true)
		.write(true)
		.open(output_file)
		.expect("Failed to create output file");
	writeln!(
		file,
		"name,input_gate_count,output_gate_count,input_gate_depth,output_gate_depth"
	)
	.expect("Failed to write to file");

	let file = Arc::new(Mutex::new(file));
	let jobs = Arc::new(Mutex::new(targets));

	let mut handles: Vec<JoinHandle<_>> = Vec::new();

	let threads = env::var("N_THREADS")
		.map(|v| v.parse::<usize>().ok())
		.ok()
		.flatten()
		.unwrap_or(8);

	for _ in 0..threads {
		let file = file.clone();
		let jobs = jobs.clone();
		let connectivity = connectivity.clone();

		handles.push(thread::spawn(move || {
			let connectivity = connectivity.as_ref().as_ref();
			loop {
				let job = jobs.lock().unwrap().next();
				if job.is_none() {
					break;
				}

				let (name, target) = job.unwrap();

				let input_gate_count = gate_count(&target, multi_qubit_filter);
				let input_gate_depth = gate_depth(&target, multi_qubit_filter);

				#[cfg(not(feature = "return_ordered"))]
				let (mut circuit, clifford) = synthesize(target, gate_size, connectivity);
				#[cfg(feature = "return_ordered")]
				let (mut circuit, clifford, _) = synthesize(target, gate_size, connectivity);

				let clifford: Vec<PauliExp<CliffordPauliAngle>> = {
					let mut tableau = CliffordTableau::id();
					for clifford_op in clifford.into_iter() {
						tableau.merge_clifford(clifford_op);
					}

					tableau.decompose(gate_size, connectivity)
				};

				circuit.append(&mut clifford.into_iter().map(PauliExp::from).collect());

				let output_gate_count = gate_count(&circuit, multi_qubit_filter);
				let output_gate_depth = gate_depth(&circuit, multi_qubit_filter);

				println!(
					"{name},{input_gate_count},{output_gate_count},{input_gate_depth},{output_gate_depth}"
				);
				let mut f = file.lock().unwrap();
				writeln!(
					f,
					"{name},{input_gate_count},{output_gate_count},{input_gate_depth},{output_gate_depth}"
				)
				.expect("Failed towrite to file.");
			}
		}));
	}

	for handle in handles {
		handle.join().unwrap();
	}

	file.lock()
		.unwrap()
		.flush()
		.expect("Failed to write to file.")
}
