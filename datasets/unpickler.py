# %%
import glob
import sys
import pickle
from pauliopt.utils import pi, AngleVar, Angle
from numbers import Number

# %%
input_path = sys.argv[1]
output_path = sys.argv[2]

input_files = glob.glob(input_path  + "*.pickle")
print(input_files)

for file in input_files:
	output_name = file.split("/")[-1].replace(".pickle", ".exp")
	output = open(output_path + output_name, "x")

	with open(file, "rb") as handle:
		pp = pickle.load(handle)

		for gadget in pp.pauli_gadgets:
			if isinstance(gadget.angle, Angle):
				output.write(f"{float(gadget.angle.value)};")
				print("hi")
			elif isinstance(gadget.angle, AngleVar):
				output.write(f"{gadget.angle};")
			elif isinstance(gadget.angle, Number):
				output.write(f"{gadget.angle};")
			else:
				raise Exception("Unknown angle")
			for letter in gadget.paulis:
				output.write(letter.value)
			output.write("\n")
	output.close()

# %%
