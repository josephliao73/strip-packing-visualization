#pragma once
#include <algorithm>
#include <functional>
#include <map>
#include <optional>
#include <stdexcept>
#include <tuple>
#include <utility>
#include <vector>

namespace packing {


struct Rect {
    int width, height;
};

struct RectType {
    int width, height, quantity;
};


// Expand rectangle types (w, h, q) into individual Rect items.
inline std::vector<Rect> expand_items(
    const std::vector<std::tuple<int, int, int>>& rectangles
) {
    std::vector<Rect> items;
    for (const auto& [w, h, q] : rectangles)
        for (int i = 0; i < q; i++)
            items.push_back({w, h});
    return items;
}

// Sort expanded items by height.
inline std::vector<Rect> sort_by_height(
    std::vector<Rect> items, bool descending = true
) {
    std::sort(items.begin(), items.end(), [descending](const Rect& a, const Rect& b) {
        return descending ? a.height > b.height : a.height < b.height;
    });
    return items;
}

// Sort expanded items by width.
inline std::vector<Rect> sort_by_width(
    std::vector<Rect> items, bool descending = true
) {
    std::sort(items.begin(), items.end(), [descending](const Rect& a, const Rect& b) {
        return descending ? a.width > b.width : a.width < b.width;
    });
    return items;
}

// Sort expanded items by area.
inline std::vector<Rect> sort_by_area(
    std::vector<Rect> items, bool descending = true
) {
    std::sort(items.begin(), items.end(), [descending](const Rect& a, const Rect& b) {
        return descending ? (a.width * a.height) > (b.width * b.height)
                          : (a.width * a.height) < (b.width * b.height);
    });
    return items;
}

inline std::optional<std::pair<double, double>> find_bottom_left_position(
    const std::vector<std::tuple<double, double, int, int>>& placements,
    int bin_width,
    int width,
    int height,
    std::optional<double> max_height = std::nullopt
) {
    std::vector<double> candidate_xs = {0.0};
    for (const auto& [px, py, pw, ph] : placements) {
        (void)py;
        (void)ph;
        candidate_xs.push_back(px + pw);
    }

    std::sort(candidate_xs.begin(), candidate_xs.end());
    candidate_xs.erase(std::unique(candidate_xs.begin(), candidate_xs.end()), candidate_xs.end());

    std::optional<std::pair<double, double>> best_position;
    for (double candidate_x : candidate_xs) {
        if (candidate_x + width > bin_width) {
            continue;
        }

        double candidate_y = 0.0;
        for (const auto& [px, py, pw, ph] : placements) {
            bool overlaps_x = candidate_x < px + pw && candidate_x + width > px;
            if (overlaps_x) {
                candidate_y = std::max(candidate_y, py + ph);
            }
        }

        if (max_height.has_value() && candidate_y + height > *max_height) {
            continue;
        }

        std::pair<double, double> position = {candidate_y, candidate_x};
        if (!best_position.has_value() || position < *best_position) {
            best_position = position;
        }
    }

    if (!best_position.has_value()) {
        return std::nullopt;
    }

    return std::make_pair(best_position->second, best_position->first);
}


// Convert raw tuples to RectType structs.
inline std::vector<RectType> to_rect_types(
    const std::vector<std::tuple<int, int, int>>& rectangles
) {
    std::vector<RectType> types;
    for (const auto& [w, h, q] : rectangles)
        types.push_back({w, h, q});
    return types;
}

// Sort rectangle types by quantity.
inline std::vector<RectType> sort_by_quantity(
    std::vector<RectType> types, bool descending = true
) {
    std::sort(types.begin(), types.end(), [descending](const RectType& a, const RectType& b) {
        return descending ? a.quantity > b.quantity : a.quantity < b.quantity;
    });
    return types;
}

// Merge duplicate (w, h) rectangle types by summing their quantities.
inline std::vector<RectType> dedup_rectangles(
    const std::vector<std::tuple<int, int, int>>& rectangles
) {
    std::map<std::pair<int, int>, int> totals;
    for (const auto& [w, h, q] : rectangles)
        totals[{w, h}] += q;
    std::vector<RectType> result;
    for (const auto& [key, q] : totals)
        result.push_back({key.first, key.second, q});
    return result;
}


struct TypeKeys {
    std::vector<std::pair<int, int>> keys;  // (width, height) pairs
    std::vector<int> quantities;
};

// Extract unique (w, h) type keys and their total quantities from rectangle list.
inline TypeKeys get_type_keys(
    const std::vector<std::tuple<int, int, int>>& rectangles
) {
    std::map<std::pair<int, int>, int> totals;
    for (const auto& [w, h, q] : rectangles)
        totals[{w, h}] += q;
    TypeKeys result;
    for (const auto& [key, q] : totals) {
        result.keys.push_back(key);
        result.quantities.push_back(q);
    }
    return result;
}

// Generate all valid strip configurations for the given type keys and bin width.
inline std::vector<std::vector<int>> get_configurations(
    const std::vector<std::pair<int, int>>& type_keys,
    int bin_width
) {
    std::vector<std::vector<int>> configurations;
    std::vector<int> counts(type_keys.size(), 0);

    std::function<void(int, int)> recurse = [&](int type_idx, int remaining_width) {
        if (type_idx == (int)type_keys.size()) {
            for (int c : counts)
                if (c > 0) { configurations.push_back(counts); return; }
            return;
        }
        int w_i = type_keys[type_idx].first;
        int max_count = remaining_width / w_i;
        for (int c = 0; c <= max_count; c++) {
            counts[type_idx] = c;
            recurse(type_idx + 1, remaining_width - c * w_i);
        }
        counts[type_idx] = 0;
    };

    recurse(0, bin_width);
    std::sort(configurations.begin(), configurations.end());
    return configurations;
}

}
