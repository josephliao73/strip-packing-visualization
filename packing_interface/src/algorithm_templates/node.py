import packing_lib
import json
from typing import List, Tuple

class Repacking:
    def solve(self, bin_height, bin_width, rectangles, non_empty_space):
        items = packing_lib.sort_by_height(packing_lib.expand_items(rectangles))

        levels = []
        placements = []
        current_y = 0

        def intersects_obstacle(x, y, width, height):
            for obstacle in non_empty_space:
                if (
                    x < obstacle["x_2"]
                    and x + width > obstacle["x_1"]
                    and y < obstacle["y_2"]
                    and y + height > obstacle["y_1"]
                ):
                    return True
            return False

        for item in items:
            width = item["width"]
            height = item["height"]
            placed = False

            for level in levels:
                if level["used_width"] + width > bin_width:
                    continue

                x = level["used_width"]
                y = level["y"]
                if intersects_obstacle(x, y, width, height):
                    continue

                placements.append((x, y, width, height))
                level["used_width"] += width
                placed = True
                break

            if placed:
                continue

            y = current_y
            while y < bin_height and intersects_obstacle(0, y, width, height):
                y += 1

            levels.append({"height": height, "used_width": width, "y": y})
            placements.append((0, y, width, height))
            current_y = y + height

        return packing_lib.output_from_placements(bin_width, placements)
