use std::{iter::Peekable, vec::IntoIter};

use crate::{
	OpenQasm2Frontend, OpenQasm2IR, ast,
	error::{Error, ErrorKind, Location},
	tokenizer::{Token, TokenKind},
};

impl<T: OpenQasm2IR> OpenQasm2Frontend<T> {
	pub(crate) fn parse(&self, tokens: Vec<Token>) -> Result<ast::Program, Error> {
		TokenStream::new(tokens).parse_program()
	}
}

#[derive(Debug)]
struct TokenStream {
	tokens: Peekable<IntoIter<Token>>,
}

// Utilities
impl TokenStream {
	fn new(tokens: Vec<Token>) -> Self {
		Self {
			tokens: tokens.into_iter().peekable(),
		}
	}

	fn next(&mut self) -> Result<Token, Error> {
		match self.tokens.next() {
			Some(token) => Ok(token),
			_ => Err(Error::new(
				ErrorKind::UnexpectedEndOfProgram,
				Location::default(),
			)),
		}
	}

	fn next_expected(&mut self, token_kind: TokenKind) -> Result<Token, Error> {
		match self.tokens.next() {
			Some(token) => {
				if token.ty == token_kind {
					Ok(token)
				} else {
					let location = token.location.clone();
					Err(Error::new(ErrorKind::UnexpectedToken(token), location))
				}
			}
			_ => Err(Error::new(
				ErrorKind::UnexpectedEndOfProgram,
				Location::default(),
			)),
		}
	}

	fn peek(&mut self) -> Option<&Token> {
		self.tokens.peek()
	}

	fn peek_expected(&mut self) -> Result<&Token, Error> {
		match self.tokens.peek() {
			Some(token) => Ok(token),
			_ => Err(Error::new(
				ErrorKind::UnexpectedEndOfProgram,
				Location::default(),
			)),
		}
	}
}

// Parsing
impl TokenStream {
	fn parse_program(&mut self) -> Result<ast::Program, Error> {
		let token1 = self.next()?;
		if token1.ty != TokenKind::Openqasm {
			return Err(Error::new(
				ErrorKind::MissingOpenQasmVersionSpecifier,
				token1.location,
			));
		}

		let version = self.next()?;
		match version.ty {
			TokenKind::Real(2.0) => {}
			TokenKind::Real(version_number) => {
				return Err(Error::new(
					ErrorKind::WrongOpenQasmVersion(version_number),
					version.location,
				));
			}
			_ => {
				let location = version.location.clone();
				return Err(Error::new(ErrorKind::UnexpectedToken(version), location));
			}
		}

		self.next_expected(TokenKind::Semicolon)?;

		let mut statements = Vec::new();
		while self.peek().is_some() {
			statements.push(self.parse_statement()?);
		}

		Ok(ast::Program {
			version: 2.0,
			program: statements,
		})
	}

	fn parse_statement(&mut self) -> Result<ast::Statement, Error> {
		match self.peek_expected()?.ty {
			TokenKind::Qreg | TokenKind::Creg => {
				Ok(ast::Statement::Declaration(self.parse_declaration()?))
			}
			TokenKind::Gate => Ok(ast::Statement::GateDeclaration(
				self.parse_gate_declaration()?,
			)),
			TokenKind::Opaque => Ok(ast::Statement::Opaque(self.parse_opaque()?)),
			TokenKind::If => {
				let start = self.next_expected(TokenKind::If)?;
				self.next_expected(TokenKind::OpeningBracket)?;
				let id = self.parse_id()?;
				self.next_expected(TokenKind::DoubleEquals)?;
				let integer = self.parse_integer()?;
				self.next_expected(TokenKind::ClosingBracket)?;
				let quantum_operator = self.parse_quantum_operator()?;
				Ok(ast::Statement::If {
					id,
					integer,
					quantum_operator,
					location: start.location,
				})
			}
			TokenKind::Barrier => {
				self.next_expected(TokenKind::Barrier)?;
				let mut arguments = Vec::new();
				arguments.push(self.parse_argument()?);
				while self.peek_expected()?.ty == TokenKind::Comma {
					self.next_expected(TokenKind::Comma)?;
					arguments.push(self.parse_argument()?);
				}
				self.next_expected(TokenKind::Semicolon)?;

				Ok(ast::Statement::Barrier(arguments))
			}
			_ => Ok(ast::Statement::QuantumOperator(
				self.parse_quantum_operator()?,
			)),
		}
	}

	fn parse_declaration(&mut self) -> Result<ast::Declaration, Error> {
		let (ty, location) = match self.next()? {
			Token {
				ty: TokenKind::Qreg,
				location,
			} => (ast::DeclarationType::Qubit, location),
			Token {
				ty: TokenKind::Creg,
				location,
			} => (ast::DeclarationType::Bit, location),
			token => {
				let location = token.location.clone();
				return Err(Error::new(ErrorKind::UnexpectedToken(token), location));
			}
		};

		let name = self.parse_id()?;
		self.next_expected(TokenKind::OpeningSquareBracket)?;
		let size = self.parse_integer()?;
		self.next_expected(TokenKind::ClosingSquareBracket)?;
		self.next_expected(TokenKind::Semicolon)?;

		Ok(ast::Declaration {
			ty,
			name,
			size,
			location,
		})
	}

	fn parse_gate_declaration(&mut self) -> Result<ast::GateDeclaration, Error> {
		let location = self.next_expected(TokenKind::Gate)?.location;
		let name = self.parse_id()?;
		let params: Vec<ast::Id> = if self.peek_expected()?.ty == TokenKind::OpeningBracket {
			self.next_expected(TokenKind::OpeningBracket)?;
			let params = if let TokenKind::Id(_) = self.peek_expected()?.ty {
				self.parse_id_list()?
			} else {
				Vec::new()
			};

			self.next_expected(TokenKind::ClosingBracket)?;
			params
		} else {
			Vec::new()
		};

		let qargs = self.parse_id_list()?;

		self.next_expected(TokenKind::OpeningCurlyBracket)?;
		let mut quantum_operators: Vec<ast::GateOperation> = Vec::new();
		while self.peek_expected()?.ty != TokenKind::ClosingCurlyBracket {
			quantum_operators.push(self.parse_gate_operation()?);
		}
		self.next_expected(TokenKind::ClosingCurlyBracket)?;

		Ok(ast::GateDeclaration {
			name,
			params,
			qargs,
			quantum_operators,
			location,
		})
	}

	fn parse_gate_operation(&mut self) -> Result<ast::GateOperation, Error> {
		if self.peek_expected()?.ty == TokenKind::Barrier {
			self.next_expected(TokenKind::Barrier)?;
			let qargs = self.parse_id_list()?;
			self.next_expected(TokenKind::Semicolon)?;

			Ok(ast::GateOperation::Barrier(qargs))
		} else {
			Ok(ast::GateOperation::UnitaryOperator(
				self.parse_unitary_operator()?,
			))
		}
	}

	fn parse_opaque(&mut self) -> Result<ast::Opaque, Error> {
		let location = self.next_expected(TokenKind::Opaque)?.location;
		let name = self.parse_id()?;
		let params: Vec<ast::Id> = if self.peek_expected()?.ty == TokenKind::OpeningBracket {
			self.next_expected(TokenKind::OpeningBracket)?;
			let params = if let TokenKind::Id(_) = self.peek_expected()?.ty {
				self.parse_id_list()?
			} else {
				Vec::new()
			};

			self.next_expected(TokenKind::ClosingBracket)?;
			params
		} else {
			Vec::new()
		};

		let qargs = self.parse_id_list()?;
		self.next_expected(TokenKind::Semicolon)?;
		Ok(ast::Opaque {
			location,
			name,
			params,
			qargs,
		})
	}

	fn parse_quantum_operator(&mut self) -> Result<ast::QuantumOperator, Error> {
		match self.peek_expected()?.ty {
			TokenKind::Measure => {
				let location = self.next_expected(TokenKind::Measure)?.location;
				let source = self.parse_argument()?;
				self.next_expected(TokenKind::Arrow)?;
				let target = self.parse_argument()?;
				self.next_expected(TokenKind::Semicolon)?;
				Ok(ast::QuantumOperator::Measure {
					source,
					target,
					location,
				})
			}
			TokenKind::Reset => {
				let location = self.next_expected(TokenKind::Reset)?.location;
				let target = self.parse_argument()?;
				self.next_expected(TokenKind::Semicolon)?;
				Ok(ast::QuantumOperator::Reset { target, location })
			}
			_ => Ok(ast::QuantumOperator::UnitaryOperator(
				self.parse_unitary_operator()?,
			)),
		}
	}

	fn parse_unitary_operator(&mut self) -> Result<ast::UnitaryOperator, Error> {
		match self.next()? {
			Token {
				ty: TokenKind::U,
				location,
			} => {
				self.next_expected(TokenKind::OpeningBracket)?;
				let theta = self.parse_expression()?;
				self.next_expected(TokenKind::Comma)?;
				let phi = self.parse_expression()?;
				self.next_expected(TokenKind::Comma)?;
				let lambda = self.parse_expression()?;
				self.next_expected(TokenKind::ClosingBracket)?;
				let target = self.parse_argument()?;
				self.next_expected(TokenKind::Semicolon)?;
				Ok(ast::UnitaryOperator::U {
					theta,
					phi,
					lambda,
					target,
					location,
				})
			}
			Token {
				ty: TokenKind::Cx,
				location,
			} => {
				let control = self.parse_argument()?;
				self.next_expected(TokenKind::Comma)?;
				let target = self.parse_argument()?;
				self.next_expected(TokenKind::Semicolon)?;
				Ok(ast::UnitaryOperator::Cx {
					control,
					target,
					location,
				})
			}
			Token {
				ty: TokenKind::Id(name),
				location,
			} => {
				let name = ast::Id {
					location: location.clone(),
					text: name,
				};
				let params: Vec<ast::Expression> =
					if self.peek_expected()?.ty == TokenKind::OpeningBracket {
						self.next_expected(TokenKind::OpeningBracket)?;
						let mut params = Vec::new();
						while self.peek_expected()?.ty != TokenKind::ClosingBracket {
							params.push(self.parse_expression()?);
							if self.peek_expected()?.ty != TokenKind::Comma {
								break;
							}
							self.next_expected(TokenKind::Comma)?;
						}

						self.next_expected(TokenKind::ClosingBracket)?;
						params
					} else {
						Vec::new()
					};

				let mut qargs = Vec::new();
				qargs.push(self.parse_argument()?);
				while self.peek_expected()?.ty == TokenKind::Comma {
					self.next_expected(TokenKind::Comma)?;
					qargs.push(self.parse_argument()?);
				}
				self.next_expected(TokenKind::Semicolon)?;

				Ok(ast::UnitaryOperator::CustomGate {
					name,
					params,
					qargs,
					location,
				})
			}
			token => {
				let location = token.location.clone();
				Err(Error::new(ErrorKind::UnexpectedToken(token), location))
			}
		}
	}

	fn parse_argument(&mut self) -> Result<ast::Argument, Error> {
		let name = self.parse_id()?;
		if self.peek().is_some() && self.peek().unwrap().ty == TokenKind::OpeningSquareBracket {
			self.next_expected(TokenKind::OpeningSquareBracket)?;
			let index = self.parse_integer()?;
			self.next_expected(TokenKind::ClosingSquareBracket)?;
			Ok(ast::Argument::Indexed { name, index })
		} else {
			Ok(ast::Argument::Named { name })
		}
	}

	// does not contain + - / * outside of ()
	fn parse_trivial_expression(&mut self) -> Result<ast::Expression, Error> {
		let left = match self.peek_expected()?.ty {
			TokenKind::Sin
			| TokenKind::Cos
			| TokenKind::Tan
			| TokenKind::Exp
			| TokenKind::Ln
			| TokenKind::Sqrt => {
				let Token { ty, location } = self.next()?;
				let operator = match ty {
					TokenKind::Sin => ast::UnaryOperator::Sin,
					TokenKind::Cos => ast::UnaryOperator::Cos,
					TokenKind::Tan => ast::UnaryOperator::Tan,
					TokenKind::Exp => ast::UnaryOperator::Exp,
					TokenKind::Ln => ast::UnaryOperator::Ln,
					TokenKind::Sqrt => ast::UnaryOperator::Sqrt,
					_ => unreachable!(),
				};
				self.next_expected(TokenKind::OpeningBracket)?;
				let argument = self.parse_expression()?;
				self.next_expected(TokenKind::ClosingBracket)?;
				ast::Expression::UnaryOperator {
					operator,
					argument: Box::new(argument),
					location,
				}
			}
			TokenKind::Minus => {
				let location = self.next_expected(TokenKind::Minus)?.location;
				ast::Expression::UnaryOperator {
					operator: ast::UnaryOperator::Negation,
					argument: Box::new(self.parse_expression()?),
					location,
				}
			}
			TokenKind::OpeningBracket => {
				self.next_expected(TokenKind::OpeningBracket)?;
				let expression = self.parse_expression()?;
				self.next_expected(TokenKind::ClosingBracket)?;
				expression
			}
			TokenKind::Real(_) => {
				let Ok(Token {
					ty: TokenKind::Real(value),
					location,
				}) = self.next()
				else {
					unreachable!()
				};

				ast::Expression::Real { value, location }
			}
			TokenKind::Nninteger(_) => {
				let Ok(Token {
					ty: TokenKind::Nninteger(value),
					location,
				}) = self.next()
				else {
					unreachable!()
				};

				ast::Expression::Integer { value, location }
			}
			TokenKind::Id(_) => ast::Expression::Id(self.parse_id()?),
			TokenKind::Pi => {
				let location = self.next_expected(TokenKind::Pi)?.location;
				ast::Expression::Pi { location }
			}
			_ => {
				let token = self.next()?;
				let location = token.location.clone();
				return Err(Error::new(ErrorKind::UnexpectedToken(token), location));
			}
		};

		if self.peek().is_some() && self.peek().unwrap().ty == TokenKind::Pow {
			let location = self.next_expected(TokenKind::Pow)?.location;
			let right = self.parse_trivial_expression()?;
			Ok(ast::Expression::BinaryOperator {
				operator: ast::BinaryOperator::Power,
				left: Box::new(left),
				right: Box::new(right),
				location,
			})
		} else {
			Ok(left)
		}
	}

	// does not contain * / outside of ()
	fn parse_mul_div_expression(&mut self) -> Result<ast::Expression, Error> {
		let mut expression = self.parse_trivial_expression()?;
		while let Some(Token {
			ty: TokenKind::Mul, ..
		})
		| Some(Token {
			ty: TokenKind::Div, ..
		}) = self.peek()
		{
			let Token { ty, location } = self.next()?;
			let operator = match ty {
				TokenKind::Mul => ast::BinaryOperator::Multiplication,
				TokenKind::Div => ast::BinaryOperator::Division,
				_ => unreachable!(),
			};
			expression = ast::Expression::BinaryOperator {
				operator,
				left: Box::new(expression),
				right: Box::new(self.parse_trivial_expression()?),
				location,
			}
		}

		Ok(expression)
	}

	fn parse_expression(&mut self) -> Result<ast::Expression, Error> {
		let mut expression = self.parse_mul_div_expression()?;
		while let Some(Token {
			ty: TokenKind::Plus,
			..
		})
		| Some(Token {
			ty: TokenKind::Minus,
			..
		}) = self.peek()
		{
			let Token { ty, location } = self.next()?;
			let operator = match ty {
				TokenKind::Plus => ast::BinaryOperator::Addition,
				TokenKind::Minus => ast::BinaryOperator::Subtraction,
				_ => unreachable!(),
			};
			expression = ast::Expression::BinaryOperator {
				operator,
				left: Box::new(expression),
				right: Box::new(self.parse_mul_div_expression()?),
				location,
			}
		}

		Ok(expression)
	}

	fn parse_id_list(&mut self) -> Result<Vec<ast::Id>, Error> {
		let mut ids: Vec<ast::Id> = Vec::new();
		ids.push(self.parse_id()?);
		while self.peek_expected()?.ty == TokenKind::Comma {
			self.next_expected(TokenKind::Comma)?;
			ids.push(self.parse_id()?);
		}
		Ok(ids)
	}

	fn parse_id(&mut self) -> Result<ast::Id, Error> {
		match self.next()? {
			Token {
				ty: TokenKind::Id(text),
				location,
			} => Ok(ast::Id { text, location }),
			token => {
				let location = token.location.clone();
				Err(Error::new(ErrorKind::UnexpectedToken(token), location))
			}
		}
	}

	fn parse_integer(&mut self) -> Result<u64, Error> {
		match self.next()? {
			Token {
				ty: TokenKind::Nninteger(integer),
				..
			} => Ok(integer),
			token => {
				let location = token.location.clone();
				Err(Error::new(ErrorKind::UnexpectedToken(token), location))
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::rc::Rc;

	use crate::ast::Id;

	use super::*;

	#[test]
	fn test_parse_no_param_opaque() {
		const IN1: &str = "opaque name q1, q2;";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", IN1).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};
		let goal = ast::Opaque {
			name: ast::Id {
				text: String::from("name"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 8,
				},
			},
			params: Vec::new(),
			qargs: vec![
				ast::Id {
					text: String::from("q1"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 13,
					},
				},
				ast::Id {
					text: String::from("q2"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 17,
					},
				},
			],
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 1,
			},
		};
		assert_eq!(token_stream.parse_opaque(), Ok(goal));
	}

	#[test]
	fn test_parse_no_param_bracket_opaque() {
		const IN1: &str = "opaque name () q1, q2;";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", IN1).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};
		let goal = ast::Opaque {
			name: ast::Id {
				text: String::from("name"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 8,
				},
			},
			params: Vec::new(),
			qargs: vec![
				ast::Id {
					text: String::from("q1"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 16,
					},
				},
				ast::Id {
					text: String::from("q2"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 20,
					},
				},
			],
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 1,
			},
		};
		assert_eq!(token_stream.parse_opaque(), Ok(goal));
	}

	#[test]
	fn test_parse_opaque() {
		const IN1: &str = "opaque name (p1, p2, p3) q1, q2;";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", IN1).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};
		let goal = ast::Opaque {
			name: ast::Id {
				text: String::from("name"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 8,
				},
			},
			params: vec![
				ast::Id {
					text: String::from("p1"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 14,
					},
				},
				ast::Id {
					text: String::from("p2"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 18,
					},
				},
				ast::Id {
					text: String::from("p3"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 22,
					},
				},
			],
			qargs: vec![
				ast::Id {
					text: String::from("q1"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 26,
					},
				},
				ast::Id {
					text: String::from("q2"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 30,
					},
				},
			],
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 1,
			},
		};
		assert_eq!(token_stream.parse_opaque(), Ok(goal));
	}

	#[test]
	fn test_parse_no_param_nop_gate() {
		const IN1: &str = "gate name q1, q2 {}";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", IN1).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};
		let goal = ast::GateDeclaration {
			name: ast::Id {
				text: String::from("name"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 6,
				},
			},
			params: Vec::new(),
			qargs: vec![
				ast::Id {
					text: String::from("q1"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 11,
					},
				},
				ast::Id {
					text: String::from("q2"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 15,
					},
				},
			],
			quantum_operators: Vec::new(),
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 1,
			},
		};
		assert_eq!(token_stream.parse_gate_declaration(), Ok(goal));
	}

	#[test]
	fn test_parse_no_param_bracket_nop_gate() {
		const IN1: &str = "gate name () q1, q2 {}";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", IN1).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};
		let goal = ast::GateDeclaration {
			name: ast::Id {
				text: String::from("name"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 6,
				},
			},
			params: Vec::new(),
			qargs: vec![
				ast::Id {
					text: String::from("q1"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 14,
					},
				},
				ast::Id {
					text: String::from("q2"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 18,
					},
				},
			],
			quantum_operators: Vec::new(),
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 1,
			},
		};
		assert_eq!(token_stream.parse_gate_declaration(), Ok(goal));
	}

	#[test]
	fn test_parse_nop_gate() {
		const IN1: &str = "gate name (p1, p2, p3) q1, q2 {}";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", IN1).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};
		let goal = ast::GateDeclaration {
			name: ast::Id {
				text: String::from("name"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 6,
				},
			},
			params: vec![
				ast::Id {
					text: String::from("p1"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 12,
					},
				},
				ast::Id {
					text: String::from("p2"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 16,
					},
				},
				ast::Id {
					text: String::from("p3"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 20,
					},
				},
			],
			qargs: vec![
				ast::Id {
					text: String::from("q1"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 24,
					},
				},
				ast::Id {
					text: String::from("q2"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 28,
					},
				},
			],
			quantum_operators: Vec::new(),
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 1,
			},
		};
		assert_eq!(token_stream.parse_gate_declaration(), Ok(goal));
	}

	#[test]
	fn test_gate_operation_barrier() {
		const IN1: &str = "barrier q1, q2;";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", IN1).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};
		let goal = ast::GateOperation::Barrier(vec![
			ast::Id {
				text: String::from("q1"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 9,
				},
			},
			ast::Id {
				text: String::from("q2"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 13,
				},
			},
		]);
		assert_eq!(token_stream.parse_gate_operation(), Ok(goal));
	}

	#[test]
	fn test_gate_declaration() {
		const INPUT: &str =
			"gate swap_r (p) q1, q2 { cx q1, q2; cx q2 ,q1; cx q1, q2; u (p, 0, 0) q2;}";
		let frontend = OpenQasm2Frontend::<()>::default();
		let tokens = frontend.tokenize("file", INPUT).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};

		let goal = ast::GateDeclaration {
			name: ast::Id {
				text: String::from("swap_r"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 6,
				},
			},
			params: vec![ast::Id {
				text: String::from("p"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 14,
				},
			}],
			qargs: vec![
				ast::Id {
					text: String::from("q1"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 17,
					},
				},
				ast::Id {
					text: String::from("q2"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 21,
					},
				},
			],
			quantum_operators: vec![
				ast::GateOperation::UnitaryOperator(ast::UnitaryOperator::Cx {
					control: ast::Argument::Named {
						name: ast::Id {
							text: String::from("q1"),
							location: Location {
								file: Rc::from("file"),
								row: 1,
								col: 29,
							},
						},
					},
					target: ast::Argument::Named {
						name: Id {
							text: String::from("q2"),
							location: Location {
								file: Rc::from("file"),
								row: 1,
								col: 33,
							},
						},
					},
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 26,
					},
				}),
				ast::GateOperation::UnitaryOperator(ast::UnitaryOperator::Cx {
					control: ast::Argument::Named {
						name: ast::Id {
							text: String::from("q2"),
							location: Location {
								file: Rc::from("file"),
								row: 1,
								col: 40,
							},
						},
					},
					target: ast::Argument::Named {
						name: ast::Id {
							text: String::from("q1"),
							location: Location {
								file: Rc::from("file"),
								row: 1,
								col: 44,
							},
						},
					},
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 37,
					},
				}),
				ast::GateOperation::UnitaryOperator(ast::UnitaryOperator::Cx {
					control: ast::Argument::Named {
						name: ast::Id {
							text: String::from("q1"),
							location: Location {
								file: Rc::from("file"),
								row: 1,
								col: 51,
							},
						},
					},
					target: ast::Argument::Named {
						name: ast::Id {
							text: String::from("q2"),
							location: Location {
								file: Rc::from("file"),
								row: 1,
								col: 55,
							},
						},
					},
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 48,
					},
				}),
				ast::GateOperation::UnitaryOperator(ast::UnitaryOperator::U {
					theta: ast::Expression::Id(Id {
						text: String::from("p"),
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 62,
						},
					}),
					phi: ast::Expression::Integer {
						value: 0,
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 65,
						},
					},
					lambda: ast::Expression::Integer {
						value: 0,
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 68,
						},
					},
					target: ast::Argument::Named {
						name: ast::Id {
							text: String::from("q2"),
							location: Location {
								file: Rc::from("file"),
								row: 1,
								col: 71,
							},
						},
					},
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 59,
					},
				}),
			],
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 1,
			},
		};

		assert_eq!(token_stream.parse_gate_declaration(), Ok(goal));
	}

	#[test]
	fn test_quantum_operator_measure() {
		const INPUT: &str = "measure q1 -> c1;";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", INPUT).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};

		let goal = ast::QuantumOperator::Measure {
			source: ast::Argument::Named {
				name: ast::Id {
					text: String::from("q1"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 9,
					},
				},
			},
			target: ast::Argument::Named {
				name: ast::Id {
					text: String::from("c1"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 15,
					},
				},
			},
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 1,
			},
		};

		assert_eq!(token_stream.parse_quantum_operator(), Ok(goal));
	}

	#[test]
	fn test_quantum_operator_reset() {
		const INPUT: &str = "reset q1;";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", INPUT).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};

		let goal = ast::QuantumOperator::Reset {
			target: ast::Argument::Named {
				name: ast::Id {
					text: String::from("q1"),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 7,
					},
				},
			},
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 1,
			},
		};

		assert_eq!(token_stream.parse_quantum_operator(), Ok(goal));
	}

	#[test]
	fn test_unitary_operator_custom_gate_no_param() {
		const INPUT: &str = "something q1, q2;";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", INPUT).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};

		let goal = ast::UnitaryOperator::CustomGate {
			name: ast::Id {
				text: String::from("something"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 1,
				},
			},
			params: Vec::new(),
			qargs: vec![
				ast::Argument::Named {
					name: ast::Id {
						text: String::from("q1"),
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 11,
						},
					},
				},
				ast::Argument::Named {
					name: ast::Id {
						text: String::from("q2"),
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 15,
						},
					},
				},
			],
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 1,
			},
		};

		assert_eq!(token_stream.parse_unitary_operator(), Ok(goal))
	}

	#[test]
	fn test_unitary_operator_custom_gate_no_param_brackets() {
		const INPUT: &str = "something () q1, q2;";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", INPUT).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};

		let goal = ast::UnitaryOperator::CustomGate {
			name: ast::Id {
				text: String::from("something"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 1,
				},
			},
			params: Vec::new(),
			qargs: vec![
				ast::Argument::Named {
					name: ast::Id {
						text: String::from("q1"),
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 14,
						},
					},
				},
				ast::Argument::Named {
					name: ast::Id {
						text: String::from("q2"),
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 18,
						},
					},
				},
			],
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 1,
			},
		};

		assert_eq!(token_stream.parse_unitary_operator(), Ok(goal))
	}

	#[test]
	fn test_unitary_operator_custom_gate() {
		const INPUT: &str = "something (1, 2) q1, q2;";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", INPUT).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};

		let goal = ast::UnitaryOperator::CustomGate {
			name: ast::Id {
				text: String::from("something"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 1,
				},
			},
			params: vec![
				ast::Expression::Integer {
					value: 1,
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 12,
					},
				},
				ast::Expression::Integer {
					value: 2,
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 15,
					},
				},
			],
			qargs: vec![
				ast::Argument::Named {
					name: ast::Id {
						text: String::from("q1"),
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 18,
						},
					},
				},
				ast::Argument::Named {
					name: ast::Id {
						text: String::from("q2"),
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 22,
						},
					},
				},
			],
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 1,
			},
		};

		assert_eq!(token_stream.parse_unitary_operator(), Ok(goal))
	}

	#[test]
	fn test_argument() {
		const INPUT1: &str = "q1";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", INPUT1).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};

		let goal = ast::Argument::Named {
			name: ast::Id {
				text: String::from("q1"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 1,
				},
			},
		};
		assert_eq!(token_stream.parse_argument(), Ok(goal));

		const INPUT2: &str = "q[1]";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", INPUT2).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};

		let goal = ast::Argument::Indexed {
			name: ast::Id {
				text: String::from("q"),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 1,
				},
			},
			index: 1,
		};
		assert_eq!(token_stream.parse_argument(), Ok(goal));
	}

	#[test]
	fn test_expression() {
		const INPUT: &str = "a ^ 2 + 1.4 ^ (12 + 3) * 5 - 2 / -1";
		let frontend = OpenQasm2Frontend::<()>::default();

		let tokens = frontend.tokenize("file", INPUT).unwrap();
		let mut token_stream = TokenStream {
			tokens: tokens.0.into_iter().peekable(),
		};

		let goal = ast::Expression::BinaryOperator {
			operator: ast::BinaryOperator::Subtraction,
			left: Box::new(ast::Expression::BinaryOperator {
				operator: ast::BinaryOperator::Addition,
				left: Box::new(ast::Expression::BinaryOperator {
					operator: ast::BinaryOperator::Power,
					left: Box::new(ast::Expression::Id(Id {
						text: String::from("a"),
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 1,
						},
					})),
					right: Box::new(ast::Expression::Integer {
						value: 2,
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 5,
						},
					}),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 3,
					},
				}),
				right: Box::new(ast::Expression::BinaryOperator {
					operator: ast::BinaryOperator::Multiplication,
					left: Box::new(ast::Expression::BinaryOperator {
						operator: ast::BinaryOperator::Power,
						left: Box::new(ast::Expression::Real {
							value: 1.4,
							location: Location {
								file: Rc::from("file"),
								row: 1,
								col: 9,
							},
						}),
						right: Box::new(ast::Expression::BinaryOperator {
							operator: ast::BinaryOperator::Addition,
							left: Box::new(ast::Expression::Integer {
								value: 12,
								location: Location {
									file: Rc::from("file"),
									row: 1,
									col: 16,
								},
							}),
							right: Box::new(ast::Expression::Integer {
								value: 3,
								location: Location {
									file: Rc::from("file"),
									row: 1,
									col: 21,
								},
							}),
							location: Location {
								file: Rc::from("file"),
								row: 1,
								col: 19,
							},
						}),
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 13,
						},
					}),
					right: Box::new(ast::Expression::Integer {
						value: 5,
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 26,
						},
					}),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 24,
					},
				}),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 7,
				},
			}),
			right: Box::new(ast::Expression::BinaryOperator {
				operator: ast::BinaryOperator::Division,
				left: Box::new(ast::Expression::Integer {
					value: 2,
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 30,
					},
				}),
				right: Box::new(ast::Expression::UnaryOperator {
					operator: ast::UnaryOperator::Negation,
					argument: Box::new(ast::Expression::Integer {
						value: 1,
						location: Location {
							file: Rc::from("file"),
							row: 1,
							col: 35,
						},
					}),
					location: Location {
						file: Rc::from("file"),
						row: 1,
						col: 34,
					},
				}),
				location: Location {
					file: Rc::from("file"),
					row: 1,
					col: 32,
				},
			}),
			location: Location {
				file: Rc::from("file"),
				row: 1,
				col: 28,
			},
		};

		assert_eq!(token_stream.parse_expression(), Ok(goal));
	}

	#[test]
	fn full_program() {
		const SRC: &str = r#"
			OPENQASM 2.0;
			// a comment
			qreg q[2];
			creg c[2];
			gate mygate(theta) a, b { U(theta / 2, 0, pi) a; CX a, b; }
			if (c == 3) reset q[1];
			barrier q;
			measure q -> c;
		"#;
		let frontend = OpenQasm2Frontend::<()>::default();
		let tokens = frontend.tokenize("file", SRC).unwrap();
		let ast = frontend.parse(tokens.0);

		let goal = ast::Program {
			version: 2.0,
			program: vec![
				ast::Statement::Declaration(ast::Declaration {
					ty: ast::DeclarationType::Qubit,
					name: ast::Id {
						text: String::from("q"),
						location: Location {
							file: Rc::from("file"),
							row: 4,
							col: 9,
						},
					},
					size: 2,
					location: Location {
						file: Rc::from("file"),
						row: 4,
						col: 4,
					},
				}),
				ast::Statement::Declaration(ast::Declaration {
					ty: ast::DeclarationType::Bit,
					name: ast::Id {
						text: String::from("c"),
						location: Location {
							file: Rc::from("file"),
							row: 5,
							col: 9,
						},
					},
					size: 2,
					location: Location {
						file: Rc::from("file"),
						row: 5,
						col: 4,
					},
				}),
				ast::Statement::GateDeclaration(ast::GateDeclaration {
					name: ast::Id {
						text: String::from("mygate"),
						location: Location {
							file: Rc::from("file"),
							row: 6,
							col: 9,
						},
					},
					params: vec![ast::Id {
						text: String::from("theta"),
						location: Location {
							file: Rc::from("file"),
							row: 6,
							col: 16,
						},
					}],
					qargs: vec![
						ast::Id {
							text: String::from("a"),
							location: Location {
								file: Rc::from("file"),
								row: 6,
								col: 23,
							},
						},
						ast::Id {
							text: String::from("b"),
							location: Location {
								file: Rc::from("file"),
								row: 6,
								col: 26,
							},
						},
					],
					quantum_operators: vec![
						ast::GateOperation::UnitaryOperator(ast::UnitaryOperator::U {
							theta: ast::Expression::BinaryOperator {
								operator: ast::BinaryOperator::Division,
								left: Box::new(ast::Expression::Id(ast::Id {
									text: String::from("theta"),
									location: Location {
										file: Rc::from("file"),
										row: 6,
										col: 32,
									},
								})),
								right: Box::new(ast::Expression::Integer {
									value: 2,
									location: Location {
										file: Rc::from("file"),
										row: 6,
										col: 40,
									},
								}),
								location: Location {
									file: Rc::from("file"),
									row: 6,
									col: 38,
								},
							},
							phi: ast::Expression::Integer {
								value: 0,
								location: Location {
									file: Rc::from("file"),
									row: 6,
									col: 43,
								},
							},
							lambda: ast::Expression::Pi {
								location: Location {
									file: Rc::from("file"),
									row: 6,
									col: 46,
								},
							},
							target: ast::Argument::Named {
								name: ast::Id {
									text: String::from("a"),
									location: Location {
										file: Rc::from("file"),
										row: 6,
										col: 50,
									},
								},
							},
							location: Location {
								file: Rc::from("file"),
								row: 6,
								col: 30,
							},
						}),
						ast::GateOperation::UnitaryOperator(ast::UnitaryOperator::Cx {
							control: ast::Argument::Named {
								name: ast::Id {
									text: String::from("a"),
									location: Location {
										file: Rc::from("file"),
										row: 6,
										col: 56,
									},
								},
							},
							target: ast::Argument::Named {
								name: ast::Id {
									text: String::from("b"),
									location: Location {
										file: Rc::from("file"),
										row: 6,
										col: 59,
									},
								},
							},
							location: Location {
								file: Rc::from("file"),
								row: 6,
								col: 53,
							},
						}),
					],
					location: Location {
						file: Rc::from("file"),
						row: 6,
						col: 4,
					},
				}),
				ast::Statement::If {
					id: ast::Id {
						text: String::from("c"),
						location: Location {
							file: Rc::from("file"),
							row: 7,
							col: 8,
						},
					},
					integer: 3,
					quantum_operator: ast::QuantumOperator::Reset {
						target: ast::Argument::Indexed {
							name: ast::Id {
								text: String::from("q"),
								location: Location {
									file: Rc::from("file"),
									row: 7,
									col: 22,
								},
							},
							index: 1,
						},
						location: Location {
							file: Rc::from("file"),
							row: 7,
							col: 16,
						},
					},
					location: Location {
						file: Rc::from("file"),
						row: 7,
						col: 4,
					},
				},
				ast::Statement::Barrier(vec![ast::Argument::Named {
					name: ast::Id {
						text: String::from("q"),
						location: Location {
							file: Rc::from("file"),
							row: 8,
							col: 12,
						},
					},
				}]),
				ast::Statement::QuantumOperator(ast::QuantumOperator::Measure {
					source: ast::Argument::Named {
						name: ast::Id {
							text: String::from("q"),
							location: Location {
								file: Rc::from("file"),
								row: 9,
								col: 12,
							},
						},
					},
					target: ast::Argument::Named {
						name: ast::Id {
							text: String::from("c"),
							location: Location {
								file: Rc::from("file"),
								row: 9,
								col: 17,
							},
						},
					},
					location: Location {
						file: Rc::from("file"),
						row: 9,
						col: 4,
					},
				}),
			],
		};

		assert_eq!(ast, Ok(goal));
	}
}
