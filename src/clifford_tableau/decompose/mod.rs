mod delicate_solver;
mod simple_solver;

use crate::{
	clifford_tableau::CliffordTableau,
	connectivity::{Connectivity, ExplosionNode, hypergraph::HyperEdgeIndex},
	misc::NonZeroEvenUsize,
	pauli::{CliffordPauliAngle, PauliExp, PauliLetter, PauliString},
};
use delicate_solver::{delicate_solver, fastest_delicate};
use petgraph::{Undirected, prelude::StableGraph};
use simple_solver::{fastest, simple_solver};

#[derive(Debug, PartialEq, Eq)]
enum QubitProtection {
	X,
	Z,
	None,
}

impl<const N: usize> CliffordTableau<N> {
	/// # Decompose
	///
	/// Decomposes the tableau into clifford gates.
	pub fn decompose(
		self,
		gate_size: NonZeroEvenUsize,
		connectivity: Option<&Connectivity>,
	) -> Vec<PauliExp<N, CliffordPauliAngle>> {
		match connectivity {
			Some(connectivity) => {
				let mut graph: StableGraph<ExplosionNode, usize, Undirected, u32> =
					connectivity.explosion.clone().into();
				let mut handled_edges: Vec<HyperEdgeIndex> = Vec::new();

				while graph.node_count() != 0 {
					let indices: Vec<_> = graph.node_indices().collect();
					for index in indices {
						let neighbors: Vec<_> = graph.neighbors(index).collect();
						if neighbors.len() == 1 {
							let mut node = graph.remove_node(index).unwrap();
							if node.hyper_edges.len() == 1 {
								// maps to the hyperedge that we are working on.
								let edge_index = node.hyper_edges.pop().unwrap();
								let edge = connectivity.hypergraph.get_edge(edge_index).unwrap();
								for node_inxdex in edge.nodes {
									let node = connectivity.hypergraph.get_node(node_inxdex);
									// HERE
								}
								connectivity.hypergraph.get_node(index)
								// Check what qubits we have 

								let neighbor_node = steiner_tree
									.node_weight(*neighbors.first().unwrap())
									.unwrap();
								let targets = neighbor_node.hyper_nodes.clone();

								result.push((edge, Some(targets)));
							}
						} else if neighbors.is_empty() {
							let weight = graph.node_weight(index).unwrap();
							if weight.hyper_edges.len() == 1 {
								assert_eq!(graph.node_count(), 1);
								let mut node = graph.remove_node(index).unwrap();
								let edge = node.hyper_edges.pop().unwrap();
								// TODO: handle last edge
							} else {
								graph.remove_node(index).unwrap();
							}
						}
					}
				}
				// Find tree for everything

				// for hyperedges
				//     for qubits that are not in other reamining hyperedges
				//         make instructions so that row (x or z) only has qubits in hyperedge
				//		   use solver (simple or delicate) to solve row
				//         do same for other row (z or x)
				//     remove hyperedge

				todo!()
			}
			_ => self.decompose_full_connectivity(gate_size),
		}
	}

	fn decompose_full_connectivity(
		mut self,
		gate_size: NonZeroEvenUsize,
	) -> Vec<PauliExp<N, CliffordPauliAngle>> {
		let mut decomposition: Vec<PauliExp<N, CliffordPauliAngle>> = Vec::new();
		let mut dirty_qubits: Vec<usize> = (0..N).collect();

		while dirty_qubits.len() >= gate_size.as_value() {
			let (qubit, letter) = fastest(&self, &dirty_qubits, gate_size).unwrap();

			match letter {
				PauliLetter::X => {
					let x_moves = simple_solver(
						self.x.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::X,
						&dirty_qubits,
						QubitProtection::None,
					);
					for string in x_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
					let z_moves = simple_solver(
						self.z.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::Z,
						&dirty_qubits,
						QubitProtection::X,
					);
					for string in z_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
				}
				PauliLetter::Z => {
					let z_moves = simple_solver(
						self.z.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::Z,
						&dirty_qubits,
						QubitProtection::None,
					);
					for string in z_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
					let x_moves = simple_solver(
						self.x.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::X,
						&dirty_qubits,
						QubitProtection::Z,
					);
					for string in x_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
				}
				_ => unreachable!(),
			}

			// Now the qubit is not dirty anymore
			dirty_qubits.retain(|q| *q != qubit);
		}

		// then for remaining use delicate solver
		while !dirty_qubits.is_empty() {
			let (qubit, letter) = fastest_delicate(&self, &dirty_qubits).unwrap();

			match letter {
				PauliLetter::X => {
					let x_moves = delicate_solver(
						self.x.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::X,
					);
					for string in x_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
					let z_moves = delicate_solver(
						self.z.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::Z,
					);
					for string in z_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
				}
				PauliLetter::Z => {
					let z_moves = delicate_solver(
						self.z.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::Z,
					);
					for string in z_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
					let x_moves = delicate_solver(
						self.x.get(qubit).unwrap(),
						gate_size,
						qubit,
						PauliLetter::X,
					);
					for string in x_moves {
						self.merge_pi_over_4_pauli(false, &string);
						// the decomposition has the reverse operation
						decomposition.push(PauliExp {
							string,
							angle: CliffordPauliAngle::NeqPiOver4,
						});
					}
				}
				_ => unreachable!(),
			}

			// Now the qubit is not dirty anymore
			dirty_qubits.retain(|q| *q != qubit);
		}

		for (i, (x, z)) in self
			.x_signs
			.clone()
			.into_iter()
			.zip(self.z_signs.clone().into_iter())
			.enumerate()
		{
			let string = match (x, z) {
				(true, true) => PauliString::y(i),
				(true, false) => PauliString::z(i),
				(false, true) => PauliString::x(i),
				_ => {
					continue;
				}
			};

			self.merge_pi_over_4_pauli(true, &string);
			self.merge_pi_over_4_pauli(true, &string);
			decomposition.push(PauliExp {
				string: string.clone(),
				angle: CliffordPauliAngle::PiOver4,
			});
			decomposition.push(PauliExp {
				string,
				angle: CliffordPauliAngle::PiOver4,
			});
		}

		assert_eq!(self, CliffordTableau::id());

		decomposition.into_iter().rev().collect()
	}
}
