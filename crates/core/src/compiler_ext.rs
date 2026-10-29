use std::marker::PhantomData;

use crate::Compiler;

pub struct Chain<First: Compiler<I, IR, D>, Last: Compiler<IR, O, D>, I, IR, O, D> {
	pub first: First,
	pub last: Last,
	pub _phantom: PhantomData<(I, IR, O, D)>,
}

impl<First: Compiler<I, IR, D>, Last: Compiler<IR, O, D>, I, IR, O, D> Compiler<I, O, D>
	for Chain<First, Last, I, IR, O, D>
{
	fn compile(&self, input: I, device: &D) -> O {
		let ir = self.first.compile(input, device);
		self.last.compile(ir, device)
	}
}

pub trait CompilerExt<I, IR, D>: Compiler<I, IR, D> {
	fn chain<Next: Compiler<IR, O, D>, O>(self, next: Next) -> Chain<Self, Next, I, IR, O, D> {
		Chain {
			first: self,
			last: next,
			_phantom: PhantomData,
		}
	}
}

impl<C: Compiler<I, IR, D>, I, IR, D> CompilerExt<I, IR, D> for C {}
