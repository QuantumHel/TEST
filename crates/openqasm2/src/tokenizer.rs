use std::{fs::read_to_string, rc::Rc};

use crate::{
	OpaqueFunctionDefinition, OpenQasm2Frontend, OpenQasm2IR, VirtualFileOverrideList,
	error::{Error, ErrorKind, Location},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
	pub ty: TokenKind,
	pub location: Location,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
	// OPENQASM
	Openqasm,
	/// opaque
	Opaque,
	/// {
	OpeningCurlyBracket,
	/// }
	ClosingCurlyBracket,
	/// [
	OpeningSquareBracket,
	/// ]
	ClosingSquareBracket,
	/// (
	OpeningBracket,
	/// )
	ClosingBracket,
	/// if
	If,
	/// barrier
	Barrier,
	/// ==
	DoubleEquals,
	/// qreg
	Qreg,
	/// creg
	Creg,
	/// gate
	Gate,
	/// measure
	Measure,
	/// reset
	Reset,
	/// [1-9]+[0-9]*|0
	Nninteger(u64),
	/// ([0-9]+\.[0-9]*|[0-9]*\.[0-9]+)([eE][-+]?[0-9]+)?
	Real(f64),
	/// [a-z][A-Za-z0-9_]*
	Id(String),
	/// Pi
	Pi,
	/// +
	Plus,
	/// -
	Minus,
	/// *
	Mul,
	/// /
	Div,
	/// ^
	Pow,
	/// sin
	Sin,
	/// cos,
	Cos,
	/// tan
	Tan,
	/// exp
	Exp,
	/// ln
	Ln,
	/// sqrt
	Sqrt,
	/// `;`
	Semicolon,
	/// `,`
	Comma,
	/// `->`
	Arrow,
	/// U
	U,
	/// CX
	Cx,
}

impl<T: OpenQasm2IR> OpenQasm2Frontend<T> {
	pub(crate) fn tokenize(
		&self,
		file: &str,
		src: &str,
	) -> Result<(Vec<Token>, VirtualFileOverrideList), Error> {
		let (mut tokens, mut override_list) = Tokenizer::new(self, file, src).run()?;

		let mut default_file_tokens = Vec::new();
		for opaque in self.default_file.opaque_functions.values() {
			default_file_tokens.append(&mut opaque.definition().tokenize());
		}

		let (mut new_tokes, mut new_overrides) =
			Tokenizer::new(self, "Default virtual file", &self.default_file.text).run()?;
		default_file_tokens.append(&mut new_tokes);
		override_list.append(&mut new_overrides);

		// FIXME: This is valid if the program is valid.
		// Maybe need to move to parser for proper handling.
		let insert_location = tokens.len().min(3);
		tokens.splice(insert_location..insert_location, default_file_tokens);

		Ok((tokens, override_list))
	}
}

struct Tokenizer<'a, T: OpenQasm2IR> {
	settings: &'a OpenQasm2Frontend<T>,
	file: Rc<str>,
	src: &'a str,
	/// Byte offset into `src`.
	pos: usize,
	row: usize,
	col: usize,
	output: Vec<Token>,
	override_list: VirtualFileOverrideList,
}

impl<'a, T: OpenQasm2IR> Tokenizer<'a, T> {
	/// Outer is false in all but the main file
	fn new(settings: &'a OpenQasm2Frontend<T>, file: &str, src: &'a str) -> Self {
		Self {
			settings,
			file: Rc::from(file),
			src,
			pos: 0,
			row: 1,
			col: 1,
			output: Vec::new(),
			override_list: VirtualFileOverrideList::default(),
		}
	}

	/// The location of the *next* character.
	fn location(&self) -> Location {
		Location {
			file: self.file.clone(),
			row: self.row,
			col: self.col,
		}
	}

	/// Byte `offset` bytes ahead of the cursor. Only ever compared against
	/// ASCII, which can never appear inside a multi-byte UTF-8 sequence.
	fn peek(&self, offset: usize) -> Option<u8> {
		self.src.as_bytes().get(self.pos + offset).copied()
	}

	fn bump(&mut self) -> Option<char> {
		let c = self.src[self.pos..].chars().next()?;
		self.pos += c.len_utf8();
		if c == '\n' {
			self.row += 1;
			self.col = 1;
		} else {
			self.col += 1;
		}
		Some(c)
	}

	fn slice(&self, start: usize) -> Rc<str> {
		let a = self.src[start..self.pos].to_string();
		Rc::from(a)
	}

	/// Whitespace and `//` line comments.
	fn skip_trivia(&mut self) {
		loop {
			match self.peek(0) {
				Some(b) if b.is_ascii_whitespace() => {
					self.bump();
				}
				Some(b'/') if self.peek(1) == Some(b'/') => {
					while !matches!(self.peek(0), None | Some(b'\n')) {
						self.bump();
					}
				}
				_ => return,
			}
		}
	}

	fn run(mut self) -> Result<(Vec<Token>, VirtualFileOverrideList), Error> {
		loop {
			self.skip_trivia();

			let location = self.location();
			let Some(byte) = self.peek(0) else {
				return Ok((self.output, self.override_list));
			};

			let ty = match byte {
				b'{' => self.single(TokenKind::OpeningCurlyBracket),
				b'}' => self.single(TokenKind::ClosingCurlyBracket),
				b'[' => self.single(TokenKind::OpeningSquareBracket),
				b']' => self.single(TokenKind::ClosingSquareBracket),
				b'(' => self.single(TokenKind::OpeningBracket),
				b')' => self.single(TokenKind::ClosingBracket),
				b';' => self.single(TokenKind::Semicolon),
				b',' => self.single(TokenKind::Comma),
				b'+' => self.single(TokenKind::Plus),
				b'*' => self.single(TokenKind::Mul),
				b'/' => self.single(TokenKind::Div),
				b'^' => self.single(TokenKind::Pow),
				b'-' => {
					self.bump();
					if self.peek(0) == Some(b'>') {
						self.bump();
						TokenKind::Arrow
					} else {
						TokenKind::Minus
					}
				}
				b'=' => {
					self.bump();
					if self.peek(0) == Some(b'=') {
						self.bump();
						TokenKind::DoubleEquals
					} else {
						return Err(Error::new(ErrorKind::UnexpectedEquals, location));
					}
				}
				b'0'..=b'9' | b'.' => self.number(location.clone())?,
				b if b.is_ascii_alphabetic() => match self.word(location.clone()) {
					Ok(Some(ty)) => ty,
					Ok(_) => {
						continue;
					}
					Err(e) => return Err(e),
				},
				_ => {
					// Always consume, so an error can never stall the iterator.
					let c = self.bump().expect("peeked");
					return Err(Error::new(ErrorKind::UnexpectedChar(c), location));
				}
			};

			self.output.push(Token { ty, location });
		}
	}

	fn single(&mut self, ty: TokenKind) -> TokenKind {
		self.bump();
		ty
	}

	fn read_file(&mut self) -> Result<(), Error> {
		self.skip_trivia();
		let location = self.location();

		match self.peek(0) {
			Some(b'"') => {
				let path = self.read_string(location.clone())?;
				let location = self.location();
				if self.bump() != Some(';') {
					return Err(Error::new(
						ErrorKind::Custom(String::from(
							"Malformed import. Expected ';' after import name",
						)),
						location,
					));
				}
				if self.settings.ignore_imports {
					return Ok(());
				}
				if let Some(virtual_file) = self.settings.file_overrides.get(path.as_ref()) {
					self.override_list.insert(Rc::from(path.to_string()));
					for opaque in virtual_file.opaque_functions.values() {
						self.output.append(&mut opaque.definition().tokenize());
					}

					let (mut new_tokes, mut new_overrides) = Tokenizer::new(
						self.settings,
						&format!("virtual_file_{}", path),
						&virtual_file.text,
					)
					.run()?;
					self.output.append(&mut new_tokes);
					self.override_list.append(&mut new_overrides);
				} else {
					let Ok(text) = read_to_string(path.to_string()) else {
						return Err(Error::new(
							ErrorKind::UnableToReadFile(path.to_string()),
							location.clone(),
						));
					};

					let (mut new_tokes, mut new_overrides) =
						Tokenizer::new(self.settings, &path, &text).run()?;
					self.output.append(&mut new_tokes);
					self.override_list.append(&mut new_overrides);
				}

				Ok(())
			}
			_ => Err(Error::new(ErrorKind::MissingIncludeString, self.location())),
		}
	}

	fn read_string(&mut self, start: Location) -> Result<Rc<str>, Error> {
		self.bump(); // opening quote
		let from = self.pos;
		loop {
			match self.peek(0) {
				Some(b'"') => {
					let text = self.slice(from);
					self.bump();
					return Ok(text);
				}
				Some(b'\n') => return Err(Error::new(ErrorKind::NewlineInString, start)),
				None => return Err(Error::new(ErrorKind::UnterminatedString, start)),
				Some(_) => {
					self.bump();
				}
			}
		}
	}

	fn number(&mut self, start: Location) -> Result<TokenKind, Error> {
		let from = self.pos;

		let int_len = self.munch_digits();
		if self.peek(0) != Some(b'.') {
			let text = self.slice(from);
			if text.len() > 1 && text.starts_with('0') {
				return Err(Error::new(ErrorKind::LeadingZero, start));
			}
			// Note: `1e3` is *not* a real in QASM 2.0 (the grammar requires a
			// decimal point), so it lexes as `Nninteger(1)` then `Id("e3")`.
			return text
				.parse::<u64>()
				.map(TokenKind::Nninteger)
				.map_err(|_| Error::new(ErrorKind::IntegerOverflow, start));
		}

		self.bump(); // '.'
		let frac_len = self.munch_digits();
		if int_len == 0 && frac_len == 0 {
			return Err(Error::new(ErrorKind::MalformedReal, start));
		}

		// Exponent, but only if it's well formed — otherwise the `e` belongs to
		// the next token. `1.0exp` is `Real(1.0)` followed by `Exp`.
		if matches!(self.peek(0), Some(b'e' | b'E')) {
			let digit_at = if matches!(self.peek(1), Some(b'+' | b'-')) {
				2
			} else {
				1
			};
			if matches!(self.peek(digit_at), Some(b'0'..=b'9')) {
				for _ in 0..digit_at {
					self.bump();
				}
				self.munch_digits();
			}
		}

		// The slice is a subset of Rust's own float grammar, so this can't fail.
		self.slice(from)
			.parse::<f64>()
			.map(TokenKind::Real)
			.map_err(|_| Error::new(ErrorKind::MalformedReal, start))
	}

	/// Consumes a run of digits, returning how many.
	fn munch_digits(&mut self) -> usize {
		let from = self.pos;
		while matches!(self.peek(0), Some(b'0'..=b'9')) {
			self.bump();
		}
		self.pos - from
	}

	fn word(&mut self, start: Location) -> Result<Option<TokenKind>, Error> {
		let from = self.pos;
		while matches!(
			self.peek(0),
			Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
		) {
			self.bump();
		}
		let text = self.slice(from);

		Ok(match text.as_ref() {
			"OPENQASM" => Some(TokenKind::Openqasm),
			"U" => Some(TokenKind::U),
			"CX" => Some(TokenKind::Cx),
			"include" => {
				self.read_file()?;
				None
			}
			"opaque" => Some(TokenKind::Opaque),
			"if" => Some(TokenKind::If),
			"barrier" => Some(TokenKind::Barrier),
			"qreg" => Some(TokenKind::Qreg),
			"creg" => Some(TokenKind::Creg),
			"gate" => Some(TokenKind::Gate),
			"measure" => Some(TokenKind::Measure),
			"reset" => Some(TokenKind::Reset),
			"pi" => Some(TokenKind::Pi),
			"sin" => Some(TokenKind::Sin),
			"cos" => Some(TokenKind::Cos),
			"tan" => Some(TokenKind::Tan),
			"exp" => Some(TokenKind::Exp),
			"ln" => Some(TokenKind::Ln),
			"sqrt" => Some(TokenKind::Sqrt),
			_ if text.starts_with(|c: char| c.is_ascii_lowercase()) => {
				Some(TokenKind::Id(text.to_string()))
			}
			_ => {
				return Err(Error::new(
					ErrorKind::InvalidIdentifier(text.to_string()),
					start,
				));
			}
		})
	}
}

impl OpaqueFunctionDefinition {
	fn tokenize(&self) -> Vec<Token> {
		let location = Location {
			file: Rc::from("generated"),
			row: 0,
			col: 0,
		};
		let mut tokens = vec![
			Token {
				ty: TokenKind::Opaque,
				location: location.clone(),
			},
			Token {
				ty: TokenKind::Id(self.name.clone()),
				location: location.clone(),
			},
		];

		if self.n_params > 0 {
			tokens.push(Token {
				ty: TokenKind::OpeningBracket,
				location: location.clone(),
			});

			tokens.push(Token {
				ty: TokenKind::Id(String::from("c0")),
				location: location.clone(),
			});
			for i in 1..self.n_params {
				tokens.push(Token {
					ty: TokenKind::Comma,
					location: location.clone(),
				});
				tokens.push(Token {
					ty: TokenKind::Id(format!("c{i}")),
					location: location.clone(),
				});
			}

			tokens.push(Token {
				ty: TokenKind::ClosingBracket,
				location: location.clone(),
			});
		}

		tokens.push(Token {
			ty: TokenKind::Id(String::from("q0")),
			location: location.clone(),
		});
		for qarg in 1..self.n_qargs.get() {
			tokens.push(Token {
				ty: TokenKind::Comma,
				location: location.clone(),
			});
			tokens.push(Token {
				ty: TokenKind::Id(format!("q{qarg}")),
				location: location.clone(),
			});
		}

		tokens.push(Token {
			ty: TokenKind::Semicolon,
			location,
		});

		tokens
	}
}

#[cfg(test)]
mod tests {
	use super::TokenKind::*;
	use super::*;

	fn kinds(src: &'static str) -> Vec<TokenKind> {
		OpenQasm2Frontend::<()>::default()
			.tokenize("test.qasm", src)
			.expect("lexes cleanly")
			.0
			.into_iter()
			.map(|t| t.ty)
			.collect()
	}

	fn error(src: &'static str) -> ErrorKind {
		OpenQasm2Frontend::<()>::default()
			.tokenize("test.qasm", src)
			.expect_err("should fail")
			.kind
	}

	#[test]
	fn full_program() {
		const SRC: &str = r#"OPENQASM 2.0;
// a comment
qreg q[2];
creg c[2];
gate mygate(theta) a, b { U(theta / 2, 0, pi) a; CX a, b; }
if (c == 3) reset q[1];
barrier q;
measure q -> c;
"#;

		assert_eq!(
			kinds(SRC),
			vec![
				Openqasm,
				Real(2.0),
				Semicolon,
				Qreg,
				Id(String::from("q")),
				OpeningSquareBracket,
				Nninteger(2),
				ClosingSquareBracket,
				Semicolon,
				Creg,
				Id(String::from("c")),
				OpeningSquareBracket,
				Nninteger(2),
				ClosingSquareBracket,
				Semicolon,
				Gate,
				Id(String::from("mygate")),
				OpeningBracket,
				Id(String::from("theta")),
				ClosingBracket,
				Id(String::from("a")),
				Comma,
				Id(String::from("b")),
				OpeningCurlyBracket,
				U,
				OpeningBracket,
				Id(String::from("theta")),
				Div,
				Nninteger(2),
				Comma,
				Nninteger(0),
				Comma,
				Pi,
				ClosingBracket,
				Id(String::from("a")),
				Semicolon,
				Cx,
				Id(String::from("a")),
				Comma,
				Id(String::from("b")),
				Semicolon,
				ClosingCurlyBracket,
				If,
				OpeningBracket,
				Id(String::from("c")),
				DoubleEquals,
				Nninteger(3),
				ClosingBracket,
				Reset,
				Id(String::from("q")),
				OpeningSquareBracket,
				Nninteger(1),
				ClosingSquareBracket,
				Semicolon,
				Barrier,
				Id(String::from("q")),
				Semicolon,
				Measure,
				Id(String::from("q")),
				Arrow,
				Id(String::from("c")),
				Semicolon,
			]
		);
	}

	#[test]
	fn numbers() {
		assert_eq!(
			kinds("0 10 1.0 .5 2. 1.5e-3 1.5E+3 3.e2"),
			vec![
				Nninteger(0),
				Nninteger(10),
				Real(1.0),
				Real(0.5),
				Real(2.0),
				Real(1.5e-3),
				Real(1.5e3),
				Real(3.0e2),
			]
		);
		// No decimal point means it isn't a real, so the `e3` falls out as an id.
		assert_eq!(kinds("1e3"), vec![Nninteger(1), Id(String::from("e3"))]);
		// A trailing `e` with no digits isn't part of the literal either.
		assert_eq!(kinds("1.0ex"), vec![Real(1.0), Id(String::from("ex"))]);
		assert_eq!(error("007"), ErrorKind::LeadingZero);
		assert_eq!(error("99999999999999999999"), ErrorKind::IntegerOverflow);
		assert_eq!(error("."), ErrorKind::MalformedReal);
	}

	#[test]
	fn operators_and_functions() {
		assert_eq!(
			kinds("+ - * / ^ sin cos tan exp ln sqrt pi"),
			vec![Plus, Minus, Mul, Div, Pow, Sin, Cos, Tan, Exp, Ln, Sqrt, Pi,]
		);
		assert_eq!(kinds("->"), vec![Arrow]);
		// `>` only exists as part of `->`.
		assert_eq!(error("- >"), ErrorKind::UnexpectedChar('>'));
	}

	#[test]
	fn identifiers() {
		assert_eq!(
			kinds("q0 my_gate_1 sine"),
			vec![
				Id(String::from("q0")),
				Id(String::from("my_gate_1")),
				Id(String::from("sine"))
			]
		);
		assert_eq!(
			error("Foo"),
			ErrorKind::InvalidIdentifier(String::from("Foo"))
		);
	}

	#[test]
	fn locations() {
		let tokens = OpenQasm2Frontend::<()>::default()
			.tokenize("test.qasm", "qreg\n  q[1];")
			.unwrap();
		assert_eq!(
			tokens.0[0].location,
			Location {
				file: Rc::from("test.qasm"),
				row: 1,
				col: 1
			}
		);
		assert_eq!(
			tokens.0[1].location,
			Location {
				file: Rc::from("test.qasm"),
				row: 2,
				col: 3
			}
		);
		assert_eq!(
			tokens.0[2].location,
			Location {
				file: Rc::from("test.qasm"),
				row: 2,
				col: 4
			}
		);
	}

	#[test]
	fn errors() {
		assert_eq!(error("c = 3"), ErrorKind::UnexpectedEquals);
		assert_eq!(error("include \"unclosed"), ErrorKind::UnterminatedString);
		assert_eq!(error("include \"oops\nx"), ErrorKind::NewlineInString);
		assert_eq!(error("q @ 1"), ErrorKind::UnexpectedChar('@'));
	}

	#[test]
	fn comments_and_eof() {
		assert_eq!(kinds(""), vec![]);
		assert_eq!(kinds("   \n\t "), vec![]);
		assert_eq!(kinds("// just a comment"), vec![]);
		assert_eq!(
			kinds("q; // trailing\nc"),
			vec![Id(String::from("q")), Semicolon, Id(String::from("c"))]
		);
	}
}
