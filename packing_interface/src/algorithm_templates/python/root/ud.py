import packing_lib
from typing import List, Tuple


Placement = Tuple[float, float, int, int]


class Packing:
    def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:
        items = packing_lib.expand_items(rectangles)
        items = packing_lib.sort_by_width(items)

        regions = [
            {"placements": [], "height": 0.0}
            for _ in range(5)
        ]

        large_groups = [[], [], [], []]
        small_items = []

        for item in items:
            width = item["width"]
            height = item["height"]

            if width > bin_width:
                raise ValueError(f"Rectangle width {width} exceeds bin width {bin_width}.")

            ratio = width / bin_width if bin_width > 0 else 1.0
            if ratio > 0.5:
                large_groups[0].append(item)
            elif ratio > (1.0 / 3.0):
                large_groups[1].append(item)
            elif ratio > 0.25:
                large_groups[2].append(item)
            elif ratio > 0.2:
                large_groups[3].append(item)
            else:
                small_items.append(item)

        for group_index, group_items in enumerate(large_groups):
            own_region = regions[group_index]
            for item in group_items:
                width = item["width"]
                height = item["height"]
                placed = False

                for prior_region in regions[:group_index]:
                    position = packing_lib.find_bottom_left_position(
                        prior_region["placements"],
                        bin_width,
                        width,
                        height,
                        prior_region["height"],
                    )
                    if position is not None:
                        x, y = position
                        prior_region["placements"].append((x, y, width, height))
                        placed = True
                        break

                if placed:
                    continue

                position = packing_lib.find_bottom_left_position(
                    own_region["placements"],
                    bin_width,
                    width,
                    height,
                )
                if position is None:
                    raise ValueError("Unable to place rectangle with the Up-Down heuristic.")

                x, y = position
                own_region["placements"].append((x, y, width, height))
                own_region["height"] = max(own_region["height"], y + height)

        small_items = packing_lib.sort_by_height(small_items)
        region_five_shelves: List[dict] = []

        for item in small_items:
            width = item["width"]
            height = item["height"]
            placed = False

            for prior_region in regions[:4]:
                position = packing_lib.find_bottom_left_position(
                    prior_region["placements"],
                    bin_width,
                    width,
                    height,
                    prior_region["height"],
                )
                if position is not None:
                    x, y = position
                    prior_region["placements"].append((x, y, width, height))
                    placed = True
                    break

            if placed:
                continue

            for shelf in region_five_shelves:
                if shelf["used_width"] + width <= bin_width and height <= shelf["height"]:
                    x = shelf["used_width"]
                    y = shelf["y"]
                    shelf["used_width"] += width
                    regions[4]["placements"].append((x, y, width, height))
                    placed = True
                    break

            if placed:
                continue

            shelf_y = sum(shelf["height"] for shelf in region_five_shelves)
            region_five_shelves.append({"y": shelf_y, "height": height, "used_width": width})
            regions[4]["placements"].append((0.0, shelf_y, width, height))
            regions[4]["height"] = max(regions[4]["height"], shelf_y + height)

        placements: List[Placement] = []
        y_offset = 0.0
        for region in regions:
            for x, y, width, height in region["placements"]:
                placements.append((x, y + y_offset, width, height))
            y_offset += region["height"]

        return packing_lib.output_from_placements(bin_width, placements)

