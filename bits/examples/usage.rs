use bits::Bits;

fn main() {
	let mut bits1 = Bits::new();
	bits1.set(2, true);
	bits1.set(100, true);

	let mut bits2 = Bits::new();
	bits2.set(4, true);
	bits2.set(100, true);
	bits2.set(98, true);

	println!("{bits1:b}");
	println!("{bits2:b}");
	println!("{:b}", bits1 ^ bits2);
}
