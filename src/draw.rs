use crate::pauli::{Negate, PauliExp, PauliLetter, PauliString};

type SVGImage = String;

#[derive(Debug, Clone, Copy)]
pub enum ImageSize {
	FixedWidth(u32),
	FixedHeight(u32),
	Fixed { width: u32, height: u32 },
}

const LETTER_Y_ADJUST: f64 = 0.15;
const PURPLE: &str = "magenta";
const RED: &str = "red";
const GREEN: &str = "lightgreen";

impl<T: Negate + Clone> PauliExp<T> {
	/// # Draw pi over 4 evolution
	///
	/// Draws an image that shows how the Pauli string evolves in the
	/// exponential as pi over 4 exponentials are pushed trough it.
	pub fn draw_pi_over_4_evolution(&self, strings: &[PauliString], size: ImageSize) -> SVGImage {
		let len = strings
			.iter()
			.map(PauliString::start_of_trailin_is)
			.max()
			.unwrap_or_default();
		let len = len.max(self.string.start_of_trailin_is());

		// need to fit 3+3*strings.len() squares in height
		// need to fit 4 + len squares in width
		let (width, height, square_size, padding_w, padding_h) = {
			match size {
				ImageSize::FixedWidth(width) => {
					let square_size = width as f64 / (4 + len) as f64;
					let height = square_size * (3 + 3 * strings.len()) as f64;
					(
						width,
						height.round() as u32,
						square_size,
						square_size,
						square_size,
					)
				}
				ImageSize::FixedHeight(height) => {
					let square_size = height as f64 / (3 + 3 * strings.len()) as f64;
					let width = square_size * (4 + len) as f64;
					(
						width.round() as u32,
						height,
						square_size,
						square_size,
						square_size,
					)
				}
				ImageSize::Fixed { width, height } => {
					let max_width = width as f64 / (4 + len) as f64;
					let max_height = height as f64 / (3 + 3 * strings.len()) as f64;
					let square_size = max_height.min(max_width);

					let padding_w = (width as f64 - square_size * (4 + len) as f64) / 2.0;
					let padding_h =
						(height as f64 - square_size * (3 + 3 * strings.len()) as f64) / 2.0;
					(width, height, square_size, padding_w, padding_h)
				}
			}
		};

		let mut res =
			format!("<svg width='{width}' height='{height}' xmlns='http://www.w3.org/2000/svg'>");

		let mut p = self.clone();

		for (y, string) in strings.iter().enumerate() {
			// P
			res += &format!(
				"<text x='{}' y='{}' font-size='{square_size}' text-anchor='middle'>P</text>",
				padding_w + 0.5 * square_size,
				square_size * ((3 * y + 1) as f64 - LETTER_Y_ADJUST) + padding_h
			);
			// O_i
			res += &format!(
				"<text x='{}' y='{}' font-size='{square_size}' text-anchor='middle'>O<tspan font-size='{}'>{}</tspan></text>",
				padding_w + 0.5 * square_size,
				square_size * ((3 * y + 2) as f64 - LETTER_Y_ADJUST) + padding_h,
				0.25 * square_size,
				y
			);

			let commutes: bool = p.string.commutes_with(string);
			// TODO:
			// used to make anticommuting purple
			// tuen if anticommute as whole: additions green, deletions red

			// print Squares for p
			for (x, letter) in p.string.letters() {
				let color = if letter.anticommutes_with(&string.get(x)) {
					PURPLE
				} else if commutes {
					"none"
				} else if string.get(x) == letter {
					RED
				} else {
					"none"
				};
				res += &format!(
					"<rect x='{}' y='{}' width='{square_size}' height={square_size} style='fill:{color};stroke:black;stroke-width:{}' />",
					padding_w + square_size * (x + 2) as f64,
					square_size * (3 * y) as f64 + padding_h,
					square_size * 0.05
				);
				res += &format!(
					"<text x='{}' y='{}' font-size={square_size} text-anchor='middle' test>{}</text>",
					padding_w + square_size * ((x + 2) as f64 + 0.5),
					square_size * ((3 * y + 1) as f64 - LETTER_Y_ADJUST) + padding_h,
					letter
				);
			}

			// print squares for o
			for (x, letter) in string.letters() {
				let color = if letter.anticommutes_with(&p.string.get(x)) {
					PURPLE
				} else if commutes {
					"none"
				} else if p.string.get(x) == letter {
					RED
				} else if p.string.get(x) == PauliLetter::I {
					GREEN
				} else {
					"none"
				};

				res += &format!(
					"<rect x='{}' y='{}' width='{square_size}' height={square_size} style='fill:{color};stroke:black;stroke-width:{}' />",
					padding_w + square_size * (x + 2) as f64,
					square_size * (3 * y + 1) as f64 + padding_h,
					square_size * 0.05
				);
				res += &format!(
					"<text x='{}' y='{}' font-size={square_size} text-anchor='middle' test>{}</text>",
					padding_w + square_size * ((x + 2) as f64 + 0.5),
					square_size * ((3 * y + 2) as f64 - LETTER_Y_ADJUST) + padding_h,
					letter
				);
			}

			// print arrow
			let left = width as f64 / 2. + square_size / 2.;
			let right = left + square_size;
			let top = square_size * ((3 * y + 2) as f64 + LETTER_Y_ADJUST) + padding_h;
			let bot = square_size * ((3 * y + 3) as f64 - LETTER_Y_ADJUST) + padding_h;
			res += &format!(
				"<path d='M{} {} L{} {} L{} {} Z' />",
				left,
				top,
				right,
				top,
				(left + right) / 2.,
				bot,
			);

			p.push_pi_over_4(false, string);
		}

		// print last p row
		let y = strings.len();

		// P
		res += &format!(
			"<text x='{}' y='{}' font-size='{square_size}' text-anchor='middle'>P</text>",
			padding_w + 0.5 * square_size,
			square_size * ((3 * y + 1) as f64 - LETTER_Y_ADJUST) + padding_h
		);

		// letters
		for (x, letter) in p.string.letters() {
			res += &format!(
				"<rect x='{}' y='{}' width='{square_size}' height={square_size} style='fill:none;stroke:black;stroke-width:{}' />",
				padding_w + square_size * (x + 2) as f64,
				square_size * (3 * y) as f64 + padding_h,
				square_size * 0.05
			);
			res += &format!(
				"<text x='{}' y='{}' font-size={square_size} text-anchor='middle' test>{}</text>",
				padding_w + square_size * ((x + 2) as f64 + 0.5),
				square_size * ((3 * y + 1) as f64 - LETTER_Y_ADJUST) + padding_h,
				letter
			);
		}
		// testing
		res.push_str("</svg>");
		res
	}
}

pub struct VisualText {
	pub text: String,
	pub subscript: Option<String>,
	pub superscript: Option<String>,
	pub bg: String,
}

impl Default for VisualText {
	fn default() -> Self {
		Self {
			text: String::new(),
			subscript: None,
			superscript: None,
			bg: String::from("none"),
		}
	}
}

impl VisualText {
	pub fn plain_text(text: &str) -> Self {
		VisualText {
			text: text.to_string(),
			..Default::default()
		}
	}

	pub fn with_subscript(self, subscript: &str) -> Self {
		Self {
			subscript: Some(subscript.to_string()),
			..self
		}
	}

	pub fn with_superscript(self, superscript: &str) -> Self {
		Self {
			superscript: Some(superscript.to_string()),
			..self
		}
	}
}

impl VisualText {
	pub fn as_svg(&self, x: f64, y: f64, font_size: f64) -> String {
		let mut res = format!(
			"<text x='{x}' y='{y}' font-size='{font_size}' text-anchor='middle'>{}",
			self.text
		);

		if let Some(subscript) = self.subscript.as_ref() {
			res += &format!(
				"<tspan font-size='{}'>{}</tspan>",
				font_size * 0.4,
				subscript
			);
		}

		if let Some(superscript) = self.superscript.as_ref() {
			res += &format!(
				"<tspan font-size='{}' dx='-{}' dy='-{}'>{}</tspan>",
				font_size * 0.4,
				if self.subscript.is_some() {
					font_size * 0.224
				} else {
					0.0
				},
				font_size * 0.5,
				superscript
			);
		}

		res += "</text>";

		res
	}
}

pub enum VisualRow {
	String {
		name: VisualText,
		letters: Vec<Option<VisualText>>,
	},
	Arrow,
	Empty,
}

const FONT_MULTIPLIER: f64 = 0.9;

pub fn draw_rows(rows: Vec<VisualRow>, size: ImageSize) -> SVGImage {
	let len = rows
		.iter()
		.map(|r| {
			if let VisualRow::String { letters, .. } = r {
				letters.len()
			} else {
				0
			}
		})
		.max()
		.unwrap_or_default();

	// need to fit 3+3*strings.len() squares in height
	// need to fit 4 + len squares in width
	let (width, height, square_size, padding_w, padding_h) = {
		match size {
			ImageSize::FixedWidth(width) => {
				let square_size = width as f64 / (4 + len) as f64;
				let height = square_size * (2 + rows.len()) as f64;
				(
					width,
					height.round() as u32,
					square_size,
					square_size,
					square_size,
				)
			}
			ImageSize::FixedHeight(height) => {
				let square_size = height as f64 / (2 + rows.len()) as f64;
				let width = square_size * (4 + len) as f64;
				(
					width.round() as u32,
					height,
					square_size,
					square_size,
					square_size,
				)
			}
			ImageSize::Fixed { width, height } => {
				let max_width = width as f64 / (4 + len) as f64;
				let max_height = height as f64 / (rows.len()) as f64;
				let square_size = max_height.min(max_width);

				let padding_w = (width as f64 - square_size * (4 + len) as f64) / 2.0;
				let padding_h = (height as f64 - square_size * (2 + rows.len()) as f64) / 2.0;
				(width, height, square_size, padding_w, padding_h)
			}
		}
	};

	let mut res =
		format!("<svg width='{width}' height='{height}' xmlns='http://www.w3.org/2000/svg'>");

	for (y, row) in rows.iter().enumerate() {
		match row {
			VisualRow::String { name, letters } => {
				res += &name.as_svg(
					padding_w + 0.5 * square_size,
					square_size * (y as f64 - LETTER_Y_ADJUST) + padding_h,
					square_size * FONT_MULTIPLIER,
				);

				for (x, letter) in letters.iter().enumerate() {
					let letter = match letter {
						Some(letter) => letter,
						_ => {
							continue;
						}
					};

					res += &format!(
						"<rect x='{}' y='{}' width='{square_size}' height={square_size} style='fill:{};stroke:black;stroke-width:{}' />",
						padding_w + square_size * (x + 2) as f64,
						square_size * y as f64 + padding_h - square_size,
						letter.bg,
						square_size * 0.05
					);
					res += &letter.as_svg(
						padding_w + square_size * ((x + 2) as f64 + 0.5),
						square_size * (y as f64 - LETTER_Y_ADJUST) + padding_h,
						square_size * FONT_MULTIPLIER,
					);
				}
			}
			VisualRow::Arrow => {
				let left = width as f64 / 2. + square_size / 2.;
				let right = left + square_size;
				let top = square_size * (y as f64 + LETTER_Y_ADJUST) + padding_h;
				let bot = square_size * (y as f64 - LETTER_Y_ADJUST) + padding_h;
				res += &format!(
					"<path d='M{} {} L{} {} L{} {} Z' />",
					left,
					top,
					right,
					top,
					(left + right) / 2.,
					bot,
				);
			}
			VisualRow::Empty => {}
		}
	}

	res.push_str("</svg>");
	res
}
