import json
import math
from scipy.optimize import linprog

with open("algorithm_input.json") as f:
    d = json.load(f)

bin_width = d["width_of_bin"]
rectangle_types = d["rectangle_list"]

type_keys = []
seen = set()
for rect in rectangle_types:
    key = (rect["width"], rect["height"])
    if key not in seen:
        seen.add(key)
        type_keys.append(key)

type_index = {key: i for i, key in enumerate(type_keys)}
num_types = len(type_keys)

configurations = []

def get_config(type_idx, counts, remaining_width):
    if type_idx == num_types:
        if not any(c > 0 for c in counts):
            return

        for i in range(num_types):
            w_i = type_keys[i][0]
            if w_i <= remaining_width + 1e-9:
                return

        configurations.append(counts.copy())
        return

    w_i = type_keys[type_idx][0]
    max_count = int((remaining_width + 1e-9) // w_i)

    for c in range(max_count, -1, -1):
        counts[type_idx] = c
        get_config(type_idx + 1, counts, remaining_width - c * w_i)

    counts[type_idx] = 0

get_config(0, [0] * num_types, bin_width)

unique_configurations = sorted(configurations)

m = len(unique_configurations)

c = [1.0] * m

A = [[0] * m for _ in range(num_types)]
for j, config in enumerate(unique_configurations):
    for i in range(num_types):
        A[i][j] = config[i]

b = []
for key in type_keys:
    total_quantity = 0
    height = key[1]
    for rect in rectangle_types:
        if (rect["width"], rect["height"]) == key:
            total_quantity += rect["quantity"]
    b.append(total_quantity * height)

A_ub = [[-value for value in row] for row in A]
b_ub = [-value for value in b]
bounds = [(0, None)] * m

res = linprog(c=c, A_ub=A_ub, b_ub=b_ub, bounds=bounds, method="highs")

if not res.success:
    raise RuntimeError(f"LP solve failed: {res.message}")

x_values = res.x.tolist()

eps = 1e-9
used_configurations = []

for j, xj in enumerate(x_values):
    if xj <= eps:
        continue

    config = unique_configurations[j]

    blocks = []
    for i, count in enumerate(config):
        if count == 0:
            continue

        w_i, h_i = type_keys[i]
        stacked = xj / h_i
        f_i = stacked - math.floor(stacked + eps)

        if abs(f_i) < eps or abs(f_i - 1.0) < eps:
            f_i = 0.0

        blocks.append({
            "type_index": i,
            "count_side_by_side": count,
            "width": w_i,
            "rect_height": h_i,
            "stacked_count": stacked,
            "fraction_top": f_i
        })

    blocks.sort(key=lambda block: block["fraction_top"])

    used_configurations.append({
        "config_index": j,
        "fractional_height": xj,
        "used_binary": 1,
        "original_counts": config,
        "sorted_blocks": blocks
    })


def build_blocks_from_counts(counts, fractions_by_type):
    blocks = []
    for i, count in enumerate(counts):
        if count <= 0:
            continue
        w_i, h_i = type_keys[i]
        blocks.append({
            "type_index": i,
            "count_side_by_side": count,
            "width_per_rect": w_i,
            "total_width": count * w_i,
            "rect_height": h_i,
            "fraction_top": fractions_by_type[i],
        })
    blocks.sort(key=lambda b: b["fraction_top"])
    return blocks


def build_block_ranges(blocks):
    ranges = []
    x = 0.0
    for b in blocks:
        ranges.append({
            "x_left": x,
            "x_right": x + b["total_width"],
            "type_index": b["type_index"],
            "fraction_top": b["fraction_top"],
            "rect_height": b["rect_height"],
            "count_side_by_side": b["count_side_by_side"],
        })
        x += b["total_width"]
    return ranges


def locate_block_at_x(ranges, x_mid, eps=1e-9):
    for r in ranges:
        if r["x_left"] - eps <= x_mid <= r["x_right"] + eps:
            return r
    return None


def fraction_map_from_used_config(used_cfg):
    frac_map = {i: 0.0 for i in range(num_types)}
    for block in used_cfg["sorted_blocks"]:
        frac_map[block["type_index"]] = block["fraction_top"]
    return frac_map


def counts_map_from_used_config(used_cfg):
    return list(used_cfg["original_counts"])


def reorder_three_configs_for_case_logic(cfgs):
    perms = [
        (0, 1, 2), (0, 2, 1),
        (1, 0, 2), (1, 2, 0),
        (2, 0, 1), (2, 1, 0),
    ]

    def first_non_case1_score(order):
        top, mid, bot = [cfgs[i] for i in order]
        t_ranges = build_block_ranges(top["uncommon_blocks"])
        m_ranges = build_block_ranges(mid["uncommon_blocks"])
        b_ranges = build_block_ranges(bot["uncommon_blocks"])

        boundaries = {0.0}
        for rs in (t_ranges, m_ranges, b_ranges):
            for r in rs:
                boundaries.add(r["x_right"])
        boundaries = sorted(boundaries)

        for a, b in zip(boundaries[:-1], boundaries[1:]):
            if b - a <= 1e-9:
                continue
            xm = (a + b) / 2.0
            rt = locate_block_at_x(t_ranges, xm)
            rm = locate_block_at_x(m_ranges, xm)
            rb = locate_block_at_x(b_ranges, xm)
            if rt is None or rm is None or rb is None:
                continue
            ft = rt["fraction_top"]
            fm = rm["fraction_top"]
            fb = rb["fraction_top"]
            if not (ft <= 1/3 + 1e-9 and fm <= 1/3 + 1e-9 and fb <= 1/3 + 1e-9):
                return 1 if fb > 1/3 + 1e-9 else 0
        return 1

    best = max(perms, key=first_non_case1_score)
    return [cfgs[i] for i in best]


def build_chapter4_style_placements(chapter4_result):
    placements = []

    CTop = chapter4_result["CTop"]
    CMid = chapter4_result["CMid"]
    CBot = chapter4_result["CBot"]

    x_top = CTop["fractional_height"]
    x_mid = CMid["fractional_height"]
    x_bot = CBot["fractional_height"]

    common_counts = chapter4_result["common_counts"]
    common_width = chapter4_result["common_width"]
    sections = chapter4_result["sections"]

    max_rect_height = max(h for _, h in type_keys)

    def floor_count(x, h):
        return math.floor((x + eps) / h)

    def ceil_count(x, h):
        return math.ceil((x - eps) / h)

    common_total_height = 0.0
    common_col_rects_by_type = {}

    for i, c in enumerate(common_counts):
        if c <= 0:
            continue
        _, h_i = type_keys[i]
        total_common_rects = ceil_count(x_top + x_mid + x_bot, h_i)
        common_col_rects_by_type[i] = total_common_rects
        common_total_height = max(common_total_height, total_common_rects * h_i)

    x_cursor = 0.0
    for i, c in enumerate(common_counts):
        if c <= 0:
            continue
        w_i, h_i = type_keys[i]
        num_rects_in_column = common_col_rects_by_type[i]

        for _ in range(c):
            y_cursor = 0.0
            for _ in range(num_rects_in_column):
                placements.append({
                    "x": x_cursor,
                    "y": y_cursor,
                    "width": w_i,
                    "height": h_i
                })
                y_cursor += h_i
            x_cursor += w_i


    section_layouts = []
    ca_width_by_type = {i: 0.0 for i in range(num_types)}
    bot_band_height = 0.0
    mid_band_height = 0.0
    top_band_height = 0.0
    need_ca = False

    for s in sections:
        width = s["width"]
        case = s["case"]

        top_i = s["top"]["type_index"]
        mid_i = s["mid"]["type_index"]
        bot_i = s["bot"]["type_index"]

        ft = s["top"]["fraction_top"]
        fm = s["mid"]["fraction_top"]
        fb = s["bot"]["fraction_top"]

        w_top, h_top = type_keys[top_i]
        w_mid, h_mid = type_keys[mid_i]
        w_bot, h_bot = type_keys[bot_i]

        if case == "Case1":
            top_n = floor_count(x_top, h_top)
            mid_n = floor_count(x_mid, h_mid)
            bot_n = floor_count(x_bot, h_bot)

            if ft > eps:
                ca_width_by_type[top_i] += width * ft
                need_ca = True
            if fm > eps:
                ca_width_by_type[mid_i] += width * fm
                need_ca = True
            if fb > eps:
                ca_width_by_type[bot_i] += width * fb
                need_ca = True

        elif case == "Case2":
            top_n = floor_count(x_top, h_top)
            mid_n = floor_count(x_mid, h_mid)
            bot_n = ceil_count(x_bot, h_bot)

            if ft > eps:
                ca_width_by_type[top_i] += width * ft
                need_ca = True
            if fm > eps:
                ca_width_by_type[mid_i] += width * fm
                need_ca = True

        else:  # Case3
            top_n = ceil_count(x_top, h_top)
            mid_n = ceil_count(x_mid, h_mid)
            bot_n = ceil_count(x_bot, h_bot)

        bot_height = bot_n * h_bot
        mid_height = mid_n * h_mid
        top_height = top_n * h_top

        bot_band_height = max(bot_band_height, bot_height)
        mid_band_height = max(mid_band_height, mid_height)
        top_band_height = max(top_band_height, top_height)

        section_layouts.append({
            "x_left": common_width + s["x_left"],
            "width": width,
            "case": case,
            "top_type": top_i,
            "mid_type": mid_i,
            "bot_type": bot_i,
            "top_count": top_n,
            "mid_count": mid_n,
            "bot_count": bot_n,
        })

    ca_band_height = max_rect_height if need_ca else 0.0

    uncommon_bottom_y = 0.0
    uncommon_mid_y = uncommon_bottom_y + bot_band_height
    uncommon_ca_y = uncommon_mid_y + mid_band_height
    uncommon_top_y = uncommon_ca_y + ca_band_height

    uncommon_total_height = bot_band_height + mid_band_height + ca_band_height + top_band_height

    for layout in section_layouts:
        x_left = layout["x_left"]
        width = layout["width"]

        bot_i = layout["bot_type"]
        mid_i = layout["mid_type"]
        top_i = layout["top_type"]

        _, h_bot = type_keys[bot_i]
        _, h_mid = type_keys[mid_i]
        _, h_top = type_keys[top_i]

        y = uncommon_bottom_y
        for _ in range(layout["bot_count"]):
            placements.append({
                "x": x_left,
                "y": y,
                "width": width,
                "height": h_bot
            })
            y += h_bot

        y = uncommon_mid_y
        for _ in range(layout["mid_count"]):
            placements.append({
                "x": x_left,
                "y": y,
                "width": width,
                "height": h_mid
            })
            y += h_mid

        y = uncommon_top_y
        for _ in range(layout["top_count"]):
            placements.append({
                "x": x_left,
                "y": y,
                "width": width,
                "height": h_top
            })
            y += h_top

    if need_ca:
        x_ca = common_width
        for i in range(num_types):
            reshaped_width = ca_width_by_type[i]
            if reshaped_width <= eps:
                continue
            _, h_i = type_keys[i]
            placements.append({
                "x": x_ca,
                "y": uncommon_ca_y,
                "width": reshaped_width,
                "height": h_i
            })
            x_ca += reshaped_width

    total_height = max(common_total_height, uncommon_total_height)

    return placements, total_height


def chapter4_three_config_rounding(used_configurations):
    if len(used_configurations) != 3:
        raise ValueError("chapter4_three_config_rounding requires exactly 3 used configurations.")

    count_lists = [counts_map_from_used_config(cfg) for cfg in used_configurations]
    common_counts = [min(count_lists[0][i], count_lists[1][i], count_lists[2][i]) for i in range(num_types)]

    common_width = 0.0
    for i, c in enumerate(common_counts):
        common_width += c * type_keys[i][0]

    enriched = []
    for cfg in used_configurations:
        frac_map = fraction_map_from_used_config(cfg)
        counts = counts_map_from_used_config(cfg)
        uncommon_counts = [counts[i] - common_counts[i] for i in range(num_types)]
        uncommon_blocks = build_blocks_from_counts(uncommon_counts, frac_map)

        enriched.append({
            **cfg,
            "fraction_map": frac_map,
            "common_counts": common_counts,
            "uncommon_counts": uncommon_counts,
            "uncommon_blocks": uncommon_blocks,
        })

    ordered = reorder_three_configs_for_case_logic(enriched)
    CTop, CMid, CBot = ordered

    top_ranges = build_block_ranges(CTop["uncommon_blocks"])
    mid_ranges = build_block_ranges(CMid["uncommon_blocks"])
    bot_ranges = build_block_ranges(CBot["uncommon_blocks"])

    boundaries = {0.0}
    for rs in (top_ranges, mid_ranges, bot_ranges):
        for r in rs:
            boundaries.add(r["x_right"])
    boundaries = sorted(boundaries)

    sections = []
    for a, b in zip(boundaries[:-1], boundaries[1:]):
        if b - a <= 1e-9:
            continue
        xm = (a + b) / 2.0
        rt = locate_block_at_x(top_ranges, xm)
        rm = locate_block_at_x(mid_ranges, xm)
        rb = locate_block_at_x(bot_ranges, xm)

        if rt is None or rm is None or rb is None:
            continue

        ft = rt["fraction_top"]
        fm = rm["fraction_top"]
        fb = rb["fraction_top"]

        if ft <= 1/3 + 1e-9 and fm <= 1/3 + 1e-9 and fb <= 1/3 + 1e-9:
            case = "Case1"
        elif fb > 1/3 + 1e-9 and ft + fm <= 1 + 1e-9:
            case = "Case2"
        else:
            case = "Case3"

        sections.append({
            "x_left": a,
            "x_right": b,
            "width": b - a,
            "top": {"type_index": rt["type_index"], "fraction_top": ft},
            "mid": {"type_index": rm["type_index"], "fraction_top": fm},
            "bot": {"type_index": rb["type_index"], "fraction_top": fb},
            "case": case,
        })

    has_case2 = any(s["case"] == "Case2" for s in sections)
    has_case3 = any(s["case"] == "Case3" for s in sections)
    has_case1 = any(s["case"] == "Case1" for s in sections)

    if has_case2 or has_case3:
        height_increase = 5.0 / 3.0
    elif common_width > 0 or has_case1:
        height_increase = 1.0
    else:
        height_increase = 0.0

    fractional_height = sum(cfg["fractional_height"] for cfg in used_configurations)
    rounded_height_bound = fractional_height + height_increase

    return {
        "mode": "3ConfigurationRounding",
        "common_counts": common_counts,
        "common_width": common_width,
        "ordered_config_indices": [
            CTop["config_index"],
            CMid["config_index"],
            CBot["config_index"],
        ],
        "CTop": CTop,
        "CMid": CMid,
        "CBot": CBot,
        "sections": sections,
        "fractional_height": fractional_height,
        "height_increase_bound": height_increase,
        "rounded_height_bound": rounded_height_bound,
    }

if len(used_configurations) == 3:
    chapter4_result = chapter4_three_config_rounding(used_configurations)
elif len(used_configurations) == 2:
    raise ValueError("Need 2-configuration Chapter 4 routine here.")
elif len(used_configurations) == 1:
    raise ValueError("Need 1-configuration Chapter 4 routine here.")
else:
    raise ValueError(f"Expected 1, 2, or 3 used configurations, got {len(used_configurations)}")

placements, total_height = build_chapter4_style_placements(chapter4_result)

output = {
    "bin_width": bin_width,
    "total_height": total_height,
    "placements": placements
}

print("DONE")
with open("bad.json", "w") as out:
    json.dump(output, out, indent=2)
