import packing_lib


class Repacking:
    def solve(self, bin_height, bin_width, rectangles, non_empty_space):
        items = packing_lib.sort_by_height(packing_lib.expand_items(rectangles))
        placements = []
        blockers = [
            {
                "x_1": float(obstacle["x_1"]),
                "x_2": float(obstacle["x_2"]),
                "y_1": float(obstacle["y_1"]),
                "y_2": float(obstacle["y_2"]),
            }
            for obstacle in non_empty_space
        ]

        def intersects(rect, blocker):
            return (
                rect["x_1"] < blocker["x_2"]
                and rect["x_2"] > blocker["x_1"]
                and rect["y_1"] < blocker["y_2"]
                and rect["y_2"] > blocker["y_1"]
            )

        def can_place(x, y, width, height):
            if x < 0 or y < 0:
                return False
            if x + width > bin_width or y + height > bin_height:
                return False

            candidate = {
                "x_1": float(x),
                "x_2": float(x + width),
                "y_1": float(y),
                "y_2": float(y + height),
            }

            for blocker in blockers:
                if intersects(candidate, blocker):
                    return False

            return True

        def candidate_x_positions(y, height):
            blocked_segments = []
            for blocker in blockers:
                if y < blocker["y_2"] and y + height > blocker["y_1"]:
                    blocked_segments.append((blocker["x_1"], blocker["x_2"]))

            blocked_segments.sort()
            positions = {0.0}
            for start, end in blocked_segments:
                positions.add(max(0.0, start))
                positions.add(max(0.0, end))

            return sorted(positions)

        for item in items:
            width = item["width"]
            height = item["height"]
            placed = False

            max_y = max(0, int(bin_height - height))
            for y in range(max_y + 1):
                for x in candidate_x_positions(y, height):
                    x = int(x)
                    if not can_place(x, y, width, height):
                        continue

                    placements.append((x, y, width, height))
                    blockers.append(
                        {
                            "x_1": float(x),
                            "x_2": float(x + width),
                            "y_1": float(y),
                            "y_2": float(y + height),
                        }
                    )
                    placed = True
                    break

                if placed:
                    break

            if not placed:
                raise ValueError(
                    f"Unable to place rectangle {width}x{height} without intersecting obstacles"
                )

        return packing_lib.output_from_placements(bin_width, placements)
