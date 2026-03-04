# %%
import numpy as np
import pandas as pd
import copy

# %%
base_data = [
	["H2_P_sto3g", 2],
	["H2_BK_sto3g", 4],
	["H2_JW_sto3g", 4],
	["H2_P_631g", 6],
	["H4_P_sto3g", 6],
	["H2_JW_631g", 8],
	["H2_BK_631g", 8],
	["H4_BK_sto3g", 8],
	["H4_JW_sto3g", 8],
	["NH_BK_sto3g", 12],
	["LiH_BK_sto3g", 12],
	["LiH_JW_sto3g", 12],
	["NH_JW_sto3g", 12],
	["H2O_P_sto3g", 12],
	["BeH2_P_sto3g", 12],
	["CH2_P_sto3g", 12],
	["H2O_JW_sto3g", 14],
	["H2O_BK_sto3g", 14],
	["CH2_BK_sto3g", 14],
	["BeH2_JW_sto3g", 14],
	["BeH2_BK_sto3g", 14],
	["CH2_JW_sto3g", 14],
	["H4_P_631g", 14],
	["H4_JW_631g", 16],
	["H4_BK_631g", 16],
	["NH3_JW_sto3g", 16],
	["NH3_BK_sto3g", 16],
	["HCl_JW_sto3g", 20],
	["HCl_BK_sto3g", 20],
]

for molecule in base_data:
	path = f"./fully_connected/molecules/gate_size_2.exp"
	df = pd.read_csv(path)
	count = df.loc[df["name"] == f"./datasets/molecules_small/{molecule[0]}.exp"]["input_count"].values[0]
	depth = df.loc[df["name"] == f"./datasets/molecules_small/{molecule[0]}.exp"]["input_depth"].values[0]
	molecule.append(count.item())
	molecule.append(depth.item())

# %%
for connectivity in ["fully_connected", "line", "square_grid"]:
	data = copy.deepcopy(base_data)
	for gate_size in [2, 4, 6, 10, 20]:
		path = f"./{connectivity}/molecules/gate_size_{gate_size}.exp"
		df = pd.read_csv(path)
		for molecule in data:
			count = df.loc[df["name"] == f"./datasets/molecules_small/{molecule[0]}.exp"]["output_count"].values[0]
			depth = df.loc[df["name"] == f"./datasets/molecules_small/{molecule[0]}.exp"]["output_depth"].values[0]
			molecule.append(count.item())
			molecule.append(depth.item())
	
	for molecule in data:
		molecule[0] = molecule[0].replace("_", "\\_")
		print(*molecule, sep=" & ", end="\\\\\\hline\n")
	
	for _ in range(3):
		print()
# %%
