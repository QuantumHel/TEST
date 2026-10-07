use crate::error::Location;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Program {
	pub(crate) version: f64,
	pub(crate) program: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Statement {
	Declaration(Declaration),
	GateDeclaration(GateDeclaration),
	Opaque(Opaque),
	QuantumOperator(QuantumOperator),
	If {
		id: Id,
		integer: u64,
		quantum_operator: QuantumOperator,
		location: Location,
	},
	Barrier(Vec<Argument>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DeclarationType {
	Qubit,
	Bit,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Declaration {
	pub(crate) ty: DeclarationType,
	pub(crate) name: Id,
	pub(crate) size: u64,
	pub(crate) location: Location,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GateDeclaration {
	pub(crate) name: Id,
	pub(crate) params: Vec<Id>,
	pub(crate) qargs: Vec<Id>,
	pub(crate) quantum_operators: Vec<GateOperation>,
	pub(crate) location: Location,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum GateOperation {
	/// All arguments inside of here need to be checked to not include indexed
	UnitaryOperator(UnitaryOperator),
	Barrier(Vec<Id>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Opaque {
	pub(crate) name: Id,
	pub(crate) params: Vec<Id>,
	pub(crate) qargs: Vec<Id>,
	pub(crate) location: Location,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum QuantumOperator {
	UnitaryOperator(UnitaryOperator),
	Measure {
		source: Argument,
		target: Argument,
		location: Location,
	},
	Reset {
		target: Argument,
		location: Location,
	},
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UnitaryOperator {
	U {
		theta: Expression,
		phi: Expression,
		lambda: Expression,
		target: Argument,
		location: Location,
	},
	Cx {
		control: Argument,
		target: Argument,
		location: Location,
	},
	CustomGate {
		name: Id,
		params: Vec<Expression>,
		qargs: Vec<Argument>,
		location: Location,
	},
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Argument {
	Named { name: Id },
	Indexed { name: Id, index: u64 },
}

impl Argument {
	pub(crate) fn name(&self) -> &Id {
		match self {
			Argument::Named { name } | Argument::Indexed { name, .. } => name,
		}
	}

	pub(crate) fn is_indexed(&self) -> bool {
		matches!(self, Argument::Indexed { .. })
	}
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expression {
	Real {
		value: f64,
		location: Location,
	},
	Integer {
		value: u64,
		location: Location,
	},
	Pi {
		location: Location,
	},
	Id(Id),
	BinaryOperator {
		operator: BinaryOperator,
		left: Box<Expression>,
		right: Box<Expression>,
		location: Location,
	},
	UnaryOperator {
		operator: UnaryOperator,
		argument: Box<Expression>,
		location: Location,
	},
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Id {
	pub(crate) text: String,
	pub(crate) location: Location,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BinaryOperator {
	Addition,
	Subtraction,
	Multiplication,
	Division,
	Power,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UnaryOperator {
	Sin,
	Cos,
	Tan,
	Exp,
	Ln,
	Sqrt,
	Negation,
}
