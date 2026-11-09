use crate::CliffordTableau;
use pauli::PauliLetter;

pub fn print_tableau(tableau: &CliffordTableau) {
	println!("Tableau:");
	for row in 0..tableau.size() {
		let mut row_str = String::new();
		for qubit in 0..tableau.size() {
			row_str.push(to_char(tableau.get_x_row(row).get(qubit)));
		}
		row_str.push(if tableau.get_x_signs().get(row) {
			'-'
		} else {
			'+'
		});
		println!("{}", row_str);
	}
	println!(" ");
	for row in 0..tableau.size() {
		let mut row_str = String::new();
		for qubit in 0..tableau.size() {
			row_str.push(to_char(tableau.get_z_row(row).get(qubit)));
		}
		row_str.push(if tableau.get_z_signs().get(row) {
			'-'
		} else {
			'+'
		});
		println!("{}", row_str);
	}
}

fn to_char(letter: PauliLetter) -> char {
	match letter {
		PauliLetter::I => ' ',
		PauliLetter::X => 'X',
		PauliLetter::Y => 'Y',
		PauliLetter::Z => 'Z',
	}
}

pub fn to_str(tableau: &CliffordTableau) -> String {
	let mut result = String::new();
	for row in 0..tableau.size() {
		if row == 0 {
			result.push_str("X: ");
		} else {
			result.push_str("   ");
		}
		for qubit in 0..tableau.size() {
			result.push(to_char(tableau.get_x_row(row).get(qubit)));
		}
		result.push(if tableau.get_x_signs().get(row) {
			'-'
		} else {
			'+'
		});
		result.push('\n');
	}
	for row in 0..tableau.size() {
		if row == 0 {
			result.push_str("Z: ");
		} else {
			result.push_str("   ");
		}
		for qubit in 0..tableau.size() {
			result.push(to_char(tableau.get_z_row(row).get(qubit)));
		}
		result.push(if tableau.get_z_signs().get(row) {
			'-'
		} else {
			'+'
		});
		result.push('\n');
	}
	result
}
