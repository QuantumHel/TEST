use bits::Bits;

use crate::pauli::{Negate, PauliExp};

/// How many "layers" we need
pub fn gate_depth<A: Negate, F: Fn(&PauliExp<A>) -> bool>(
	circuit: &[PauliExp<A>],
	filter: F,
) -> usize {
	let mut layers: Vec<Bits> = Vec::new();

	for exp in circuit.iter() {
		if !filter(exp) {
			continue;
		}

		let mut stop: Option<usize> = None;
		'layer: for (i, layer) in layers.iter().enumerate().rev() {
			for qubit in exp.string.targets() {
				if layer.get(qubit) {
					stop = Some(i);
					break 'layer;
				}
			}
		}

		let layer: usize = match stop {
			Some(i) => {
				if (i + 1) == layers.len() {
					layers.push(Bits::new());
				}
				i + 1
			}
			None => {
				if layers.is_empty() {
					layers.push(Bits::new());
				}
				0
			}
		};

		for (qubit, _) in exp.string.letters() {
			layers[layer].set(qubit, true);
		}
	}

	layers.len()
}

pub fn gate_count<A: Negate, F: Fn(&PauliExp<A>) -> bool>(
	gates: &[PauliExp<A>],
	filter: F,
) -> usize {
	gates.iter().filter(|p| filter(*p)).count()
}

pub fn multi_qubit_filter<A: Negate>(exp: &PauliExp<A>) -> bool {
	exp.len() >= 2
}
