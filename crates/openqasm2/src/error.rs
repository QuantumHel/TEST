use std::{fmt, rc::Rc};

use crate::tokenizer::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
	pub kind: ErrorKind,
	pub location: Location,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Location {
	pub file: Rc<str>,
	pub row: usize,
	pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
	/// A character that can't start any token.
	UnexpectedChar(char),
	/// A lone `=`; only `==` is a token in QASM 2.0.
	UnexpectedEquals,
	/// A string literal ran to end of file without a closing `"`.
	UnterminatedString,
	/// A newline inside a string literal.
	NewlineInString,
	/// `007` — `nninteger` is `[1-9]+[0-9]*|0`, so no leading zeros.
	LeadingZero,
	/// An integer literal that doesn't fit in an `i64`.
	IntegerOverflow,
	/// A bare `.` with no digits on either side.
	MalformedReal,
	/// An identifier starting with an uppercase letter that isn't `OPENQASM`,
	/// `U` or `CX`. Ids must match `[a-z][A-Za-z0-9_]*`.
	InvalidIdentifier(String),
	/// include not followed by string
	MissingIncludeString,
	/// Unable to read a source file
	UnableToReadFile(String),
	/// The program end of program
	UnexpectedEndOfProgram,
	UnexpectedToken(Token),
	MissingOpenQasmVersionSpecifier,
	WrongOpenQasmVersion(f64),
	UndefinedIdentifier(String),
	Custom(String),
	IndexOutOfBounds {
		index: u64,
		size: u64,
	},
	NotYetImplemented(&'static str),
}

impl Error {
	pub(crate) const fn new(kind: ErrorKind, location: Location) -> Self {
		Self { kind, location }
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}:{}:{}: {}",
			self.location.file, self.location.row, self.location.col, self.kind
		)
	}
}

impl fmt::Display for ErrorKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::UnexpectedChar(c) => write!(f, "unexpected character `{c}`"),
			Self::UnexpectedEquals => write!(f, "unexpected `=`, did you mean `==`?"),
			Self::UnterminatedString => write!(f, "unterminated string literal"),
			Self::NewlineInString => write!(f, "newline in string literal"),
			Self::LeadingZero => write!(f, "integer literals may not have leading zeros"),
			Self::IntegerOverflow => write!(f, "integer literal does not fit in an i64"),
			Self::MalformedReal => write!(f, "malformed real literal"),
			Self::InvalidIdentifier(s) => {
				write!(
					f,
					"invalid identifier `{s}`, identifiers must start with a lowercase letter"
				)
			}
			Self::MissingIncludeString => {
				write!(f, "include keword needs to be followed by file path")
			}
			Self::UnableToReadFile(s) => write!(f, "Unable to read source file '{s}'"),
			Self::UnexpectedEndOfProgram => write!(f, "The program end of program"),
			Self::UnexpectedToken(token) => write!(f, "Unexpected token '{token:?}'"),
			Self::MissingOpenQasmVersionSpecifier => {
				write!(f, "Programs should star with 'OPENQASM 2.0;'.")
			}
			Self::WrongOpenQasmVersion(version) => {
				write!(
					f,
					"This compiler only supports open qasm version 2.0 not {version}."
				)
			}
			Self::UndefinedIdentifier(id) => {
				write!(f, "Identifier '{}' is not defined.", id)
			}
			Self::Custom(string) => write!(f, "{string}"),
			Self::IndexOutOfBounds { index, size } => write!(
				f,
				"Index {index} is out of bounds for register of size {size}"
			),
			Self::NotYetImplemented(thing) => write!(
				f,
				"The feature {} is not yet implemented. Only features needed for specific experiments are implemented.",
				thing
			),
		}
	}
}

impl std::error::Error for Error {}
