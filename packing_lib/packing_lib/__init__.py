import json

def make_output(bin_width, total_height, placements):
    return json.dumps({
        "bin_width": bin_width,
        "total_height": total_height, 
        "placements": [{"x": x, "y": y, "width": w, "height": h} for x, y, w, h in placements]
    })
