use circuit::{
	Circuit, RandomGate,
	gates::{CNot, CNotOpaque, H, HOpaque, Rz, X, XOpaque, Y, YOpaque, Z, ZOpaque},
};
use openqasm2::{
	OpaqueFunction, OpaqueFunctionDefinition, OpenQasm2Config, OpenQasm2Frontend,
	VirtualOpenqasmFile,
};
use rand::{
	RngExt,
	distr::{Distribution, StandardUniform},
};
use simulator::{Complex, Simulatable};
use std::{num::NonZero, ops::AddAssign};

use super::squirrel::Squirrel;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuarterPi(pub u32);

impl Distribution<QuarterPi> for StandardUniform {
	fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> QuarterPi {
		QuarterPi(rng.random_range(..8))
	}
}

impl AddAssign for QuarterPi {
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0;
		self.0 %= 8;
	}
}

#[derive(Debug, Clone, Copy)]
pub enum CNotRzXYH {
	CNot(CNot),
	Rz(Rz<QuarterPi>),
	X(X),
	Y(Y),
	H(H),
}

impl CNotRzXYH {
	/// Returns the index of the highest used qubit + 1
	pub fn n_required_qubits(&self) -> usize {
		1 + match self {
			Self::CNot(cnot) => cnot.target().max(cnot.control()),
			Self::Rz(rz) => rz.target,
			Self::X(x) => x.target,
			Self::Y(y) => y.target,
			Self::H(h) => h.target,
		}
	}

	/// Creates an openQasm frontend for a [Circuit] of [CNotRzXYH] gates.
	///
	/// Ignores imports.
	///
	/// Supported gates:
	/// - 'cnot'
	/// - 'x'
	/// - 'y'
	/// - 'h'
	/// - 't' (converted to [CNotRzXYH::Rz])
	/// - 's' (converted to [CNotRzXYH::Rz])
	/// - 'z' (converted to [CNotRzXYH::Rz])
	/// - 'sdg' (converted to [CNotRzXYH::Rz])
	/// - 'tdg' (converted to [CNotRzXYH::Rz])
	/// - 'cxx' (decompposed like in figure 13 in https://arxiv.org/pdf/1206.0758)
	pub fn open_qasm2_frontend() -> OpenQasm2Frontend<Circuit<Self>> {
		let mut file: VirtualOpenqasmFile<Circuit<Self>> = VirtualOpenqasmFile::default();
		file.add_opaque(CNotOpaque).unwrap();
		file.add_opaque(XOpaque).unwrap();
		file.add_opaque(YOpaque).unwrap();
		file.add_opaque(HOpaque).unwrap();
		file.add_opaque(TOpaque).unwrap();
		file.add_opaque(SOpaque).unwrap();
		file.add_opaque(ZOpaque).unwrap();
		file.add_opaque(SdgOpaque).unwrap();
		file.add_opaque(TdgOpaque).unwrap();

		// From figure 13 in https://arxiv.org/pdf/1206.0758
		file.add_text(
			"gate ccx a, b, c {
			h c;
			t a; t b; t c;
			cx b, a;
			cx c, b;
			cx a, c;
			tdg b;
			cx a, b;
			tdg a; tdg b; t c;
			cx c, b;
			cx a, c;
			cx b, a;
			h c;
		}",
		);

		OpenQasm2Frontend::new(OpenQasm2Config {
			default_file: Some(file),
			cx: Some(Box::new(CNotOpaque)),
			ignore_imports: true,
		})
	}
}

impl Simulatable<Squirrel> for CNotRzXYH {
	fn matrix(&self) -> [Complex<Squirrel>; 4] {
		match &self {
			CNotRzXYH::CNot(cnot) => cnot.matrix(),
			CNotRzXYH::Rz(rz) => rz.matrix(),
			CNotRzXYH::X(x) => x.matrix(),
			CNotRzXYH::Y(y) => y.matrix(),
			CNotRzXYH::H(h) => h.matrix(),
		}
	}

	fn target(&self) -> usize {
		match &self {
			CNotRzXYH::CNot(cnot) => cnot.target(),
			CNotRzXYH::Rz(rz) => rz.target(),
			CNotRzXYH::X(x) => x.target(),
			CNotRzXYH::Y(y) => y.target(),
			CNotRzXYH::H(h) => h.target(),
		}
	}

	fn controls(&self) -> Vec<usize> {
		match &self {
			CNotRzXYH::CNot(cnot) => cnot.controls(),
			CNotRzXYH::Rz(rz) => rz.controls(),
			CNotRzXYH::X(x) => x.controls(),
			CNotRzXYH::Y(y) => y.controls(),
			CNotRzXYH::H(h) => h.controls(),
		}
	}
}

impl RandomGate for CNotRzXYH {
	fn random<R: rand::prelude::Rng>(n_qubits: usize, rng: &mut R) -> Self {
		match rng.random_range(0..5) {
			0 => Self::CNot(CNot::random(n_qubits, rng)),
			1 => Self::Rz(Rz::random(n_qubits, rng)),
			2 => Self::X(X::random(n_qubits, rng)),
			3 => Self::Y(Y::random(n_qubits, rng)),
			_ => Self::H(H::random(n_qubits, rng)),
		}
	}
}

impl From<CNot> for CNotRzXYH {
	fn from(value: CNot) -> Self {
		CNotRzXYH::CNot(value)
	}
}

impl Simulatable<Squirrel> for CNot {
	fn matrix(&self) -> [Complex<Squirrel>; 4] {
		[
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::one(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::one(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
		]
	}

	fn controls(&self) -> Vec<usize> {
		vec![self.control()]
	}

	fn target(&self) -> usize {
		self.target()
	}
}

impl From<Rz<QuarterPi>> for CNotRzXYH {
	fn from(value: Rz<QuarterPi>) -> Self {
		CNotRzXYH::Rz(value)
	}
}

impl From<Z> for CNotRzXYH {
	fn from(value: Z) -> Self {
		CNotRzXYH::Rz(Rz {
			angle: QuarterPi(4),
			target: value.target,
		})
	}
}

impl Simulatable<Squirrel> for Rz<QuarterPi> {
	fn matrix(&self) -> [Complex<Squirrel>; 4] {
		[
			Complex {
				re: Squirrel::one(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
			Complex {
				re: match self.angle.0 % 8 {
					0 => Squirrel::one(),
					1 | 7 => Squirrel::divided_by_sqrt_2(),
					2 | 6 => Squirrel::zero(),
					3 | 5 => -Squirrel::divided_by_sqrt_2(),
					4 => -Squirrel::one(),
					_ => unreachable!(),
				},
				im: match self.angle.0 % 8 {
					0 | 4 => Squirrel::zero(),
					1 | 3 => Squirrel::divided_by_sqrt_2(),
					2 => Squirrel::one(),
					5 | 7 => -Squirrel::divided_by_sqrt_2(),
					6 => -Squirrel::one(),
					_ => unreachable!(),
				},
			},
		]
	}

	fn controls(&self) -> Vec<usize> {
		Vec::new()
	}

	fn target(&self) -> usize {
		self.target
	}
}

macro_rules! rz_opaque {
	($opaque:ident, $name:literal, $rotation:literal) => {
		struct $opaque;

		impl OpaqueFunction<Circuit<CNotRzXYH>> for $opaque {
			fn definition(&self) -> OpaqueFunctionDefinition {
				OpaqueFunctionDefinition {
					name: String::from($name),
					n_params: 0,
					n_qargs: NonZero::new(1).unwrap(),
				}
			}

			fn insert_gate(&self, qargs: Vec<usize>, ir: &mut Circuit<CNotRzXYH>) {
				let gate: Rz<QuarterPi> = Rz {
					angle: QuarterPi($rotation),
					target: qargs[0],
				};
				ir.push(CNotRzXYH::Rz(gate));
			}

			fn type_check(&self, _: Vec<usize>) -> Result<(), &'static str> {
				Ok(())
			}
		}
	};
}
rz_opaque!(TOpaque, "t", 1);
rz_opaque!(SOpaque, "s", 2);
rz_opaque!(SdgOpaque, "sdg", 6);
rz_opaque!(TdgOpaque, "tdg", 7);

impl From<X> for CNotRzXYH {
	fn from(value: X) -> Self {
		CNotRzXYH::X(value)
	}
}

impl Simulatable<Squirrel> for X {
	fn matrix(&self) -> [Complex<Squirrel>; 4] {
		[
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::one(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::one(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
		]
	}

	fn controls(&self) -> Vec<usize> {
		Vec::new()
	}

	fn target(&self) -> usize {
		self.target
	}
}

impl From<Y> for CNotRzXYH {
	fn from(value: Y) -> Self {
		CNotRzXYH::Y(value)
	}
}

impl Simulatable<Squirrel> for Y {
	fn matrix(&self) -> [Complex<Squirrel>; 4] {
		[
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::zero(),
				im: -Squirrel::one(),
			},
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::one(),
			},
			Complex {
				re: Squirrel::zero(),
				im: Squirrel::zero(),
			},
		]
	}

	fn controls(&self) -> Vec<usize> {
		Vec::new()
	}

	fn target(&self) -> usize {
		self.target
	}
}

impl From<H> for CNotRzXYH {
	fn from(value: H) -> Self {
		CNotRzXYH::H(value)
	}
}

impl Simulatable<Squirrel> for H {
	fn matrix(&self) -> [Complex<Squirrel>; 4] {
		[
			Complex {
				re: Squirrel::divided_by_sqrt_2(),
				im: Squirrel::zero(),
			},
			Complex {
				re: Squirrel::divided_by_sqrt_2(),
				im: -Squirrel::zero(),
			},
			Complex {
				re: Squirrel::divided_by_sqrt_2(),
				im: Squirrel::zero(),
			},
			Complex {
				re: -Squirrel::divided_by_sqrt_2(),
				im: Squirrel::zero(),
			},
		]
	}

	fn controls(&self) -> Vec<usize> {
		Vec::new()
	}

	fn target(&self) -> usize {
		self.target
	}
}
