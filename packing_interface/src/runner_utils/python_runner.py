#!/usr/bin/env python3


"""
Runner for user-submitted Packing solutions.
Executes the user's Packing class and outputs JSON results.

Usage:
    python python_runner.py <solution_file> <bin_width> <rectangles_json>

Example:
    python python_runner.py solution.py 100 '[[10,20,1],[15,25,2]]'
"""

import sys
import json
import importlib.util


def load_solution(solution_path: str):
    """Load the user's solution module and return the Packing class."""
    spec = importlib.util.spec_from_file_location("solution", solution_path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    if not hasattr(module, "Packing"):
        raise AttributeError("Solution must contain a 'Packing' class")

    return module.Packing


def main():
    if len(sys.argv) != 4:
        print("Usage: python python_runner.py <solution_file> <bin_width> <rectangles_json>", file=sys.stderr)
        sys.exit(1)

    solution_path = sys.argv[1]
    bin_width = int(sys.argv[2])
    rectangles_raw = json.loads(sys.argv[3])

    rectangles = [(r[0], r[1], r[2]) for r in rectangles_raw]

    Packing = load_solution(solution_path)
    packing = Packing()

    result = packing.solve(bin_width, rectangles)

    total_height = 0.0
    placements = []

    for placement in result:
        x, y, w, h = placement
        placements.append({
            "x": float(x),
            "y": float(y),
            "width": int(w),
            "height": int(h)
        })
        top = y + h
        if top > total_height:
            total_height = float(top)

    output = {
        "bin_width": bin_width,
        "total_height": total_height,
        "placements": placements
    }

    print(json.dumps(output))


if __name__ == "__main__":
    main()
