fn main() {
	let mut self_ = vec![0, 1, 2];
	let mut rhs = vec![0, 1, 2, 23];

	let a = &mut rhs.drain(self_.len()..);
	self_.extend(a);
}
