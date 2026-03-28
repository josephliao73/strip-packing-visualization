import packing_lib


class Repacking:
    def solve(self, bin_height, bin_width, rectangles, non_empty_space):
        items = packing_lib.sort_by_height(packing_lib.expand_items(rectangles))
        blockers = packing_lib.normalize_obstacles(non_empty_space)
        placements = []
        current_y = None
        current_height = 0

        for item in items:
            width = item["width"]
            height = item["height"]

            if current_y is None:
                current_height = height
                current_y = packing_lib.next_level_y(blockers, 0, current_height, bin_width, bin_height)
                if current_y is None:
                    raise ValueError(f"Unable to open initial NFDH level for rectangle {width}x{height}")

            x = packing_lib.leftmost_feasible_x_for_band(
                blockers, current_y, current_height, width, bin_width, bin_height
            )
            if x is None:
                previous_height = current_height
                next_y = packing_lib.next_level_y(
                    blockers, current_y + previous_height, height, bin_width, bin_height
                )
                if next_y is not None:
                    current_y = next_y
                    current_height = height
                    x = packing_lib.leftmost_feasible_x_for_band(
                        blockers, current_y, current_height, width, bin_width, bin_height
                    )

            if x is None:
                position = packing_lib.find_bottom_left_position_with_obstacles(
                    blockers, bin_width, bin_height, width, height
                )
                if position is None:
                    raise ValueError(f"Unable to place rectangle {width}x{height} with NFDH node packing")
                x, y = position
                placement = (x, y, width, height)
            else:
                placement = (x, current_y, width, height)
            placements.append(placement)
            packing_lib.append_placement_as_blocker(blockers, placement)

        return packing_lib.output_from_placements(bin_width, placements)
