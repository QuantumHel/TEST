mod ast;
mod error;
mod generate_ir;
mod parser;
mod tokenizer;
mod type_checker;

use std::{
	collections::{BTreeSet, HashMap},
	fs::read_to_string,
	io,
	num::NonZero,
	ops::{Deref, DerefMut},
	path::Path,
	rc::Rc,
};

use self::error::{Error, ErrorKind, Location};

#[derive(Debug)]
pub struct Redefinition;

pub struct VirtualOpenqasmFile<T: OpenQasm2IR> {
	text: String,
	// make into hashmap
	opaque_functions: HashMap<String, Box<dyn OpaqueFunction<T>>>,
}

impl<T: OpenQasm2IR> Default for VirtualOpenqasmFile<T> {
	fn default() -> Self {
		Self {
			text: String::default(),
			opaque_functions: HashMap::default(),
		}
	}
}

impl<T: OpenQasm2IR> VirtualOpenqasmFile<T> {
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

/// Tracks the virtual files that the program used (they overwrite something)
#[derive(Default, Debug)]
struct VirtualFileOverrideList(BTreeSet<Rc<str>>);

impl Deref for VirtualFileOverrideList {
	type Target = BTreeSet<Rc<str>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for VirtualFileOverrideList {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub struct OpaqueFunctionDefinition {
	pub name: String,
	pub n_params: usize,
	pub n_qargs: NonZero<usize>,
}

pub trait OpaqueFunction<T: OpenQasm2IR>: 'static {
	fn definition(&self) -> OpaqueFunctionDefinition;

	/// qargs has qubits with fake indices. The only thing that these indices
	/// should be used for is checking which ones are the same and which not.
	///
	/// Returs an error str in the case of a type check fail
	fn type_check(&self, qargs: Vec<usize>) -> Result<(), &'static str>;

	/// Since we do not support numeric values yet this only takes qargs.
	fn insert_gate(&self, qargs: Vec<usize>, ir: &mut T);
}

pub struct OpenQasm2Frontend<T: OpenQasm2IR> {
	default_file: VirtualOpenqasmFile<T>,
	file_overrides: HashMap<String, VirtualOpenqasmFile<T>>,
	ignore_imports: bool,
}

impl<T: OpenQasm2IR> Default for OpenQasm2Frontend<T> {
	fn default() -> Self {
		Self {
			default_file: VirtualOpenqasmFile::default(),
			ignore_imports: false,
			file_overrides: HashMap::new(),
		}
	}
}

#[cfg(test)]
impl OpenQasm2IR for () {
	fn insert_cnot(&mut self, _: usize, _: usize) {
		panic!()
	}
}

/// A trait for a IR struct that can be created from Open QASM 2.0 as defined in
/// https://arxiv.org/abs/1707.03429
pub trait OpenQasm2IR: 'static {
	/// This is run on CX gate invocations. Importantly CX != cx
	fn insert_cnot(&mut self, control: usize, target: usize);

	//fn insert_u(&mut self, a: f64, b: f64, c: f64);
}

impl<T: OpenQasm2IR> OpenQasm2Frontend<T> {
	pub fn new() -> Self {
		Self::default()
	}

	/// `virtual_file`` is set as a default file that is always automatically
	/// imported at the start of the program. (even when ignore_imports is
	/// active)
	pub fn new_with_virtual_file(virtual_file: VirtualOpenqasmFile<T>) -> Self {
		Self {
			default_file: virtual_file,
			..Default::default()
		}
	}

	/// Ignores all other imports than potential default file given with
	/// [OpenQasm2Frontend::new_with_virtual_file].
	pub fn with_ignore_imports(self) -> Self {
		Self {
			ignore_imports: true,
			..self
		}
	}

	pub fn with_file_override(
		mut self,
		path: &'static str,
		file: VirtualOpenqasmFile<T>,
	) -> Result<Self, Redefinition> {
		if self.file_overrides.contains_key(path) {
			return Err(Redefinition);
		}

		self.file_overrides.insert(path.to_string(), file);
		Ok(self)
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
