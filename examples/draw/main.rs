use std::fs::File;
use std::io::Write;
use test_transpiler::clifford_tableau::CliffordTableau;
use test_transpiler::draw::ImageSize;
use test_transpiler::draw::{VisualRow, VisualText, draw_rows};
use test_transpiler::pauli::PauliString;
use test_transpiler::pauli_string;

const N: usize = 4;

fn make_image(clifford_tableau: &CliffordTableau, string: Option<&PauliString>, name: &str) {
	let mut rows: Vec<VisualRow> = Vec::new();

	for i in 0..clifford_tableau.size().max(N) {
		let mut chars: Vec<_> = clifford_tableau.get_x_row(i).as_string().chars().collect();
		chars.resize_with(clifford_tableau.size().max(N), || ' ');
		rows.push(VisualRow::String {
			name: VisualText::plain_text("X").with_subscript(&format!("{i}")),
			letters: chars
				.into_iter()
				.map(|c| if c == 'I' { ' ' } else { c })
				.map(|c| Some(VisualText::plain_text(&format!("{c}"))))
				.collect(),
		});

		let mut chars: Vec<_> = clifford_tableau.get_z_row(i).as_string().chars().collect();
		chars.resize_with(clifford_tableau.size().max(N), || ' ');
		rows.push(VisualRow::String {
			name: VisualText::plain_text("Z").with_subscript(&format!("{i}")),
			letters: chars
				.into_iter()
				.map(|c| if c == 'I' { ' ' } else { c })
				.map(|c| Some(VisualText::plain_text(&format!("{c}"))))
				.collect(),
		});
	}

	let size = ImageSize::Fixed {
		width: 500,
		height: 625,
	};
	//	let size = ImageSize::FixedWidth(500);

	let svg = draw_rows(rows, size);

	let string = match string {
		Some(string) => {
			let mut chars: Vec<_> = string.as_string().chars().collect();
			chars.resize_with(clifford_tableau.size().max(N), || 'I');
			let chars = "i(π/4)".chars().chain(chars);

			VisualRow::String {
				name: VisualText::plain_text("e").with_superscript(&chars.collect::<String>()),
				letters: Vec::new(),
			}
		}
		_ => VisualRow::Empty,
	};
	let svg2 = draw_rows(
		vec![
			VisualRow::Empty,
			VisualRow::Empty,
			string,
			VisualRow::Empty,
			VisualRow::Empty,
			VisualRow::Empty,
		],
		size,
	);

	let mut file = File::options()
		.write(true)
		.truncate(true)
		.create(true)
		.open(format!("./examples/draw/{name}.html"))
		.unwrap();

	writeln!(&mut file, "<!DOCTYPE html>").unwrap();
	writeln!(&mut file, "<html>").unwrap();
	let style = r#"
		<head><style>
.svg-container {
	display: flex;
	gap: 0px;          /* Space between the SVGs */
	align-items: center; /* Vertically align centers */
	width: 1000px
}

.svg-container > svg {
    height: auto;
  }
</style>"#;
	writeln!(&mut file, "{style}").unwrap();
	writeln!(&mut file, "<body>").unwrap();
	writeln!(&mut file, "<div class=\"svg-container\">").unwrap();
	writeln!(&mut file, "{svg}").unwrap();
	writeln!(&mut file, "{svg2}").unwrap();
	writeln!(&mut file, "</div>").unwrap();
	writeln!(&mut file, "</body>").unwrap();
	writeln!(&mut file, "</html>").unwrap();
}

fn main() {
	let _strings = vec![
		pauli_string!("XXIX"),
		pauli_string!("XZZY"),
		pauli_string!("XIYX"),
		pauli_string!("YYYY"),
		pauli_string!("IIXY"),
		pauli_string!("IXZ"),
		pauli_string!("YZ"),
		pauli_string!("IIZY"),
		pauli_string!("IZY"),
		pauli_string!("XX"),
		pauli_string!("IIZY"),
		pauli_string!("IYY"),
		pauli_string!("IIZZ"),
		pauli_string!("IXY"),
		pauli_string!("IX"),
		pauli_string!("IIYY"),
		pauli_string!("IIZX"),
		pauli_string!("IIXZ"),
		pauli_string!("IIX"),
		pauli_string!("IIIX"),
	];

	#[allow(clippy::useless_vec)]
	let strings = vec![
		pauli_string!("IIXZ"),
		pauli_string!("IZXZ"),
		pauli_string!("IIYY"),
		pauli_string!("IIYY"),
		pauli_string!("IZXZ"),
		pauli_string!("IXYZ"),
		// solution
		pauli_string!("YZXY"),
		pauli_string!("YYZX"),
		pauli_string!("YZXY"),
	];

	// Take 12 - 18

	let mut clifford_tableau = CliffordTableau::default();
	make_image(&clifford_tableau, None, "image");
	for (i, string) in strings.iter().enumerate() {
		make_image(&clifford_tableau, None, &format!("image{}", i * 2));
		make_image(
			&clifford_tableau,
			Some(string),
			&format!("image{}", i * 2 + 1),
		);
		clifford_tableau.merge_pi_over_4_pauli(false, string);
	}
	make_image(
		&clifford_tableau,
		None,
		&format!("image{}", strings.len() * 2),
	);
}
