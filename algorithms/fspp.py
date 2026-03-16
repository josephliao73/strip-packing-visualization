import json
import math
from scipy.optimize import linprog

with open("algorithm_input.json") as f:
    d = json.load(f)

bin_width = d["width_of_bin"]
rectangle_types = d["rectangle_list"]

type_keys = []
seen = set()
for rect in rectangle_types:
    key = (rect["width"], rect["height"])
    if key not in seen:
        seen.add(key)
        type_keys.append(key)

type_index = {key: i for i, key in enumerate(type_keys)}
num_types = len(type_keys)

configurations = []

def get_config(type_idx, counts, remaining_width):
    if type_idx == num_types:
        if not any(c > 0 for c in counts):
            return

        for i in range(num_types):
            w_i = type_keys[i][0]
            if w_i <= remaining_width + 1e-9:
                return

        configurations.append(counts.copy())
        return

    w_i = type_keys[type_idx][0]
    max_count = int((remaining_width + 1e-9) // w_i)

    for c in range(max_count, -1, -1):
        counts[type_idx] = c
        get_config(type_idx + 1, counts, remaining_width - c * w_i)

    counts[type_idx] = 0

get_config(0, [0] * num_types, bin_width)

unique_configurations = sorted(configurations)

m = len(unique_configurations)

c = [1.0] * m

A = [[0] * m for _ in range(num_types)]
for j, config in enumerate(unique_configurations):
    for i in range(num_types):
        A[i][j] = config[i]

b = []
for key in type_keys:
    total_quantity = 0
    height = key[1]
    for rect in rectangle_types:
        if (rect["width"], rect["height"]) == key:
            total_quantity += rect["quantity"]
    b.append(total_quantity * height)

A_ub = [[-value for value in row] for row in A]
b_ub = [-value for value in b]
bounds = [(0, None)] * m

res = linprog(c=c, A_ub=A_ub, b_ub=b_ub, bounds=bounds, method="highs")

if not res.success:
    raise RuntimeError(f"LP solve failed: {res.message}")

x_values = res.x.tolist()

eps = 1e-9
fractional_bands = []
current_y = 0.0

for j, xj in enumerate(x_values):
    if xj <= eps:
        continue

    config = unique_configurations[j]
    columns = []
    x_cursor = 0.0

    for i, count in enumerate(config):
        if count == 0:
            continue

        width_i, height_i = type_keys[i]

        for _ in range(count):
            columns.append({
                "type_index": i,
                "x": x_cursor,
                "width": width_i,
                "height": xj,
                "rect_height": height_i
            })
            x_cursor += width_i

    fractional_bands.append({
        "config_index": j,
        "fractional_height": xj,
        "y_bottom": current_y,
        "counts": config,
        "columns": columns
    })

    current_y += xj

print(fractional_bands)
print(len(fractional_bands))
fractional_total_height = current_y

placements = []
rounded_y = 0.0

for band in fractional_bands:
    xj = band["fractional_height"]
    config = band["counts"]

    rounded_band_height = 0.0
    for i, count in enumerate(config):
        if count > 0:
            _, h_i = type_keys[i]
            col_height = math.ceil((xj - eps) / h_i) * h_i
            rounded_band_height = max(rounded_band_height, col_height)

    x_cursor = 0.0
    for i, count in enumerate(config):
        if count == 0:
            continue

        w_i, h_i = type_keys[i]

        for _ in range(count):
            num_rects_in_column = math.ceil((xj - eps) / h_i)
            y_cursor = rounded_y

            for _ in range(num_rects_in_column):
                placements.append({
                    "x": x_cursor,
                    "y": y_cursor,
                    "width": w_i,
                    "height": h_i
                })
                y_cursor += h_i

            x_cursor += w_i

    rounded_y += rounded_band_height

total_height = rounded_y

output = {
    "bin_width": bin_width,
    "total_height": total_height,
    "placements": placements
}

print("DONE")
with open("bad.json", "w") as out:
    json.dump(output, out, indent=2)
