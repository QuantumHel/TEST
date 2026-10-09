mod ast;
mod error;
mod generate_ir;
mod parser;
mod tokenizer;
mod type_checker;
mod virtual_file_override_list;

use std::{collections::HashMap, fs::read_to_string, io, num::NonZero, path::Path, rc::Rc};

use self::error::{Error, ErrorKind, Location};

#[derive(Debug)]
pub struct Redefinition;

pub struct VirtualOpenqasmFile<T> {
	text: String,
	// make into hashmap
	opaque_functions: HashMap<String, Box<dyn OpaqueFunction<T>>>,
}

impl<T> Default for VirtualOpenqasmFile<T> {
	fn default() -> Self {
		Self {
			text: String::default(),
			opaque_functions: HashMap::default(),
		}
	}
}

impl<T> VirtualOpenqasmFile<T> {
	pub fn add_text(&mut self, text: &str) {
		self.text += text;
	}

	pub fn add_text_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
		match read_to_string(path) {
			Ok(text) => {
				self.text += &text;
				Ok(())
			}
			Err(err) => Err(err),
		}
	}

	/// Adds an opaque function to open QASM that is handled according to the
	/// rust struct.
	pub fn add_opaque(&mut self, opaque: impl OpaqueFunction<T>) -> Result<(), Redefinition> {
		let name = opaque.definition().name.clone();
		if self.opaque_functions.contains_key(&name) {
			return Err(Redefinition);
		}

		self.opaque_functions.insert(name, Box::new(opaque));
		Ok(())
	}
}

pub struct OpaqueFunctionDefinition {
	pub name: String,
	pub n_params: usize,
	pub n_qargs: NonZero<usize>,
}

pub trait OpaqueFunction<T>: 'static {
	fn definition(&self) -> OpaqueFunctionDefinition;

	/// qargs has qubits with fake indices. The only thing that these indices
	/// should be used for is checking which ones are the same and which not.
	///
	/// Returs an error str in the case of a type check fail
	fn type_check(&self, qargs: Vec<usize>) -> Result<(), &'static str>;

	/// Since we do not support numeric values yet this only takes qargs.
	fn insert_gate(&self, qargs: Vec<usize>, ir: &mut T);
}

/// Corresponds to the build in CX gate in open QASM 2.0
pub trait OpenQasm2Cx<T>: 'static {
	fn insert_cnot(&self, control: usize, target: usize, ir: &mut T);
}

pub struct OpenQasm2Config<T> {
	/// When `default_file` is set, it is always automatically
	/// imported at the start of the program. (even when ignore_imports is
	/// active)
	pub default_file: Option<VirtualOpenqasmFile<T>>,
	pub cx: Option<Box<dyn OpenQasm2Cx<T>>>,
	/// Ignores all other imports than `default_file`.
	pub ignore_imports: bool,
}

/// An open QASM 2.0 frontend (converts source code to some IR).
///
/// The language is explained in https://arxiv.org/abs/1707.03429
pub struct OpenQasm2Frontend<T: 'static> {
	default_file: VirtualOpenqasmFile<T>,
	cx: Option<Box<dyn OpenQasm2Cx<T>>>,
	file_overrides: HashMap<String, VirtualOpenqasmFile<T>>,
	ignore_imports: bool,
}

impl<T> Default for OpenQasm2Frontend<T> {
	fn default() -> Self {
		Self {
			default_file: VirtualOpenqasmFile::default(),
			cx: None,
			ignore_imports: false,
			file_overrides: HashMap::new(),
		}
	}
}

impl<T: 'static> OpenQasm2Frontend<T> {
	pub fn new(config: OpenQasm2Config<T>) -> Self {
		Self {
			default_file: config.default_file.unwrap_or_default(),
			file_overrides: HashMap::default(),
			cx: config.cx,
			ignore_imports: config.ignore_imports,
		}
	}

	pub fn add_file_override(
		&mut self,
		path: &'static str,
		file: VirtualOpenqasmFile<T>,
	) -> Result<(), Redefinition> {
		if self.file_overrides.contains_key(path) {
			return Err(Redefinition);
		}

		self.file_overrides.insert(path.to_string(), file);
		Ok(())
	}

	pub fn compile_file<P: AsRef<Path>>(&self, path: P, ir: &mut T) -> Result<(), Error> {
		let Ok(src) = read_to_string(&path) else {
			let name = path.as_ref().to_string_lossy().to_string();
			return Err(Error::new(
				ErrorKind::UnableToReadFile(name.clone()),
				Location {
					file: Rc::from(name.as_str()),
					row: 1,
					col: 1,
				},
			));
		};
		let (tokens, override_list) =
			self.tokenize(&path.as_ref().to_string_lossy(), &src.to_owned())?;
		let ast = self.parse(tokens)?;
		self.type_check(&ast, &override_list)?;
		self.generate_ir(&ast, &override_list, ir)
	}

	pub fn compile_str(&self, src: &str, ir: &mut T) -> Result<(), Error> {
		let (tokens, override_list) = self.tokenize("main", src)?;
		let ast = self.parse(tokens)?;
		self.type_check(&ast, &override_list)?;
		self.generate_ir(&ast, &override_list, ir)
	}
}
