struct SetElement {
	parent: usize,
	rank: usize,
}

/// Set elements can only be indexes in [0, n). Simply panics on errors.
///
/// This code is an implementation from the descriptions in
/// [Wikipedia](https://en.wikipedia.org/wiki/Disjoint-set_data_structure)
pub(crate) struct DisjointSetForest {
	n_trees: usize,
	elements: Vec<SetElement>,
}

impl DisjointSetForest {
	pub(crate) fn new(n: usize) -> Self {
		DisjointSetForest {
			n_trees: n,
			elements: (0..n).map(|i| SetElement { parent: i, rank: 0 }).collect(),
		}
	}

	pub(crate) fn n_trees(&self) -> usize {
		self.n_trees
	}

	pub(crate) fn find(&mut self, mut x: usize) -> usize {
		while self.elements[x].parent != x {
			let parent = self.elements[x].parent;
			self.elements[x].parent = self.elements[self.elements[x].parent].parent;
			x = parent;
		}

		x
	}

	pub(crate) fn union(&mut self, x: usize, y: usize) {
		let x = self.find(x);
		let y = self.find(y);

		if x == y {
			return;
		}

		self.n_trees -= 1;
		let x_rank = self.elements[x].rank;
		let y_rank = self.elements[y].rank;

		if x_rank < y_rank {
			// y becomes parent
			self.elements[x].parent = y;
		} else if x_rank > y_rank {
			// x becomes parent
			self.elements[y].parent = x;
		} else {
			// x becomes parent
			self.elements[x].rank += 1;
			self.elements[y].parent = x;
		}
	}
}
