import json

def make_output(bin_width, total_height, placements):
    return json.dumps({
        "bin_width": bin_width,
        "total_height": total_height,
        "placements": [{"x": x, "y": y, "width": w, "height": h} for x, y, w, h in placements]
    })

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

def sort_by_height(items, descending=True):
    return sorted(items, key=lambda x: x["height"], reverse=descending)

def sort_by_width(items, descending=True):
    return sorted(items, key=lambda x: x["width"], reverse=descending)

def sort_by_area(items, descending=True):
    return sorted(items, key=lambda x: x["width"] * x["height"], reverse=descending)

def sort_by_quantity(rectangle_list, descending=True):
    return sorted(rectangle_list, key=lambda x: x["quantity"], reverse=descending)

def dedup_rectangles(rectangle_list):
    """
    Merge duplicate rectangle types by summing their quantities.
    rectangle_list: list of [w, h, q] or dicts with width/height/quantity.
    Returns a deduplicated list in the same format as the input.
    """
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
    else:
        return [[w, h, q] for (w, h), q in totals.items()]

def get_type_keys(rectangle_list):
    """
    Flatten a rectangle list into unique (width, height) type keys, merging
    duplicate types by summing their quantities.
    rectangle_list: list of [w, h, q] or dicts with width/height/quantity.
    Returns (type_keys, quantities) where type_keys is a list of (w, h) tuples
    and quantities is a parallel list of total counts for each type.
    """
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
    """
    Generate all valid strip configurations for the given rectangle types and bin width.
    type_keys: list of (width, height) tuples, one per unique rectangle type.
    Returns a sorted list of configurations, where each configuration is a list
    of counts [c_0, c_1, ...] indicating how many of each type fit in one strip.
    """
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

