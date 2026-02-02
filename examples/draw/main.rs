use std::fs::File;
use std::io::Write;
use test_transpiler::draw::{VisualRow, VisualText, draw_rows};
use test_transpiler::{
	draw::ImageSize,
	pauli::{PauliAngle, PauliExp},
	pauli_string,
};

fn main() {
	let size = ImageSize::FixedWidth(500);

	/*
	let p: PauliExp<PauliAngle> = PauliExp {
		angle: PauliAngle::MultipleOfPi(1.4),
		string: pauli_string!("XYZIXXY"),
	};

	let strings = vec![
		pauli_string!("IIXIXXY"),
		pauli_string!("ZXXXIII"),
		pauli_string!("XZZXIII"),
	];

	let svg = p.draw_pi_over_4_evolution(&strings, size);
	 */

	let rows = vec![
		VisualRow::String {
			name: VisualText::plain_text("A"),
			letters: vec![
				Some(VisualText::plain_text("P").with_superscript("b")),
				Some(
					VisualText::plain_text("P")
						.with_subscript("a")
						.with_superscript("++"),
				),
				None,
				Some(VisualText::plain_text("A")),
			],
		},
		VisualRow::Arrow,
		VisualRow::Empty,
		VisualRow::String {
			name: VisualText::plain_text("A"),
			letters: vec![
				Some(VisualText::plain_text("A")),
				Some(VisualText::plain_text("A")),
				None,
				Some(VisualText::plain_text("A")),
			],
		},
	];

	let svg = draw_rows(rows, size);

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
