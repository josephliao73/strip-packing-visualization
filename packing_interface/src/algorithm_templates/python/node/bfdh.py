import packing_lib


class Repacking:
    def solve(self, bin_height, bin_width, rectangles, non_empty_space):
        items = packing_lib.sort_by_height(packing_lib.expand_items(rectangles))
        blockers = packing_lib.normalize_obstacles(non_empty_space)
        placements = []
        levels = []

        for item in items:
            width = item["width"]
            height = item["height"]
            best_choice = None

            for level in levels:
                x = packing_lib.leftmost_feasible_x_for_band(
                    blockers, level["y"], level["height"], width, bin_width, bin_height
                )
                if x is None:
                    continue
                residual = bin_width - (x + width)
                choice = (residual, level["y"], x, level)
                if best_choice is None or choice < best_choice:
                    best_choice = choice

            if best_choice is None:
                start_y = max((level["y"] + level["height"] for level in levels), default=0)
                level_y = packing_lib.next_level_y(blockers, start_y, height, bin_width, bin_height)
                if level_y is not None:
                    level = {"y": level_y, "height": height}
                    levels.append(level)
                    x = packing_lib.leftmost_feasible_x_for_band(
                        blockers, level_y, height, width, bin_width, bin_height
                    )
                    if x is not None:
                        best_choice = (bin_width - (x + width), level_y, x, level)

            if best_choice is None:
                position = packing_lib.find_bottom_left_position_with_obstacles(
                    blockers, bin_width, bin_height, width, height
                )
                if position is None:
                    raise ValueError(f"Unable to place rectangle {width}x{height} with BFDH node packing")
                x, y = position
                placement = (x, y, width, height)
            else:
                _, level_y, x, _ = best_choice
                placement = (x, level_y, width, height)
            placements.append(placement)
            packing_lib.append_placement_as_blocker(blockers, placement)

        return packing_lib.output_from_placements(bin_width, placements)
