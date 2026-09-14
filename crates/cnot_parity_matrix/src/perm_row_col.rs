use circuit::{Circuit, gates::CNot};
use core::{Compiler, QubitMapping, connectivity::Connectivity};

use crate::{ParityMatrix, TwoQubitEdge};

/// An implementation of the PermRowCol algorithm from
/// https://arxiv.org/abs/2205.00724v4
pub struct PermRowCol;

impl Compiler<ParityMatrix, (Circuit<CNot>, QubitMapping), Connectivity<TwoQubitEdge>>
	for PermRowCol
{
	fn compile(
		&self,
		input: ParityMatrix,
		device: &Connectivity<TwoQubitEdge>,
	) -> (Circuit<CNot>, QubitMapping) {
		todo!()
	}
}
