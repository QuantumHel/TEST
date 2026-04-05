use crate::connectivity::{Connectivity, ConnectivityEdge};

#[derive(Debug)]
pub struct SubEdge<'a, T: ConnectivityEdge> {
	pub original: &'a T,
	pub(super) qubits: Vec<usize>,
}

impl<'a, T: ConnectivityEdge> SubEdge<'a, T> {
	pub fn qubits(&self) -> &[usize] {
		&self.qubits
	}
}

#[derive(Debug)]
pub struct SubNode<'a> {
	pub original: &'a Vec<usize>,
	pub(super) edges: Vec<usize>,
}

impl<'a> SubNode<'a> {
	pub fn edges(&self) -> &[usize] {
		&self.edges
	}
}

#[derive(Debug)]
pub struct SubGraph<'a, T: ConnectivityEdge> {
	/// Indexes for the edges in the original graph
	pub(super) edges: Vec<Option<SubEdge<'a, T>>>,
	/// Indexes for the qubits in the original graph
	pub(super) qubits: Vec<Option<SubNode<'a>>>,
}

impl<'a, T: ConnectivityEdge> SubGraph<'a, T> {
	pub(super) fn empty(connectivity: &'a Connectivity<T>) -> Self {
		let mut edges: Vec<Option<SubEdge<'a, T>>> = Vec::with_capacity(connectivity.edges.len());
		edges.resize_with(connectivity.edges.len(), || None);
		let mut qubits: Vec<Option<SubNode<'a>>> = Vec::with_capacity(connectivity.qubits.len());
		qubits.resize_with(connectivity.qubits.len(), || None);

		Self { edges, qubits }
	}

	// Remove qubit

	// remove edge

	// leaf nodes

	// Function to get edges

	// Function to get qubits
}
