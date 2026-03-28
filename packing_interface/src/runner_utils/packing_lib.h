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

#ifndef PACKING_OBSTACLE_DEFINED
#define PACKING_OBSTACLE_DEFINED
struct Obstacle {
    double x1, x2, y1, y2;
};
#endif


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


inline Obstacle placement_to_obstacle(const std::tuple<double, double, int, int>& placement) {
    return {
        std::get<0>(placement),
        std::get<0>(placement) + std::get<2>(placement),
        std::get<1>(placement),
        std::get<1>(placement) + std::get<3>(placement),
    };
}

inline void append_placement_as_obstacle(
    std::vector<Obstacle>& blockers,
    const std::tuple<double, double, int, int>& placement
) {
    blockers.push_back(placement_to_obstacle(placement));
}

inline bool obstacles_intersect(const Obstacle& a, const Obstacle& b) {
    return a.x1 < b.x2 && a.x2 > b.x1 && a.y1 < b.y2 && a.y2 > b.y1;
}

inline bool can_place_in_region(
    double x,
    double y,
    int width,
    int height,
    int bin_width,
    int bin_height,
    const std::vector<Obstacle>& blockers
) {
    if (x < 0.0 || y < 0.0) {
        return false;
    }
    if (x + width > bin_width || y + height > bin_height) {
        return false;
    }

    Obstacle candidate{x, x + width, y, y + height};
    for (const auto& blocker : blockers) {
        if (obstacles_intersect(candidate, blocker)) {
            return false;
        }
    }
    return true;
}

inline std::vector<double> candidate_x_positions_for_band(
    const std::vector<Obstacle>& blockers,
    double y,
    int band_height
) {
    std::vector<double> positions = {0.0};
    double top = y + band_height;
    for (const auto& blocker : blockers) {
        if (y < blocker.y2 && top > blocker.y1) {
            positions.push_back(std::max(0.0, blocker.x1));
            positions.push_back(std::max(0.0, blocker.x2));
        }
    }
    std::sort(positions.begin(), positions.end());
    positions.erase(std::unique(positions.begin(), positions.end()), positions.end());
    return positions;
}

inline std::optional<double> leftmost_feasible_x_for_band(
    const std::vector<Obstacle>& blockers,
    int band_y,
    int band_height,
    int width,
    int bin_width,
    int bin_height
) {
    for (double candidate_x : candidate_x_positions_for_band(blockers, band_y, band_height)) {
        int x = static_cast<int>(candidate_x);
        if (can_place_in_region(x, band_y, width, band_height, bin_width, bin_height, blockers)) {
            return static_cast<double>(x);
        }
    }
    return std::nullopt;
}

inline std::optional<int> next_level_y(
    const std::vector<Obstacle>& blockers,
    int start_y,
    int level_height,
    int bin_width,
    int bin_height
) {
    std::vector<double> candidates = {static_cast<double>(std::max(0, start_y))};
    for (const auto& blocker : blockers) {
        if (blocker.y2 >= start_y) {
            candidates.push_back(blocker.y2);
        }
    }
    std::sort(candidates.begin(), candidates.end());
    candidates.erase(std::unique(candidates.begin(), candidates.end()), candidates.end());

    for (double candidate_y : candidates) {
        int y = static_cast<int>(candidate_y);
        if (y + level_height > bin_height) {
            continue;
        }
        if (leftmost_feasible_x_for_band(blockers, y, level_height, 1, bin_width, bin_height).has_value()) {
            return y;
        }
    }
    return std::nullopt;
}

inline std::optional<std::pair<double, double>> find_bottom_left_position_with_obstacles(
    const std::vector<Obstacle>& blockers,
    int bin_width,
    int bin_height,
    int width,
    int height
) {
    std::vector<double> candidate_ys = {0.0};
    for (const auto& blocker : blockers) {
        candidate_ys.push_back(blocker.y2);
    }
    std::sort(candidate_ys.begin(), candidate_ys.end());
    candidate_ys.erase(std::unique(candidate_ys.begin(), candidate_ys.end()), candidate_ys.end());

    std::optional<std::pair<double, double>> best_position;
    for (double candidate_y : candidate_ys) {
        int y = static_cast<int>(candidate_y);
        if (y + height > bin_height) {
            continue;
        }
        auto x = leftmost_feasible_x_for_band(blockers, y, height, width, bin_width, bin_height);
        if (!x.has_value()) {
            continue;
        }
        std::pair<double, double> position = {static_cast<double>(y), *x};
        if (!best_position.has_value() || position < *best_position) {
            best_position = position;
        }
    }

    if (!best_position.has_value()) {
        return std::nullopt;
    }
    return std::make_pair(best_position->second, best_position->first);
}


}
