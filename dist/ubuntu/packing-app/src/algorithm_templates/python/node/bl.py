import packing_lib


class Repacking:
    def solve(self, bin_height, bin_width, rectangles, non_empty_space):
        items = packing_lib.sort_by_width(packing_lib.expand_items(rectangles))
        blockers = packing_lib.normalize_obstacles(non_empty_space)
        placements = []

        for item in items:
            width = item["width"]
            height = item["height"]
            position = packing_lib.find_bottom_left_position_with_obstacles(
                blockers, bin_width, bin_height, width, height
            )
            if position is None:
                raise ValueError(f"Unable to place rectangle {width}x{height} with BL node packing")

            x, y = position
            placement = (x, y, width, height)
            placements.append(placement)
            packing_lib.append_placement_as_blocker(blockers, placement)

        return packing_lib.output_from_placements(bin_width, placements)
