# %% Initialization
import glob
from qiskit.circuit import QuantumCircuit
from qiskit.circuit.library import PauliEvolutionGate
from qiskit.quantum_info import SparsePauliOp
from qiskit.quantum_info import Statevector
import math

def read_exp(path):
	file = open(path)

	gates = []
	for line in file:
		parts = line.strip().split(";")
		angle = -float(parts[0]) * math.pi
		string = parts[1]

		operator = SparsePauliOp(string[0])
		for letter in string[1:]:
			operator = operator ^ SparsePauliOp(letter)
		
		gate = PauliEvolutionGate(operator, time=angle)
		gates.append(gate)

	if len(gates) == 0:
		raise Exception("no gates found")
	
	n_qubits = gates[0].num_qubits
	circuit = QuantumCircuit(n_qubits)

	for gate in gates:
		circuit.append(gate, range(n_qubits))

	return circuit

def read_order(path):
	return read_exp(path + ".order")


def check_equal(c1, c2):
	v1 = Statevector.from_instruction(c1)
	v2 = Statevector.from_instruction(c2)

	return v1.equiv(v2) # may need atol= or rtol=

# %% Check
files = glob.glob("./*.exp")

for file in files:
	circuit = read_exp(file)
	order = read_order(file)
	# print()
	# print(Statevector.from_instruction(circuit))
	# print(Statevector.from_instruction(order))
	print(check_equal(circuit, order))

# %%
