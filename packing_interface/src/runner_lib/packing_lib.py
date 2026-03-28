import json
import math


def make_output(bin_width, total_height, placements):
    return json.dumps({
        "bin_width": bin_width,
        "total_height": total_height,
        "placements": [{"x": x, "y": y, "width": w, "height": h} for x, y, w, h in placements]
    })


def total_height(placements):
    return max((y + h for _, y, _, h in placements), default=0)


def output_from_placements(bin_width, placements):
    return make_output(bin_width, total_height(placements), placements)


def expand_items(rectangle_list):
    items = []
    for item in rectangle_list:
        w, h, q = item
        for _ in range(q):
            items.append({
                "width": w,
                "height": h
            })
    return items


def expand_type_counts(type_keys, quantities):
    items = []
    for (width, height), quantity in zip(type_keys, quantities):
        for _ in range(quantity):
            items.append({"width": width, "height": height})
    return items


def stack_vertically(items):
    placements = []
    current_y = 0
    for item in items:
        placements.append((0, current_y, item["width"], item["height"]))
        current_y += item["height"]
    return placements


def sort_by_height(items, descending=True):
    return sorted(items, key=lambda x: x["height"], reverse=descending)


def sort_by_width(items, descending=True):
    return sorted(items, key=lambda x: x["width"], reverse=descending)


def sort_by_area(items, descending=True):
    return sorted(items, key=lambda x: x["width"] * x["height"], reverse=descending)


def sort_by_quantity(rectangle_list, descending=True):
    return sorted(rectangle_list, key=lambda x: x["quantity"], reverse=descending)


def dedup_rectangles(rectangle_list):
    totals = {}
    is_dict = isinstance(rectangle_list[0], dict) if rectangle_list else False
    for rect in rectangle_list:
        if is_dict:
            w, h, q = rect["width"], rect["height"], rect["quantity"]
        else:
            w, h, q = rect[0], rect[1], rect[2]
        key = (w, h)
        totals[key] = totals.get(key, 0) + q
    if is_dict:
        return [{"width": w, "height": h, "quantity": q} for (w, h), q in totals.items()]
    return [[w, h, q] for (w, h), q in totals.items()]


def get_type_keys(rectangle_list):
    totals = {}
    for rect in rectangle_list:
        if isinstance(rect, dict):
            w, h, q = rect["width"], rect["height"], rect["quantity"]
        else:
            w, h, q = rect[0], rect[1], rect[2]
        key = (w, h)
        totals[key] = totals.get(key, 0) + q
    type_keys = list(totals.keys())
    quantities = [totals[k] for k in type_keys]
    return type_keys, quantities


def get_configurations(type_keys, bin_width):
    configurations = []
    num_types = len(type_keys)

    def recurse(type_idx, counts, remaining_width):
        if type_idx == num_types:
            if any(c > 0 for c in counts):
                configurations.append(counts.copy())
            return
        w_i = type_keys[type_idx][0]
        max_count = remaining_width // w_i
        for c in range(int(max_count) + 1):
            counts[type_idx] = c
            recurse(type_idx + 1, counts, remaining_width - c * w_i)
        counts[type_idx] = 0

    recurse(0, [0] * num_types, bin_width)
    return sorted(configurations)


def place_items_nfdh(items, bin_width, start_y=0.0):
    items = sort_by_height(items)
    placements = []
    current_level_y = start_y
    current_level_height = 0
    current_level_used_width = 0

    for item in items:
        width = item["width"]
        height = item["height"]

        if current_level_used_width == 0:
            current_level_height = height

        if current_level_used_width + width <= bin_width:
            placements.append((current_level_used_width, current_level_y, width, height))
            current_level_used_width += width
            continue

        current_level_y += current_level_height
        current_level_height = height
        current_level_used_width = width
        placements.append((0, current_level_y, width, height))

    return placements


def place_items_ffdh(items, bin_width, start_y=0.0):
    items = sort_by_height(items)
    levels = []
    placements = []
    current_y = start_y

    for item in items:
        width = item["width"]
        height = item["height"]
        placed = False

        for level in levels:
            if level["used_width"] + width <= bin_width:
                placements.append((level["used_width"], level["y"], width, height))
                level["used_width"] += width
                placed = True
                break

        if placed:
            continue

        levels.append({"height": height, "used_width": width, "y": current_y})
        placements.append((0, current_y, width, height))
        current_y += height

    return placements


def find_bottom_left_position(placements, bin_width, width, height, max_height=None):
    candidate_xs = {0.0}
    for px, _, pw, _ in placements:
        candidate_xs.add(px + pw)

    best_position = None
    for candidate_x in sorted(candidate_xs):
        if candidate_x + width > bin_width:
            continue

        candidate_y = 0.0
        for px, py, pw, ph in placements:
            overlaps_x = candidate_x < px + pw and candidate_x + width > px
            if overlaps_x:
                candidate_y = max(candidate_y, py + ph)

        if max_height is not None and candidate_y + height > max_height:
            continue

        position = (candidate_y, candidate_x)
        if best_position is None or position < best_position:
            best_position = position

    if best_position is None:
        return None

    y, x = best_position
    return x, y


def nfdh(rectangles, bin_width):
    return place_items_nfdh(expand_items(rectangles), bin_width)


def ffdh(rectangles, bin_width):
    return place_items_ffdh(expand_items(rectangles), bin_width)


def configuration_matrix(configurations, num_types):
    matrix = [[0] * len(configurations) for _ in range(num_types)]
    for column, config in enumerate(configurations):
        for row in range(num_types):
            matrix[row][column] = config[row]
    return matrix


def get_type_demands(rectangle_list, type_keys=None):
    if type_keys is None:
        type_keys, quantities = get_type_keys(rectangle_list)
    else:
        quantity_map = {key: 0 for key in type_keys}
        for rect in rectangle_list:
            if isinstance(rect, dict):
                key = (rect["width"], rect["height"])
                quantity_map[key] += rect["quantity"]
            else:
                key = (rect[0], rect[1])
                quantity_map[key] += rect[2]
        quantities = [quantity_map[key] for key in type_keys]
    demands = [quantity * height for (_, height), quantity in zip(type_keys, quantities)]
    return type_keys, quantities, demands


def solve_fractional_strip_cover(rectangle_list, bin_width):
    from scipy.optimize import linprog

    type_keys, _, demands = get_type_demands(rectangle_list)
    configurations = get_configurations(type_keys, bin_width)
    if not configurations:
        return type_keys, [], []

    matrix = configuration_matrix(configurations, len(type_keys))
    result = linprog(
        c=[1.0] * len(configurations),
        A_ub=[[-value for value in row] for row in matrix],
        b_ub=[-value for value in demands],
        bounds=[(0, None)] * len(configurations),
        method="highs",
    )
    if not result.success:
        raise RuntimeError(f"LP solve failed: {result.message}")

    return type_keys, configurations, result.x.tolist()


def place_strip_band(type_keys, counts, strip_height, remaining_quantities, start_y=0.0, eps=1e-9):
    placements = []
    band_height = 0.0
    x_cursor = 0.0

    for index, count in enumerate(counts):
        width, height = type_keys[index]
        if count <= 0:
            continue

        per_column = math.floor((strip_height + eps) / height)
        per_column = min(per_column, remaining_quantities[index] // count if count else 0)

        if per_column <= 0:
            x_cursor += count * width
            continue

        band_height = max(band_height, per_column * height)
        for _ in range(count):
            y_cursor = start_y
            for _ in range(per_column):
                placements.append((x_cursor, y_cursor, width, height))
                y_cursor += height
                remaining_quantities[index] -= 1
            x_cursor += width

    return placements, band_height
