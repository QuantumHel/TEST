pub mod connectivity;
mod disjoint_set_forest;

pub mod prelude {
	pub use super::Compiler;
	pub use super::CompilerExt;
	pub use super::CompilerStack;
	pub use super::connectivity::{
		Connectivity, Edge, Graph, Node, Subedge, Subgraph, steiner_tree,
	};
}

pub trait Compiler: Sized {
	type Input;
	type Output;

	fn compile(&self, input: Self::Input) -> Self::Output;
}

impl<C1: Compiler, C2: Compiler> Compiler for (C1, C2) {
	type Input = (C1::Input, C2::Input);
	type Output = (C1::Output, C2::Output);

	fn compile(&self, input: Self::Input) -> Self::Output {
		(self.0.compile(input.0), self.1.compile(input.1))
	}
}

pub struct CompilerStack<First: Compiler<Output = IR>, IR, Second: Compiler<Input = IR>>(
	pub First,
	pub Second,
);

impl<First: Compiler<Output = IR>, IR, Second: Compiler<Input = IR>> Compiler
	for CompilerStack<First, IR, Second>
{
	type Input = First::Input;
	type Output = Second::Output;

	fn compile(&self, input: Self::Input) -> Self::Output {
		self.1.compile(self.0.compile(input))
	}
}

pub trait CompilerExt: Compiler + sealed::Sealed {
	fn stack<Second: Compiler<Input = Self::Output>>(
		self,
		second: Second,
	) -> CompilerStack<Self, Self::Output, Second> {
		CompilerStack(self, second)
	}
}

impl<T: Compiler> sealed::Sealed for T {}

impl<T: Compiler + sealed::Sealed> CompilerExt for T {}

mod sealed {
	pub trait Sealed {}
}
