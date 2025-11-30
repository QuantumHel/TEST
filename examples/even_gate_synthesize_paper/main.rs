// molecules 20 qubits
// random 100 qubits

use std::{fs, path::Path, sync::Arc};

use test_transpiler::{connectivity::Connectivity, experiment, misc::NonZeroEvenUsize};

struct Target {
	out_path: String,
	in_path: String,
	connectivity: Arc<Option<Connectivity>>,
}

fn main() {
	for gate_size in (2..=20).step_by(2) {
		let name = format!("gate_size_{gate_size}.exp");
		let gate_size = NonZeroEvenUsize::new(gate_size).unwrap();
		let mut targets: Vec<Target> = Vec::new();

		let full: Arc<Option<Connectivity>> = Arc::new(None);
		let linear_20 = Arc::new(Some(Connectivity::create_line(gate_size, 20)));
		let square_20 = Arc::new(Some(Connectivity::create_square_grid(gate_size, 20)));

		targets.push(Target {
			out_path: "/fully_connected/molecules/".into(),
			in_path: "./datasets/molecules_small/".into(),
			connectivity: full.clone(),
		});

		targets.push(Target {
			out_path: "/line/molecules/".into(),
			in_path: "./datasets/molecules_small/".into(),
			connectivity: linear_20.clone(),
		});

		targets.push(Target {
			out_path: "/square_grid/molecules/".into(),
			in_path: "./datasets/molecules_small/".into(),
			connectivity: square_20.clone(),
		});

		let linear_100 = Arc::new(Some(Connectivity::create_line(gate_size, 100)));
		let square_100 = Arc::new(Some(Connectivity::create_square_grid(gate_size, 100)));

		for gadget_count in (1..=7).map(|v| v * 100) {
			targets.push(Target {
				out_path: format!("/fully_connected/random/{gadget_count}/"),
				in_path: format!("./datasets/random/{gadget_count}/"),
				connectivity: full.clone(),
			});

			targets.push(Target {
				out_path: format!("/line/random/{gadget_count}/"),
				in_path: format!("./datasets/random/{gadget_count}/"),
				connectivity: linear_100.clone(),
			});

			targets.push(Target {
				out_path: format!("/square_grid/random/{gadget_count}/"),
				in_path: format!("./datasets/random/{gadget_count}/"),
				connectivity: square_100.clone(),
			});
		}

		for target in targets {
			let out_path = String::from("./examples/even_gate_synthesize_paper") + &target.out_path;

			if !Path::new(&out_path).exists() {
				fs::create_dir_all(&out_path).unwrap();
			}

			let output_file = out_path + &name;

			experiment::from_folder(
				&target.in_path,
				gate_size,
				target.connectivity,
				&output_file,
			);

			println!();
			println!("Done with {output_file}");
			println!();
		}
	}
}
