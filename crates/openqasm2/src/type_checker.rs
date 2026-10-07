use std::collections::HashMap;

use crate::{
	OpaqueFunction, OpenQasm2Frontend, OpenQasm2IR, VirtualFileOverrideList, ast,
	error::{
		Error,
		ErrorKind::{self, UndefinedIdentifier},
	},
};

impl<T: OpenQasm2IR> OpenQasm2Frontend<T> {
	pub(crate) fn type_check(
		&self,
		ast: &ast::Program,
		override_list: &VirtualFileOverrideList,
	) -> Result<(), Error> {
		let mut type_map = TypeMap {
			frontned: self,
			override_list,
			gates: HashMap::default(),
			declarations: HashMap::default(),
			gate_inputs: Option::default(),
		};
		for statement in ast.program.iter() {
			type_map.process_statement(statement)?;
		}

		Ok(())
	}
}

#[derive(Clone, Debug)]
enum DefinedGate {
	Gate(ast::GateDeclaration),
	Opaque(ast::Opaque),
}

#[derive(Debug, Clone)]
enum GateInput {
	Param,
	Qarg(ast::Argument),
	/// Inside of gate definitions we have named qargs without having anything
	/// set yet.
	QubitBinding(ast::Id),
}

impl PartialEq for GateInput {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(GateInput::Param, GateInput::Param) => false,
			(
				GateInput::QubitBinding(ast::Id { text: a, .. }),
				GateInput::QubitBinding(ast::Id { text: b, .. }),
			) => a == b,
			// FIXME: name is of interestin, but also index
			(
				GateInput::Qarg(ast::Argument::Named { name: a }),
				GateInput::Qarg(ast::Argument::Named { name: b }),
			) => a == b,
			(
				GateInput::Qarg(ast::Argument::Indexed {
					name: a_name,
					index: a_index,
				}),
				GateInput::Qarg(ast::Argument::Indexed {
					name: b_name,
					index: b_index,
				}),
			) => a_name == b_name && a_index == b_index,
			_ => false,
		}
	}
}

struct TypeMap<'a, T: OpenQasm2IR> {
	frontned: &'a OpenQasm2Frontend<T>,
	override_list: &'a VirtualFileOverrideList,
	gates: HashMap<String, DefinedGate>,
	declarations: HashMap<String, ast::Declaration>,
	gate_inputs: Option<HashMap<String, GateInput>>,
}

impl<'a, T: OpenQasm2IR> TypeMap<'a, T> {
	fn process_statement(&mut self, statement: &ast::Statement) -> Result<(), Error> {
		match statement {
			ast::Statement::Declaration(declaration) => self.add_declaration(declaration)?,
			ast::Statement::GateDeclaration(gate_declaration) => self.add_gate(gate_declaration)?,
			ast::Statement::Opaque(opaque) => self.add_opaque(opaque)?,
			ast::Statement::QuantumOperator(quantum_operator) => {
				self.process_quantum_operator(quantum_operator)?
			}
			ast::Statement::If {
				id,
				quantum_operator,
				..
			} => {
				let Some(register) = self.declarations.get(&id.text) else {
					return Err(Error::new(
						ErrorKind::UndefinedIdentifier(id.text.clone()),
						id.location.clone(),
					));
				};
				if register.ty != ast::DeclarationType::Bit {
					return Err(Error::new(
						ErrorKind::Custom(format!(
							"Expected classical register, got quantum register {}",
							register.name.text
						)),
						id.location.clone(),
					));
				}

				self.process_quantum_operator(quantum_operator)?;
			}
			ast::Statement::Barrier(args) => {
				for arg in args.iter() {
					self.is_type(arg, ast::DeclarationType::Qubit)?;
				}
			}
		}

		Ok(())
	}

	fn process_quantum_operator(
		&mut self,
		quantum_operator: &ast::QuantumOperator,
	) -> Result<(), Error> {
		match quantum_operator {
			ast::QuantumOperator::UnitaryOperator(unitary_operator) => {
				self.process_unitary_operator(unitary_operator)?
			}
			ast::QuantumOperator::Measure {
				source,
				target,
				location,
			} => {
				self.is_type(source, ast::DeclarationType::Qubit)?;
				self.is_type(target, ast::DeclarationType::Bit)?;
				// is_type checks that defined so we unwrap
				let source_size = match source {
					ast::Argument::Named { name } => {
						self.declarations.get(&name.text).unwrap().size
					}
					ast::Argument::Indexed { .. } => 1,
				};
				let target_size = match target {
					ast::Argument::Named { name } => {
						self.declarations.get(&name.text).unwrap().size
					}
					ast::Argument::Indexed { .. } => 1,
				};

				if source_size != target_size {
					return Err(Error::new(
						ErrorKind::Custom(format!(
							"Source and destination of measurement have to have matching sizes {source_size} != {target_size}"
						)),
						location.clone(),
					));
				}
			}
			ast::QuantumOperator::Reset { target, .. } => {
				self.is_type(target, ast::DeclarationType::Qubit)?;
			}
		}

		Ok(())
	}

	fn process_unitary_operator(
		&mut self,
		unitary_operator: &ast::UnitaryOperator,
	) -> Result<(), Error> {
		match unitary_operator {
			ast::UnitaryOperator::U {
				theta,
				phi,
				lambda,
				target,
				..
			} => {
				self.process_expression(theta)?;
				self.process_expression(phi)?;
				self.process_expression(lambda)?;
				self.is_type(target, ast::DeclarationType::Qubit)
			}
			ast::UnitaryOperator::Cx {
				control,
				target,
				location,
			} => {
				self.is_type(control, ast::DeclarationType::Qubit)?;
				self.is_type(target, ast::DeclarationType::Qubit)?;

				if !control.is_indexed()
					&& !target.is_indexed()
					&& self.argument_qubit_size(control)? != self.argument_qubit_size(target)?
				{
					return Err(Error::new(
						ErrorKind::Custom(String::from(
							"Register inputs to gates need to have equal sizes",
						)),
						location.clone(),
					));
				}

				if self.overlapping(control, target)? {
					Err(Error::new(
						ErrorKind::Custom(String::from("Control and target for Cx can't be equal")),
						location.clone(),
					))
				} else {
					Ok(())
				}
			}
			ast::UnitaryOperator::CustomGate {
				name,
				params,
				qargs,
				location,
			} => {
				for param in params.iter() {
					self.process_expression(param)?;
				}

				let mut input_register_size: Option<u64> = None;
				for qarg in qargs.iter() {
					self.is_type(qarg, ast::DeclarationType::Qubit)?;
					if !qarg.is_indexed()
						&& let Some(size) = self.argument_qubit_size(qarg)?
					{
						if input_register_size.is_none() {
							input_register_size = Some(size);
						}

						if Some(size) != input_register_size {
							return Err(Error::new(
								ErrorKind::Custom(String::from(
									"Gate parametes have to have matching sizes",
								)),
								location.clone(),
							));
						}
					}
				}

				let Some(gate) = self.gates.get(&name.text).cloned() else {
					return Err(Error::new(
						ErrorKind::Custom(format!("Gate definition for {} not found", name.text)),
						name.location.clone(),
					));
				};

				match gate {
					DefinedGate::Gate(gate) => {
						if gate.params.len() != params.len() {
							return Err(Error::new(
								ErrorKind::Custom(format!(
									"Expected {} parameters, but got {}",
									gate.params.len(),
									params.len()
								)),
								location.clone(),
							));
						}
						if gate.qargs.len() != qargs.len() {
							return Err(Error::new(
								ErrorKind::Custom(format!(
									"Expected {} qargs, but got {}",
									gate.qargs.len(),
									qargs.len()
								)),
								location.clone(),
							));
						}

						let mut new_gate_inputs: HashMap<String, GateInput> = HashMap::new();
						for param in gate.params.iter() {
							new_gate_inputs.insert(param.text.clone(), GateInput::Param);
						}
						for (name, qarg) in gate.qargs.iter().zip(qargs.iter()) {
							let qarg_name = qarg.name();
							if let Some(gate_inputs) = self.gate_inputs.as_ref() {
								let Some(gate_input) = gate_inputs.get(&qarg_name.text) else {
									return Err(Error::new(
										ErrorKind::UndefinedIdentifier(qarg_name.text.clone()),
										qarg_name.location.clone(),
									));
								};

								new_gate_inputs.insert(name.text.clone(), gate_input.clone());
							} else {
								new_gate_inputs
									.insert(name.text.clone(), GateInput::Qarg(qarg.clone()));
							}
						}

						let outer_inputs = self.gate_inputs.take();
						self.gate_inputs = Some(new_gate_inputs);
						for operator in gate.quantum_operators.iter() {
							match operator {
								ast::GateOperation::UnitaryOperator(unitary_operator) => {
									self.process_unitary_operator(unitary_operator)?;
								}
								ast::GateOperation::Barrier(qargs) => {
									for qarg in qargs.iter() {
										match self.gate_inputs.as_ref().unwrap().get(&qarg.text) {
											Some(GateInput::Qarg(_))
											| Some(GateInput::QubitBinding(_)) => {}
											Some(GateInput::Param) => {
												return Err(Error::new(
													ErrorKind::Custom(format!(
														"Expected qubit argument, got {} of numeric type insted",
														qarg.text
													)),
													qarg.location.clone(),
												));
											}
											_ => {
												return Err(Error::new(
													ErrorKind::UndefinedIdentifier(
														qarg.text.clone(),
													),
													qarg.location.clone(),
												));
											}
										}
									}
								}
							}
						}
						self.gate_inputs = outer_inputs;
					}
					DefinedGate::Opaque(opaque) => {
						if opaque.params.len() != params.len() {
							return Err(Error::new(
								ErrorKind::Custom(format!(
									"Expected {} parameters, but got {}",
									opaque.params.len(),
									params.len()
								)),
								location.clone(),
							));
						}
						if opaque.qargs.len() != qargs.len() {
							return Err(Error::new(
								ErrorKind::Custom(format!(
									"Expected {} qargs, but got {}",
									opaque.qargs.len(),
									qargs.len()
								)),
								location.clone(),
							));
						}

						if let Some(gate_impl) = self.get_opaque_impl(name) {
							let inputs = input_register_size.unwrap_or(1);
							for i in 0..inputs {
								let mut parameters = Vec::new();
								for (mut j, qarg) in qargs.iter().enumerate() {
									let new = match qarg {
										ast::Argument::Indexed { name, index } => {
											GateInput::Qarg(ast::Argument::Indexed {
												name: name.clone(),
												index: *index,
											})
										}
										ast::Argument::Named { name } => {
											if let Some(gate_inputs) = self.gate_inputs.as_ref() {
												let Some(gate_input) = gate_inputs.get(&name.text)
												else {
													return Err(Error::new(
														ErrorKind::UndefinedIdentifier(
															name.text.clone(),
														),
														name.location.clone(),
													));
												};

												gate_input.clone()
											} else {
												GateInput::Qarg(ast::Argument::Indexed {
													name: name.clone(),
													index: i,
												})
											}
										}
									};

									for (j_p, previous) in parameters.iter() {
										if *previous == new {
											j = *j_p;
										}
									}

									parameters.push((j, new));
								}

								if let Err(msg) = gate_impl.type_check(
									parameters.iter().map(|(i, _)| i).copied().collect(),
								) {
									return Err(Error::new(
										ErrorKind::Custom(msg.to_string()),
										location.clone(),
									));
								}
							}
						}
					}
				}

				Ok(())
			}
		}
	}

	fn process_expression(&self, expression: &ast::Expression) -> Result<(), Error> {
		match expression {
			ast::Expression::BinaryOperator { left, right, .. } => {
				self.process_expression(left.as_ref())?;
				self.process_expression(right.as_ref())?;
			}
			ast::Expression::UnaryOperator { argument, .. } => {
				self.process_expression(argument)?;
			}
			ast::Expression::Id(id) => {
				if let Some(gate_inputs) = self.gate_inputs.as_ref() {
					let Some(input) = gate_inputs.get(&id.text) else {
						return Err(Error::new(
							ErrorKind::UndefinedIdentifier(id.text.clone()),
							id.location.clone(),
						));
					};
					let GateInput::Param = input else {
						return Err(Error::new(
							ErrorKind::Custom(format!(
								"Expected identifier of numeric type, but got identifier {} of qubit type",
								id.text
							)),
							id.location.clone(),
						));
					};
				} else {
					return Err(Error::new(
						ErrorKind::Custom(format!(
							"Numeric identifiers are not supported outside of gate declarations, but got dentifier {}",
							id.text
						)),
						id.location.clone(),
					));
				}
			}
			ast::Expression::Integer { .. }
			| ast::Expression::Pi { .. }
			| ast::Expression::Real { .. } => {}
		}

		Ok(())
	}

	fn add_declaration(&mut self, declaration: &ast::Declaration) -> Result<(), Error> {
		if self
			.declarations
			.insert(declaration.name.text.clone(), declaration.clone())
			.is_some()
		{
			return Err(Error::new(
				ErrorKind::Custom(format!("Redeclaration of name {}", declaration.name.text)),
				declaration.name.location.clone(),
			));
		}

		Ok(())
	}

	fn add_gate(&mut self, gate_declaration: &ast::GateDeclaration) -> Result<(), Error> {
		assert!(self.gate_inputs.is_none());
		let mut gate_inputs = HashMap::new();
		for param in gate_declaration.params.iter() {
			if gate_inputs
				.insert(param.text.clone(), GateInput::Param)
				.is_some()
			{
				return Err(Error::new(
					ErrorKind::Custom(format!("Parameter name '{}' is already in use", param.text)),
					param.location.clone(),
				));
			}
		}

		for qarg in gate_declaration.qargs.iter() {
			if gate_inputs
				.insert(qarg.text.clone(), GateInput::QubitBinding(qarg.clone()))
				.is_some()
			{
				return Err(Error::new(
					ErrorKind::Custom(format!("Parameter name '{}' is already in use", qarg.text)),
					qarg.location.clone(),
				));
			}
		}

		self.gate_inputs = Some(gate_inputs);
		for operator in gate_declaration.quantum_operators.iter() {
			match operator {
				ast::GateOperation::UnitaryOperator(unitary_operator) => {
					self.process_unitary_operator(unitary_operator)?;
				}
				ast::GateOperation::Barrier(qargs) => {
					for qarg in qargs.iter() {
						match self.gate_inputs.as_ref().unwrap().get(&qarg.text) {
							Some(GateInput::Qarg(_)) | Some(GateInput::QubitBinding(_)) => {}
							Some(GateInput::Param) => {
								return Err(Error::new(
									ErrorKind::Custom(format!(
										"Expected qubit argument, got {} of numeric type insted",
										qarg.text
									)),
									qarg.location.clone(),
								));
							}
							_ => {
								return Err(Error::new(
									ErrorKind::UndefinedIdentifier(qarg.text.clone()),
									qarg.location.clone(),
								));
							}
						}
					}
				}
			}
		}

		self.gate_inputs = None;
		if self
			.gates
			.insert(
				gate_declaration.name.text.clone(),
				DefinedGate::Gate(gate_declaration.clone()),
			)
			.is_some()
		{
			return Err(Error::new(
				ErrorKind::Custom(format!(
					"Redefinition of gate '{}'",
					gate_declaration.name.text
				)),
				gate_declaration.location.clone(),
			));
		}

		Ok(())
	}

	fn add_opaque(&mut self, opaque: &ast::Opaque) -> Result<(), Error> {
		if self
			.gates
			.insert(
				opaque.name.text.clone(),
				DefinedGate::Opaque(opaque.clone()),
			)
			.is_some()
		{
			return Err(Error::new(
				ErrorKind::Custom(format!("Redefinition of gate '{}'", opaque.name.text)),
				opaque.location.clone(),
			));
		}

		assert!(self.gate_inputs.is_none());
		let mut gate_inputs = HashMap::new();
		for param in opaque.params.iter() {
			if gate_inputs
				.insert(param.text.clone(), GateInput::Param)
				.is_some()
			{
				return Err(Error::new(
					ErrorKind::Custom(format!("Parameter name '{}' is already in use", param.text)),
					param.location.clone(),
				));
			}
		}

		for qarg in opaque.qargs.iter() {
			if gate_inputs
				.insert(qarg.text.clone(), GateInput::QubitBinding(qarg.clone()))
				.is_some()
			{
				return Err(Error::new(
					ErrorKind::Custom(format!("Parameter name '{}' is already in use", qarg.text)),
					qarg.location.clone(),
				));
			}
		}

		Ok(())
	}

	fn argument_qubit_size(&self, argument: &ast::Argument) -> Result<Option<u64>, Error> {
		match argument {
			ast::Argument::Named { name } => {
				if let Some(gate_inputs) = self.gate_inputs.as_ref() {
					if let Some(gate_input) = gate_inputs.get(&name.text) {
						if let GateInput::Qarg(_) | GateInput::QubitBinding(_) = gate_input {
							return Ok(None);
						} else {
							return Err(Error::new(
								ErrorKind::Custom(format!(
									"Expected identifier of type Qubit, but got identifier {} of numeric type",
									name.text
								)),
								name.location.clone(),
							));
						};
					} else {
						return Err(Error::new(
							UndefinedIdentifier(name.text.clone()),
							name.location.clone(),
						));
					}
				}

				let Some(register) = self.declarations.get(&name.text) else {
					return Err(Error::new(
						ErrorKind::UndefinedIdentifier(name.text.clone()),
						name.location.clone(),
					));
				};

				if register.ty == ast::DeclarationType::Qubit {
					Ok(Some(register.size))
				} else {
					Err(Error::new(
						ErrorKind::Custom(format!(
							"Expected identifier of type Qubit, but got identifier {} of type bit",
							name.text
						)),
						name.location.clone(),
					))
				}
			}
			ast::Argument::Indexed { name, index } => {
				if self.gate_inputs.is_some() {
					return Err(Error::new(
						ErrorKind::Custom(String::from(
							"Can not index into registers inside of gate declaration",
						)),
						name.location.clone(),
					));
				}

				let Some(register) = self.declarations.get(&name.text) else {
					return Err(Error::new(
						ErrorKind::UndefinedIdentifier(name.text.clone()),
						name.location.clone(),
					));
				};

				if *index >= register.size {
					return Err(Error::new(
						ErrorKind::IndexOutOfBounds {
							index: *index,
							size: register.size,
						},
						name.location.clone(),
					));
				}

				if register.ty == ast::DeclarationType::Qubit {
					Ok(None)
				} else {
					Err(Error::new(
						ErrorKind::Custom(format!(
							"Expected identifier of type Qubit, but got identifier {} of type bit",
							name.text
						)),
						name.location.clone(),
					))
				}
			}
		}
	}

	fn is_type(&self, argument: &ast::Argument, ty: ast::DeclarationType) -> Result<(), Error> {
		match argument {
			ast::Argument::Named { name } => {
				if let Some(gate_inputs) = self.gate_inputs.as_ref() {
					if let Some(gate_input) = gate_inputs.get(&name.text) {
						if ty == ast::DeclarationType::Qubit {
							if let GateInput::Qarg(_) | GateInput::QubitBinding(_) = gate_input {
								return Ok(());
							} else {
								return Err(Error::new(
									ErrorKind::Custom(format!(
										"Expected type qubit, but got numberic gate argument {}",
										name.text
									)),
									name.location.clone(),
								));
							}
						} else {
							if let GateInput::Qarg { .. } | GateInput::QubitBinding(_) = gate_input
							{
								return Err(Error::new(
									ErrorKind::Custom(format!(
										"Expected type bit, but got qubit gate argument {}",
										name.text
									)),
									name.location.clone(),
								));
							} else {
								return Err(Error::new(
									ErrorKind::Custom(format!(
										"Expected type bit, but got numeric gate argument {}",
										name.text
									)),
									name.location.clone(),
								));
							}
						}
					} else {
						return Err(Error::new(
							UndefinedIdentifier(name.text.clone()),
							name.location.clone(),
						));
					}
				}

				let Some(register) = self.declarations.get(&name.text) else {
					return Err(Error::new(
						ErrorKind::UndefinedIdentifier(name.text.clone()),
						name.location.clone(),
					));
				};

				if register.ty != ty {
					match ty {
						ast::DeclarationType::Qubit => {
							return Err(Error::new(
								ErrorKind::Custom(format!(
									"Expected identifier of type Qubit, but got identifier {} of type Bit",
									name.text
								)),
								name.location.clone(),
							));
						}
						ast::DeclarationType::Bit => {
							return Err(Error::new(
								ErrorKind::Custom(format!(
									"Expected identifier of type Bit, but got identifier {} of type Qubit",
									name.text
								)),
								name.location.clone(),
							));
						}
					}
				}
			}
			ast::Argument::Indexed { name, index } => {
				if self.gate_inputs.is_some() {
					return Err(Error::new(
						ErrorKind::Custom(String::from(
							"Can not index into registers inside of gate declaration",
						)),
						name.location.clone(),
					));
				}

				let Some(register) = self.declarations.get(&name.text) else {
					return Err(Error::new(
						ErrorKind::UndefinedIdentifier(name.text.clone()),
						name.location.clone(),
					));
				};

				if register.ty != ty {
					match ty {
						ast::DeclarationType::Qubit => {
							return Err(Error::new(
								ErrorKind::Custom(format!(
									"Expected identifier of type Qubit, but got identifier {} of type Bit",
									name.text
								)),
								name.location.clone(),
							));
						}
						ast::DeclarationType::Bit => {
							return Err(Error::new(
								ErrorKind::Custom(format!(
									"Expected identifier of type Bit, but got identifier {} of type Qubit",
									name.text
								)),
								name.location.clone(),
							));
						}
					}
				}

				if *index >= register.size {
					return Err(Error::new(
						ErrorKind::IndexOutOfBounds {
							index: *index,
							size: register.size,
						},
						name.location.clone(),
					));
				}
			}
		}

		Ok(())
	}

	/// Checks if a and b have common qubits
	fn overlapping(&self, a: &ast::Argument, b: &ast::Argument) -> Result<bool, Error> {
		if let Some(gate_inputs) = self.gate_inputs.as_ref() {
			if a.is_indexed() || b.is_indexed() {
				return Err(Error::new(
					ErrorKind::Custom(String::from(
						"Can not index into registers inside of gate declaration",
					)),
					if a.is_indexed() {
						a.name().location.clone()
					} else {
						b.name().location.clone()
					},
				));
			}
			let Some(a) = gate_inputs.get(&a.name().text) else {
				return Err(Error::new(
					ErrorKind::UndefinedIdentifier(a.name().text.clone()),
					a.name().location.clone(),
				));
			};
			let Some(b) = gate_inputs.get(&b.name().text) else {
				return Err(Error::new(
					ErrorKind::UndefinedIdentifier(b.name().text.clone()),
					b.name().location.clone(),
				));
			};

			Ok(a == b)
		} else {
			Ok(a.name().text == b.name().text)
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
