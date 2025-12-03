# %% Initialization
import glob
from qiskit.circuit import QuantumCircuit
from qiskit.circuit.library import PauliEvolutionGate
from qiskit.quantum_info import SparsePauliOp
from qiskit.quantum_info import Statevector
import math

QUBITS = 20
tol = 1e-15 # 16 seems to work so 15 is safe?

def read_exp(path):
	file = open(path)

	num_qubits = 0
	gates = []
	for line in file:
		parts = line.strip().split(";")
		angle = -float(parts[0]) * math.pi
		string = parts[1]

		operator = SparsePauliOp(string[::-1])
		#for letter in string[1:]:
		#	operator = operator ^ SparsePauliOp(letter)
		
		gate = PauliEvolutionGate(operator, time=angle)
		qubits = len(string)
		gates.append((gate, qubits))
		if qubits > num_qubits:
			num_qubits = qubits

	if len(gates) == 0:
		raise Exception("no gates found")
	
	if num_qubits > QUBITS:
		raise Exception("Too many qubits")
	
	circuit = QuantumCircuit(QUBITS)

	for (gate, qubits) in gates:
		circuit.append(gate, range(qubits))

	return circuit

def read_order(path):
	return read_exp(path + ".order")


def check_equal(c1, c2):
	v1 = Statevector.from_instruction(c1)
	v2 = Statevector.from_instruction(c2)

	return v1.equiv(v2, atol=tol) # may need atol= or rtol=

# %% Check
files = glob.glob("./*.exp")

for file in files:
	circuit = read_exp(file)
	order = read_order(file)

	# for (a, b) in zip(Statevector.from_instruction(order), Statevector.from_instruction(circuit)):
	# 	if (not math.isclose(a.real, b.real, abs_tol=tol, rel_tol=0)) or (not math.isclose(a.imag, b.imag, abs_tol=tol, rel_tol=0)):
	# 		print(a, b)
	print(check_equal(circuit, order))

# %%
