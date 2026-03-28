import packing_lib
from typing import List, Tuple


class Packing:
    def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:
        items = packing_lib.expand_items(rectangles)
        wide_items = []
        remaining_items = []

        for item in items:
            width = item["width"]
            if width > bin_width:
                raise ValueError(f"Rectangle width {width} exceeds bin width {bin_width}.")
            if width > bin_width / 2:
                wide_items.append(item)
            else:
                remaining_items.append(item)

        placements = []
        current_y = 0.0

        for item in wide_items:
            placements.append((0.0, current_y, item["width"], item["height"]))
            current_y += item["height"]

        remaining_items = packing_lib.sort_by_height(remaining_items)
        half_width = bin_width / 2.0

        left_baseline = current_y
        right_baseline = current_y
        level_buffer = []
        level_used_width = 0.0
        level_height = 0.0

        def flush_full_width_level() -> None:
            nonlocal current_y, left_baseline, right_baseline, level_buffer, level_used_width, level_height
            if not level_buffer:
                return
            x = 0.0
            for width, height in level_buffer:
                placements.append((x, current_y, width, height))
                x += width
            current_y += level_height
            left_baseline = current_y
            right_baseline = current_y
            level_buffer = []
            level_used_width = 0.0
            level_height = 0.0


        idx = 0
        while idx < len(remaining_items):
            item = remaining_items[idx]
            width = item["width"]
            height = item["height"]

            if level_used_width + width <= bin_width:
                level_buffer.append((width, height))
                level_used_width += width
                level_height = max(level_height, height)
                idx += 1
                continue

            break

        flush_full_width_level()

        while idx < len(remaining_items):
            item = remaining_items[idx]
            width = item["width"]
            height = item["height"]

            place_left = left_baseline <= right_baseline
            baseline = left_baseline if place_left else right_baseline
            x_start = 0.0 if place_left else half_width
            used_width = 0.0
            level_items = []
            level_height = 0.0

            while idx < len(remaining_items):
                item = remaining_items[idx]
                width = item["width"]
                height = item["height"]
                if width > half_width:
                    break
                if used_width + width > half_width:
                    break
                level_items.append((width, height))
                used_width += width
                level_height = max(level_height, height)
                idx += 1

            if not level_items:
                fallback_y = max(left_baseline, right_baseline)
                fallback_placements = packing_lib.place_items_ffdh(
                    remaining_items[idx:],
                    bin_width,
                    start_y=fallback_y,
                )
                placements.extend(fallback_placements)
                break

            x = x_start
            for width, height in level_items:
                placements.append((x, baseline, width, height))
                x += width

            new_baseline = baseline + level_height
            if place_left:
                left_baseline = new_baseline
            else:
                right_baseline = new_baseline

        return packing_lib.output_from_placements(bin_width, placements)
