use std::collections::HashMap;

use crate::{
	OpaqueFunction, OpenQasm2Frontend, ast,
	error::{Error, ErrorKind},
	virtual_file_override_list::VirtualFileOverrideList,
};

struct MappedDeclaration {
	indices: Vec<usize>,
}

#[derive(Clone)]
enum GateInput {
	Argument(ast::Argument),
	_Expression(ast::Expression),
}

#[derive(Clone)]
enum DefinedGate {
	Gate(ast::GateDeclaration),
	Opaque,
}

struct SymTab<'a, T: 'static> {
	frontned: &'a OpenQasm2Frontend<T>,
	override_list: &'a VirtualFileOverrideList,
	gates: HashMap<String, DefinedGate>,
	declarations: HashMap<String, MappedDeclaration>,
	gate_inputs: Vec<HashMap<String, GateInput>>,
	n_qubits: usize,
	n_bits: usize,
}

impl<T: 'static> OpenQasm2Frontend<T> {
	pub(crate) fn generate_ir(
		&self,
		ast: &ast::Program,
		override_list: &VirtualFileOverrideList,
		ir: &mut T,
	) -> Result<(), Error> {
		let mut sym_tab = SymTab {
			frontned: self,
			override_list,
			gates: HashMap::default(),
			declarations: HashMap::default(),
			gate_inputs: Vec::default(),
			n_qubits: usize::default(),
			n_bits: usize::default(),
		};
		sym_tab.process_program(ast, ir)
	}
}

enum Indices {
	Register(Vec<usize>),
	Single(usize),
}

impl Indices {
	fn at_index(&self, index: usize) -> usize {
		match self {
			Indices::Register(register) => register[index],
			Indices::Single(i) => *i,
		}
	}
}

impl<'a, T> SymTab<'a, T> {
	fn process_program(&mut self, program: &ast::Program, ir: &mut T) -> Result<(), Error> {
		for statement in program.program.iter() {
			match statement {
				// Translates to nop
				ast::Statement::Barrier(_) => {}
				ast::Statement::Declaration(declaration) => match declaration.ty {
					ast::DeclarationType::Qubit => {
						let indices: Vec<_> =
							(self.n_qubits..(self.n_qubits + declaration.size as usize)).collect();
						self.n_qubits += declaration.size as usize;
						self.declarations
							.insert(declaration.name.text.clone(), MappedDeclaration { indices });
					}
					ast::DeclarationType::Bit => {
						let indices: Vec<_> =
							(self.n_bits..(self.n_bits + declaration.size as usize)).collect();
						self.n_bits += declaration.size as usize;
						self.declarations
							.insert(declaration.name.text.clone(), MappedDeclaration { indices });
					}
				},
				ast::Statement::GateDeclaration(gate_declaration) => {
					self.gates.insert(
						gate_declaration.name.text.clone(),
						DefinedGate::Gate(gate_declaration.clone()),
					);
				}
				ast::Statement::Opaque(opaque) => {
					self.gates
						.insert(opaque.name.text.clone(), DefinedGate::Opaque);
				}
				ast::Statement::QuantumOperator(quantum_operator) => match quantum_operator {
					ast::QuantumOperator::UnitaryOperator(operator) => {
						self.process_unitary_operator(operator, ir)?;
					}
					ast::QuantumOperator::Measure { location, .. } => {
						return Err(Error::new(
							ErrorKind::NotYetImplemented("measurement"),
							location.clone(),
						));
					}
					ast::QuantumOperator::Reset { location, .. } => {
						return Err(Error::new(
							ErrorKind::NotYetImplemented("reset"),
							location.clone(),
						));
					}
				},
				ast::Statement::If { location, .. } => {
					return Err(Error::new(
						ErrorKind::NotYetImplemented("if statement"),
						location.clone(),
					));
				}
			}
		}

		Ok(())
	}

	fn process_unitary_operator(
		&mut self,
		operator: &ast::UnitaryOperator,
		ir: &mut T,
	) -> Result<(), Error> {
		match operator {
			ast::UnitaryOperator::U { location, .. } => Err(Error::new(
				ErrorKind::NotYetImplemented(
					"default U gate (because numeric parameters not supported)",
				),
				location.clone(),
			)),
			ast::UnitaryOperator::Cx {
				control, target, ..
			} => {
				let control = self.get_indices(control);
				let target = self.get_indices(target);

				match (control, target) {
					(Indices::Single(control), Indices::Single(target)) => {
						self.frontned
							.cx
							.as_ref()
							.unwrap()
							.insert_cnot(control, target, ir);
					}
					(Indices::Single(control), Indices::Register(targets)) => {
						for target in targets.into_iter() {
							self.frontned
								.cx
								.as_ref()
								.unwrap()
								.insert_cnot(control, target, ir);
						}
					}
					(Indices::Register(controls), Indices::Single(target)) => {
						for control in controls.into_iter() {
							self.frontned
								.cx
								.as_ref()
								.unwrap()
								.insert_cnot(control, target, ir);
						}
					}
					(Indices::Register(controls), Indices::Register(targets)) => {
						for (control, target) in controls.into_iter().zip(targets) {
							self.frontned
								.cx
								.as_ref()
								.unwrap()
								.insert_cnot(control, target, ir);
						}
					}
				}

				Ok(())
			}
			ast::UnitaryOperator::CustomGate {
				name,
				params,
				qargs,
				location,
			} => {
				if !params.is_empty() {
					return Err(Error::new(
						ErrorKind::NotYetImplemented("numeric parameters"),
						location.clone(),
					));
				}

				let gate = self.gates.get(&name.text).unwrap().clone();
				match gate {
					DefinedGate::Gate(gate) => {
						let input_len = if !self.gate_inputs.is_empty() {
							1
						} else {
							qargs
								.iter()
								.filter_map(|a| match a {
									ast::Argument::Named { name } => Some(
										self.declarations.get(&name.text).unwrap().indices.len(),
									),
									_ => None,
								})
								.max()
								.unwrap_or(1)
						};

						for i in 0..input_len {
							let mut gate_inputs: HashMap<String, GateInput> = HashMap::new();
							for (name, qarg) in gate.qargs.iter().zip(qargs.iter()) {
								let qarg_name = qarg.name();
								if let Some(previous_gate_inputs) = self.gate_inputs.last() {
									let gate_input =
										previous_gate_inputs.get(&qarg_name.text).cloned().unwrap();
									// As we are inside function we are already indexed
									assert!(matches!(
										gate_input,
										GateInput::Argument(ast::Argument::Indexed { .. })
									));
									gate_inputs.insert(name.text.clone(), gate_input);
								} else {
									match qarg {
										ast::Argument::Named { name: reg } => {
											gate_inputs.insert(
												name.text.clone(),
												GateInput::Argument(ast::Argument::Indexed {
													name: reg.clone(),
													index: i as u64,
												}),
											);
										}
										ast::Argument::Indexed { name: reg, index } => {
											gate_inputs.insert(
												name.text.clone(),
												GateInput::Argument(ast::Argument::Indexed {
													name: reg.clone(),
													index: *index,
												}),
											);
										}
									}
								}
							}
							self.gate_inputs.push(gate_inputs);

							for operator in gate.quantum_operators.iter() {
								match operator {
									ast::GateOperation::UnitaryOperator(unitary_operator) => {
										self.process_unitary_operator(unitary_operator, ir)?;
									}
									// Translates to nop
									ast::GateOperation::Barrier(_) => {}
								}
							}

							self.gate_inputs.pop();
						}
					}
					DefinedGate::Opaque => {
						if let Some(gate_impl) = self.get_opaque_impl(name) {
							let inputs: Vec<_> =
								qargs.iter().map(|q| self.get_indices(q)).collect();
							let input_len = inputs
								.iter()
								.filter_map(|i| match i {
									Indices::Register(i) => Some(i.len()),
									_ => None,
								})
								.max()
								.unwrap_or(1);
							for i in 0..input_len {
								let mut qargs = Vec::new();
								for input in inputs.iter() {
									qargs.push(input.at_index(i));
								}

								gate_impl.insert_gate(qargs, ir);
							}
						} else {
							return Err(Error::new(
								ErrorKind::Custom(format!(
									"Opaque {} does to correspond to a given function",
									name.text
								)),
								location.clone(),
							));
						}
					}
				}

				Ok(())
			}
		}
	}

	/// Gets the indices to which the argument points
	fn get_indices(&self, argument: &ast::Argument) -> Indices {
		let argument = if let Some(gate_inputs) = self.gate_inputs.last() {
			let ast::Argument::Named { name } = argument else {
				unreachable!(
					"Type checking should make sure that we dont index inside of functions"
				)
			};

			let GateInput::Argument(argument) = gate_inputs.get(&name.text).unwrap() else {
				unreachable!(
					"Type checking should make sure that we dont index inside of functions"
				)
			};
			argument
		} else {
			argument
		};

		match argument {
			ast::Argument::Named { name } => {
				Indices::Register(self.declarations.get(&name.text).unwrap().indices.clone())
			}
			ast::Argument::Indexed { name, index } => Indices::Single(
				*self
					.declarations
					.get(&name.text)
					.unwrap()
					.indices
					.get(*index as usize)
					.unwrap(),
			),
		}
	}

	fn get_opaque_impl(&self, name: &ast::Id) -> Option<&dyn OpaqueFunction<T>> {
		if let Some(gate_impl) = self.frontned.default_file.opaque_functions.get(&name.text) {
			return Some(gate_impl.as_ref());
		}

		for or in self.override_list.iter() {
			let virtual_file = self
				.frontned
				.file_overrides
				.get(or.as_ref())
				.expect("We override, so we should have");

			if let Some(gate_impl) = virtual_file.opaque_functions.get(&name.text) {
				return Some(gate_impl.as_ref());
			}
		}
		None
	}
}
