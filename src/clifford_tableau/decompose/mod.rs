mod delicate_solver;
mod routing_help;
mod simple_solver;

use crate::{
	clifford_tableau::{CliffordTableau, decompose::routing_help::handle_target},
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
			Some(connectivity) => self.decompose_with_connectivity(gate_size, connectivity),
			_ => self.decompose_full_connectivity(gate_size),
		}
	}

	fn decompose_with_connectivity(
		mut self,
		gate_size: NonZeroEvenUsize,
		connectivity: &Connectivity,
	) -> Vec<PauliExp<N, CliffordPauliAngle>> {
		let mut decomposition: Vec<PauliExp<N, CliffordPauliAngle>> = Vec::new();

		let mut graph: StableGraph<ExplosionNode, usize, Undirected, u32> =
			connectivity.explosion.clone().into();
		let mut handled_edges: Vec<HyperEdgeIndex> = Vec::new();

		while graph.node_count() != 0 {
			// check for leafs
			// We need to do this collect because borrowing
			let indices: Vec<_> = graph.node_indices().collect();
			for index in indices {
				let neighbors: Vec<_> = graph.neighbors(index).collect();
				// see if leaf or separate (meaning last one)
				if neighbors.len() == 1 {
					let node = graph.node_weight(index).unwrap();
					// This means that the node does not correspond to a hyperedge
					if node.hyper_edges.len() != 1 {
						continue;
					}
					// maps to the hyperedge that we are working on.
					let edge_index: HyperEdgeIndex = *node.hyper_edges.first().unwrap();
					let edge = connectivity.hypergraph.get_edge(edge_index).unwrap();

					// Tese are the qubits that we solve now
					let mut targets: Vec<usize> = Vec::new();
					for node_inxdex in edge.nodes.iter() {
						if edge.nodes.contains(node_inxdex) {
							targets.push(*node_inxdex);
							continue;
						}
						let node = connectivity.hypergraph.get_node(*node_inxdex).unwrap();
						for edge in node.edges.iter() {
							if *edge != edge_index && !handled_edges.contains(edge) {
								// The node contributes to a hyperedge that we handle later
								continue;
							}
						}
						targets.push(*node_inxdex);
					}

					// track which qubits can still be used freely
					let mut dirty_qubits: Vec<_> = targets.clone().into_iter().rev().collect();

					// TODO: Maybe later could check which is fastes to solve
					// solve targets
					for target in targets.iter() {
						let strings = handle_target(
							self.get_x_row(*target).unwrap().0.clone(),
							PauliLetter::X,
							*target,
							QubitProtection::None,
							&graph,
							edge_index,
							connectivity,
							gate_size,
							&dirty_qubits,
							connectivity.hypergraph.get_edge(edge_index).unwrap(),
						);

						for string in strings {
							self.merge_pi_over_4_pauli(false, &string);
							// the decomposition has the reverse operation
							decomposition.push(PauliExp {
								string,
								angle: CliffordPauliAngle::NeqPiOver4,
							});
						}

						assert_eq!(self.x.get(*target).unwrap(), &PauliString::x(*target));

						let strings = handle_target(
							self.get_z_row(*target).unwrap().0.clone(),
							PauliLetter::Z,
							*target,
							QubitProtection::X,
							&graph,
							edge_index,
							connectivity,
							gate_size,
							&dirty_qubits,
							connectivity.hypergraph.get_edge(edge_index).unwrap(),
						);

						for string in strings {
							self.merge_pi_over_4_pauli(false, &string);
							// the decomposition has the reverse operation
							decomposition.push(PauliExp {
								string,
								angle: CliffordPauliAngle::NeqPiOver4,
							});
						}

						assert_eq!(self.z.get(*target).unwrap(), &PauliString::z(*target));

						dirty_qubits.pop();
					}

					// Remove the solved part from the graph
					graph.remove_node(index);
					handled_edges.push(edge_index);
				} else if neighbors.is_empty() {
					let weight = graph.node_weight(index).unwrap();
					if weight.hyper_edges.len() == 1 {
						assert_eq!(graph.node_count(), 1);
						let mut explosion_node = graph.remove_node(index).unwrap();
						let edge_index = explosion_node.hyper_edges.pop().unwrap();
						let edge = connectivity.hypergraph.get_edge(edge_index).unwrap();
						let mut dirty_qubits: Vec<usize> =
							edge.nodes.clone().into_iter().rev().collect();
						let targets = &edge.nodes;

						for target in targets.iter() {
							let strings = handle_target(
								self.get_x_row(*target).unwrap().0.clone(),
								PauliLetter::X,
								*target,
								QubitProtection::None,
								&graph,
								edge_index,
								connectivity,
								gate_size,
								&dirty_qubits,
								connectivity.hypergraph.get_edge(edge_index).unwrap(),
							);

							for string in strings {
								self.merge_pi_over_4_pauli(false, &string);
								// the decomposition has the reverse operation
								decomposition.push(PauliExp {
									string,
									angle: CliffordPauliAngle::NeqPiOver4,
								});
							}

							assert_eq!(self.x.get(*target).unwrap(), &PauliString::x(*target));

							let strings = handle_target(
								self.get_z_row(*target).unwrap().0.clone(),
								PauliLetter::Z,
								*target,
								QubitProtection::X,
								&graph,
								edge_index,
								connectivity,
								gate_size,
								&dirty_qubits,
								connectivity.hypergraph.get_edge(edge_index).unwrap(),
							);

							for string in strings {
								self.merge_pi_over_4_pauli(false, &string);
								// the decomposition has the reverse operation
								decomposition.push(PauliExp {
									string,
									angle: CliffordPauliAngle::NeqPiOver4,
								});
							}

							assert_eq!(self.z.get(*target).unwrap(), &PauliString::z(*target));

							dirty_qubits.pop();
						}
					} else {
						// This is node represents an edge connecting to nothing
						graph.remove_node(index).unwrap();
					}
				}
			}
		}

		// Fix signs
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

		decomposition
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
						None,
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
						None,
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
						None,
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
						None,
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

		// Fix signs
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
