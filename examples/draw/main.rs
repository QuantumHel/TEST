use std::fs::File;
use std::io::Write;

use test_transpiler::{
	draw::ImageSize,
	pauli::{FreePauliAngle, PauliExp},
	pauli_string,
};

fn main() {
	let p: PauliExp<7, FreePauliAngle> = PauliExp {
		angle: FreePauliAngle::MultipleOfPi(1.4),
		string: pauli_string!("XYZIXXY"),
	};

	let strings = vec![
		pauli_string!("IIXIXXY"),
		pauli_string!("ZXXXIII"),
		pauli_string!("XZZXIII"),
	];
	let size = ImageSize::FixedWidth(500);

	let svg = p.draw_pi_over_4_evolution(&strings, size);

	let mut file = File::options()
		.write(true)
		.truncate(true)
		.create(true)
		.open("./examples/draw/image.html")
		.unwrap();

	writeln!(&mut file, "<!DOCTYPE html>").unwrap();
	writeln!(&mut file, "<html>").unwrap();
	writeln!(&mut file, "<body>").unwrap();
	writeln!(&mut file, "{svg}").unwrap();
	writeln!(&mut file, "</html>").unwrap();
	writeln!(&mut file, "</body>").unwrap();
}
