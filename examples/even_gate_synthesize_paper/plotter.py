# %%
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

px = 1/plt.rcParams['figure.dpi']
gate_size = [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]

width = 560 * 3
height = 455 * 3

# %% Plots for random
import math
# 3dplot "all to all" connectivity
# x > smaller exp count, y ^ smaller gate
# 								20
#								18
#								16
#								14
#								12
#								10
#								8
#								6
#								4
#								2
# 700 600 500 400 300 200 100

# connectivity: fully_connected, line, square_grid
# property: count, depth
def random_plot(connectivity, property, part):
	property_means = {}
	for gate_count in range(100, 800, 100):
		property_means[gate_count] = {}
		for gate_size in range(2, 22 , 2):
			path = f"./{connectivity}/random/{gate_count}/gate_size_{gate_size}.exp"
			df = pd.read_csv(path)
			col = "output_"
			if part != "total":
				col += part
				col += "_"
			col += property
			property_means[gate_count][gate_size] = df.loc[:, col].mean()

	x_values = [700, 600, 500, 400, 300, 200, 100]
	y_values = [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]
	x = [x_value for x_value in x_values for _ in range(len(y_values))]
	y = [y_value for _ in range(len(x_values)) for y_value in y_values]
	z = [0 for _ in range(len(x_values)) for _ in range(len(y_values))]
	dx = [ 100 for _ in range(len(x))]
	dy = [ 2 for _ in range(len(y))]
	dz = [property_means[x_value][y_value] for x_value in x_values for y_value in y_values]

	# (424*px, 424*px)
	fig, ax = plt.subplots(figsize=(560/80, 455/80), dpi=80, subplot_kw={"projection": "3d"})

	#ax.plot_surface(X, Y, Z, cmap="plasma")

	s = (math.floor(max(dz) / 10000) + 1) * 10000
	for (x, y, z, dx, dy, dz) in zip(x, y, z, dx, dy, dz):
		a = ax.bar3d(x, y, z, dx, dy, dz, color=(0.6 + 0.4 * dz / s, 1.0 - 1.0 * dz / s , 0))
		a._sort_zpos = -dy

	ax.set_xticks([180, 280, 380, 480, 580, 680, 780], labels=["100", "200", "300", "400", "500", "600", "700"])
	ax.set_yticks([2, 4, 6, 8, 10, 12, 14, 16, 18, 20], labels=["2", "4", "6", "8", "10", "12", "14", "16", "18", "20"])

	#ax.view_init(azim=-45, elev=45)
	ax.set_xlim(800, 100)
	ax.set_ylim(20, 2)
	ax.set_zlim(0, s)

	match connectivity:
		case "fully_connected":
			conn_name = "Full Connectivity"
		case "line":
			conn_name = "Linear Connectivity"
		case "square_grid":
			conn_name = "Square Grid Connectivity"
	plt.title(f"{part.capitalize()} Multi-Qubit Gate {property.capitalize()} With {conn_name}")
	plt.xlabel("Pauli Gadgets")
	plt.ylabel("Target Gadget Size")
	ax.set_zlabel(f"Multi-Qubit Gate {property.capitalize()}")

	from pathlib import Path
	Path("./figures").mkdir(exist_ok=True)
	fig.savefig(f'./figures/{connectivity}_{part}_{property}.png', dpi=80 * 4)
	plt.show()

# %%
# connectivity: fully_connected, line, square_grid
for connectivity in ["fully_connected", "line", "square_grid"]:
	for part in ["base", "tableau", "total"]:
		for property in ["count", "depth"]:
			random_plot(connectivity, property, part)

# %% Tables for molecules

# molecules: list of molecule names
# gate_sizes: list of numbers
# connectivity: fully_connected, line, square_grid
# property: count, depth
def create_table(molecules, gate_sizes, connectivity, property):
	header = "Molecule\\Gate Size"
	rows = [molecule for molecule in molecules]

	for gate_size in gate_sizes:
		header += f" & {gate_size} Qubits"
		path = f"./{connectivity}/molecules/gate_size_{gate_size}.exp"
		df = pd.read_csv(path)
		for i, molecule in enumerate(molecules):
			name = f"./datasets/molecules_small/{molecule}.exp"
			value = df.loc[df.name == name, f"output_gate_{property}"].values[0]
			rows[i] += f" & {value}"

	rows = "\n".join([f"	{row.replace("_", "\\_")} \\\\" for row in rows])
	table = f"""
\\begin{{tabular}}{{| l | c {"| c " * len(gate_sizes)}|}}
	\\hline
	{header} \\\\
	\\hline
	\\hline
{rows}
	\\hline
\\end{{tabular}}
"""

	print(table)

create_table(["H2_JW_631g", "LiH_BK_sto3g"], [2, 4, ], "fully_connected", "count")
# %%
