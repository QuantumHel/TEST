use test_core::prelude::*;

struct Compiler1;

impl Compiler for Compiler1 {
	type Input = u32;
	type Output = usize;

	fn compile(&self, input: Self::Input) -> Self::Output {
		input as usize
	}
}

struct Compiler2;

impl Compiler for Compiler2 {
	type Input = usize;
	type Output = f64;

	fn compile(&self, input: Self::Input) -> Self::Output {
		input as f64
	}
}

fn main() {
	let left = Compiler1;
	let right = Compiler2;
	let _parallel = (left, right);

	let first = Compiler1;
	let second = Compiler2;
	let _stack = first.stack(second);
}
