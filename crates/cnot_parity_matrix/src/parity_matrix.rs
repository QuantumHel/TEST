use crate::xor_span::XorSpan;
use bits::Bits;
use circuit::gates::CNot;
use core::QubitMapping;
use graph::{Cardinality, ConstCardinalityEdge, Graph, GraphExt, Node};
use std::ops::{Range, RangeBounds};

#[derive(Debug, Default, Clone, Copy)]
pub enum Basis {
	#[default]
	Standard,
	Hadamard,
}

/// A parity matrix representing an operation that can be done using cnot gates.
///
///
/// The representation is such that output qubit parities map to rows.
///
/// For example in the standard basis the circuit:
/// ```text
///        0⊕1
/// |0>─[x]───────[x]─|0⊕2>
///      │         |
/// |1>──■──■──────┼──|1>
///         | 1⊕2 |
/// |2>────[x]─────■──|1⊕2>
/// ```
/// maps to
/// ```text
/// [ 1 0 1 ]
/// [ 0 1 0 ]
/// [ 0 1 1 ]
/// ```
///
/// , and in the hadamard basis (cnot reversed):
/// ```text
///
/// |0>─[H]─[x]─────────────[x]─[H]─|0>
///          │ 0⊕1  0⊕1⊕2 |     
/// |1>─[H]──■─────■─────────┼──[H]─|0⊕1⊕2>
///                |         |
/// |2>─[H]───────[x]────────■──[H]─|0⊕2>
/// ```
///
/// maps to
/// ```text
/// [ 1 0 0 ]
/// [ 1 1 1 ]
/// [ 1 0 1 ]
/// ```
#[derive(Debug, Clone, Default)]
pub struct ParityMatrix {
	rows: Vec<Bits>,
	basis: Basis,
}

impl ParityMatrix {
	pub fn standard_basis() -> Self {
		Self {
			rows: Vec::new(),
			basis: Basis::Standard,
		}
	}

	pub fn standard_from_rows(rows: Vec<Bits>) -> Self {
		Self {
			rows,
			basis: Basis::Standard,
		}
	}

	pub fn hadamard_basis() -> Self {
		Self {
			rows: Vec::new(),
			basis: Basis::Hadamard,
		}
	}

	pub fn hadamard_from_rows(rows: Vec<Bits>) -> Self {
		Self {
			rows,
			basis: Basis::Standard,
		}
	}

	pub fn basis(&self) -> Basis {
		self.basis
	}

	pub fn get(&self, row: usize, col: usize) -> bool {
		self.rows.get(row).unwrap_or(&Bits::with_one(row)).get(col)
	}

	pub fn get_row(&self, row: usize) -> Bits {
		self.rows.get(row).unwrap_or(&Bits::with_one(row)).clone()
	}

	pub fn get_section<T: RangeBounds<usize>>(&self, row: usize, cols: Range<usize>) -> Bits {
		self.rows
			.get(row)
			.unwrap_or(&Bits::with_one(row))
			.get_range(cols)
	}

	/// Insets a cnot to the end of the parity matrix.
	///
	/// For example if the cnots are represented as matrices $M_i$, and the
	/// original matrix is:
	///
	/// $$M_0M_1M_2M_3M_4$$
	///
	/// and we insert $M_5$, it goes to the end and `self` becomes
	///
	/// $$M_0M_1M_2M_3M_4M_5$$
	pub fn insert_cnot(&mut self, cnot: CNot) {
		match self.basis {
			Basis::Standard => self.add_row(cnot.control(), cnot.target()),
			Basis::Hadamard => self.add_row(cnot.target(), cnot.control()),
		};
	}

	/// Inserts a [QubitMapping] to the end of a [ParityMatrix] (output
	/// [QubitMapping]).
	pub fn insert_qubit_mapping(&mut self, qubit_mapping: &QubitMapping) {
		let mut clone = self.clone();
		for (original, target) in qubit_mapping.map_non_trivial() {
			let row = self.get_row(original);
			while clone.rows.len() < target {
				clone.rows.push(Bits::with_one(clone.rows.len()));
			}
			clone.rows[target] = row;
		}

		*self = clone;
	}

	/// Adds two rows together and returns the corresponding [CNot] added to the
	/// end.
	///
	/// As the [CNot]s returned are at the end, the order of them (when $M_0$ is
	/// from the first addition) is
	///
	/// $$...M_4M_3M_2M_1M_0$$
	pub fn add_row(&mut self, source: usize, target: usize) -> CNot {
		while self.rows.len() <= target {
			self.rows.push(Bits::with_one(self.rows.len()));
		}

		// We made sure above that we can unwrap on target.
		if let Some(source) = self.rows.get(source) {
			// Compiler probably removes this clone
			let source = source.clone();
			*self.rows.get_mut(target).unwrap() ^= source;
		} else {
			let target = self.rows.get_mut(target).unwrap();
			target.set(source, !target.get(source));
		}

		match self.basis {
			Basis::Standard => CNot::new(source, target).unwrap(),
			Basis::Hadamard => CNot::new(target, source).unwrap(),
		}
	}

	pub fn transpose(&self) -> Self {
		let size = self.size();
		let mut transpose = ParityMatrix {
			rows: vec![Bits::default(); size],
			basis: self.basis,
		};
		// This is reversed so that we allocate the needed bits size on first
		// round.
		for (i, row) in self.rows.iter().enumerate().rev() {
			for j in row.iter_ones() {
				transpose
					.rows
					.get_mut(j)
					.expect("Size function failed?")
					.set(i, true);
			}
		}

		// Sets the diagonal for rows that are omitted in self due to being
		// trivial (e_i)
		if size > self.rows.len() {
			for i in self.rows.len()..size {
				transpose
					.rows
					.get_mut(i)
					.expect("Size function failed?")
					.set(i, true);
			}
		}

		transpose
	}

	pub fn size(&self) -> usize {
		let mut size = 0;
		for (i, row) in self.rows.iter().enumerate() {
			size = size.max(
				row.last_one()
					.expect("Should not be able to have empty rows")
					+ 1,
			);
			if *row != Bits::with_one(i) {
				size = size.max(i + 1);
			}
		}

		size
	}

	pub fn is_identity(&self) -> bool {
		for (i, row) in self.rows.iter().enumerate() {
			if *row != Bits::with_one(i) {
				return false;
			}
		}

		true
	}

	/// removes rows that are not needed
	pub fn trim(&mut self) {
		for i in (0..self.rows.len()).rev() {
			if self.rows.get(i).unwrap() == &Bits::with_one(i) {
				self.rows.pop().unwrap();
			} else {
				return;
			}
		}
	}

	pub fn span_bits(&self, bits: &Bits) -> Option<Bits> {
		let size = self.size();
		if size > self.rows.len() {
			let mut rows = self.rows.clone();
			while size > rows.len() {
				rows.push(Bits::with_one(rows.len()));
			}

			let span = XorSpan::new(&rows);
			return span.span_element(bits);
		}

		let span = XorSpan::new(&self.rows);
		span.span_element(bits)
	}

	/// Elimination of a columnd as described in
	/// https://doi.org/10.1103/PhysRevResearch.5.013065
	///
	/// The resulting column will have `1` only in the `row` that was given as
	/// a parameter as used in paper https://arxiv.org/abs/2205.00724v4
	///
	/// Importantly the output [CNot] gates are reversed as described in
	/// [ParityMatrix::add_row]
	pub fn eliminate_column<
		G: Graph<N, E>,
		N: Node,
		E: ConstCardinalityEdge<CARDINALITY = Cardinality<2>>,
	>(
		&mut self,
		row: usize,
		column: usize,
		connectivity: &G,
	) -> Vec<CNot> {
		let mut result = Vec::new();
		let s: Vec<_> = (0..self.rows.len().max(column))
			.filter(|j| self.get(*j, column))
			.chain([row])
			.collect();

		let tree = connectivity.steiner_tree(&s);

		for (j, k) in tree.postorder_traversal(row) {
			if let Some(k) = k
				&& self.get(j, column)
				&& !self.get(k, column)
			{
				result.push(self.add_row(j, k));
			}
		}

		for (j, k) in tree.postorder_traversal(row) {
			for edge in tree.get_node(j).unwrap().edges() {
				let neighbor: usize = *tree
					.get_edge(*edge)
					.unwrap()
					.nodes()
					.iter()
					.find(|n| **n != j)
					.unwrap();

				if let Some(k) = k
					&& neighbor == k
				{
					continue;
				}

				result.push(self.add_row(j, neighbor));
			}
		}

		for i in 0..self.rows.len().max(column) {
			assert_eq!(self.get(i, column), i == row);
		}

		result
	}

	/// Elimination of a row as described in
	/// https://doi.org/10.1103/PhysRevResearch.5.013065
	///
	/// The resulting row will have `1` only in the `column` that was given as
	/// a parameter as used in paper https://arxiv.org/abs/2205.00724v4
	///
	/// Importantly the output [CNot] gates are reversed as described in
	/// [ParityMatrix::add_row]
	pub fn eliminate_row<
		G: Graph<N, E>,
		N: Node,
		E: ConstCardinalityEdge<CARDINALITY = Cardinality<2>>,
	>(
		&mut self,
		row: usize,
		column: usize,
		connectivity: &G,
	) -> Vec<CNot> {
		let mut result = Vec::new();
		let sum_target = {
			let mut original = self.get_row(row);
			original.set(column, !original.get(column));
			original
		};

		let s_prime: Vec<usize> = self
			.span_bits(&sum_target)
			.expect("should be impossible")
			.iter_ones()
			.collect();
		let terminals = {
			let mut terminals: Vec<usize> = s_prime.clone();
			terminals.push(row);
			terminals
		};

		let tree_prime = connectivity.steiner_tree(&terminals);

		for (j, parent) in tree_prime.preorder_traversal(row) {
			if let Some(parent) = parent
				&& !s_prime.contains(&j)
			{
				result.push(self.add_row(j, parent));
			}
		}

		for (j, parent) in tree_prime.postorder_traversal(row) {
			if let Some(parent) = parent {
				result.push(self.add_row(j, parent));
			}
		}

		assert_eq!(self.get_row(row), Bits::with_one(column));

		result
	}
}

impl std::fmt::Display for ParityMatrix {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let n = self.size();

		if n == 0 {
			return writeln!(f, "Empty ParityMatrix");
		}

		for row in self.rows.iter().take(n) {
			let string = (0..n)
				.map(|i| if row.get(i) { "1 " } else { "0 " })
				.collect::<String>();
			writeln!(f, "{}", string.trim())?;
		}

		Ok(())
	}
}

#[cfg(test)]
mod test {
	use core::QubitMapping;

	use bits::Bits;
	use circuit::gates::CNot;

	use crate::ParityMatrix;

	#[test]
	fn manual_testing() {
		let answer = vec![
			CNot::new(4, 3).unwrap(), // Leftmost
			CNot::new(1, 0).unwrap(),
			CNot::new(3, 1).unwrap(),
			CNot::new(5, 2).unwrap(),
			CNot::new(4, 2).unwrap(),
			CNot::new(4, 3).unwrap(),
			CNot::new(5, 4).unwrap(),
			CNot::new(2, 3).unwrap(), // Start of dashed box
			CNot::new(3, 2).unwrap(),
			CNot::new(3, 5).unwrap(),
			CNot::new(2, 4).unwrap(),
			CNot::new(1, 2).unwrap(),
			CNot::new(0, 1).unwrap(),
			CNot::new(0, 4).unwrap(),
			CNot::new(0, 3).unwrap(),
		];

		let mut partiy_matrix = ParityMatrix::default();
		for cnot in answer.iter() {
			partiy_matrix.add_row(cnot.control(), cnot.target());
		}
		println!("{partiy_matrix}");
	}

	#[test]
	fn test() {
		let mut matrix = ParityMatrix::default();
		matrix.insert_cnot(CNot::new(1, 0).unwrap());
		matrix.insert_cnot(CNot::new(2, 0).unwrap());
		matrix.insert_cnot(CNot::new(0, 1).unwrap());
		matrix.insert_cnot(CNot::new(1, 0).unwrap());
		matrix.insert_cnot(CNot::new(2, 0).unwrap());
		matrix.insert_cnot(CNot::new(0, 2).unwrap());

		let mut qubit_map = QubitMapping::default();
		qubit_map.swap_mapped(0, 1);
		qubit_map.swap_mapped(1, 2);

		let mut row_0 = Bits::with_one(1);
		row_0.set(2, true);
		let mut row_1 = Bits::with_one(0);
		row_1.set(2, true);
		let row_2 = Bits::with_one(1);
		assert_eq!(matrix.size(), 3);
		assert_eq!(matrix.get_row(0), row_0);
		assert_eq!(matrix.get_row(1), row_1);
		assert_eq!(matrix.get_row(2), row_2);
	}
}
