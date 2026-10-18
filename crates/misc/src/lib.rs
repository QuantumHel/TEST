pub enum IterEnum2<T, A: Iterator<Item = T>, B: Iterator<Item = T>> {
	Iter1(A),
	Iter2(B),
}

impl<T, A: Iterator<Item = T>, B: Iterator<Item = T>> Iterator for IterEnum2<T, A, B> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Iter1(iter) => iter.next(),
			Self::Iter2(iter) => iter.next(),
		}
	}
}
