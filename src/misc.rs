use std::ops::Deref;

use petgraph::{
	Undirected, prelude::StableGraph, visit::EdgeRef
};


#[derive(Debug, Clone, Copy)]
pub struct NonZeroEvenUsize {
	value: usize,
}

impl Deref for NonZeroEvenUsize {
	type Target = usize;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl NonZeroEvenUsize {
	pub fn new(value: usize) -> Option<Self> {
		if value == 0 {
			return None;
		}

		match value.is_multiple_of(2) {
			true => Some(Self { value }),
			false => None,
		}
	}

	pub fn as_value(self) -> usize {
		self.value
	}
}

pub mod generic_bounds {
	//! This module is used to force bounds on generic constants. This module
	//! require the use of `#![feature(generic_const_exprs)]`.
	//!
	//! # Example
	//! Asserting that N >= P
	//! ```rust
	//! impl<const N: usize> Connectivity<N> {
	//! 	pub fn something<const P: usize>(string: PauliString<P>)
	//! 	where Assert<{ N >= P}>: IsTrue
	//! 	{
	//! 		todo!()
	//! 	}
	//! }
	//! ```

	pub enum Assert<const C: bool> {}

	pub trait IsTrue {}

	impl IsTrue for Assert<true> {}
}

/// Makes sure that the graph is a tree.
pub fn enforce_tree<N, E>(graph: &mut StableGraph<N, E, Undirected>) {
	let mut visited = Vec::new();
	let mut next = Vec::new();
	let mut used_edges = Vec::new();

	let first = graph.node_indices().next().unwrap();
	visited.push(first);
	next.push(first);

	while !next.is_empty() {
		let node = next.remove(0);
		let edges = graph.edges(node);
		let mut remove = Vec::new();
		for edge in edges {
			let id = edge.id();
			if used_edges.contains(&id) {
				continue
			}

			let neighbor = if edge.source() == node {
				edge.target()
			} else {
				edge.source()
			};

			if visited.contains(&neighbor) {
				remove.push(id);
			} else {
				used_edges.push(id);
				visited.push(neighbor);
				next.push(neighbor);
			}
		}

		for id in remove {
			graph.remove_edge(id);
		}
	}
}