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
        for _ in range(item["quantity"]):
            items.append({"width": item["width"], "height": item["height"]})
    return items

def sort_by_height(items, descending=True):
    return sorted(items, key=lambda x: x["height"], reverse=descending)

def sort_by_width(items, descending=True):
    return sorted(items, key=lambda x: x["width"], reverse=descending)

def sort_by_area(items, descending=True):
    return sorted(items, key=lambda x: x["width"] * x["height"], reverse=descending)

def sort_by_quantity(rectangle_list, descending=True):
    return sorted(rectangle_list, key=lambda x: x["quantity"], reverse=descending)

