use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use test_transpiler::experiment::random_exp;
use test_transpiler::pauli::PauliExp;

fn main() {
	for l in 1..=7 {
		let len = l * 100;
		for i in 0..100 {
			let mut rng = ChaCha8Rng::seed_from_u64(i);
			let exps = (0..len)
				.map(|_| random_exp(100, &mut rng))
				.collect::<Vec<_>>();
			PauliExp::write_exp_file(
				&exps,
				&format!("./datasets/random/{len}/random{i}_{len}.exp"),
			);
		}
	}
}
