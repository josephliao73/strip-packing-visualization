#include "packing_lib.h"
using namespace packing;

class Repacking {
public:
    std::vector<std::tuple<double, double, int, int>> solve(
        int binHeight,
        int binWidth,
        const std::vector<std::tuple<int, int, int>>& rectangles,
        const std::vector<Obstacle>& obstacles
    ) {
        auto items = sort_by_height(expand_items(rectangles));
        std::vector<Obstacle> blockers = obstacles;
        std::vector<std::tuple<double, double, int, int>> placements;

        struct Level { int y; int height; };
        std::vector<Level> levels;

        for (const auto& item : items) {
            bool placed = false;
            for (const auto& level : levels) {
                auto x = leftmost_feasible_x_for_band(blockers, level.y, level.height, item.width, binWidth, binHeight);
                if (!x.has_value()) {
                    continue;
                }
                auto placement = std::make_tuple(*x, static_cast<double>(level.y), item.width, item.height);
                placements.push_back(placement);
                append_placement_as_obstacle(blockers, placement);
                placed = true;
                break;
            }
            if (placed) {
                continue;
            }

            int startY = 0;
            for (const auto& level : levels) {
                startY = std::max(startY, level.y + level.height);
            }
            std::tuple<double, double, int, int> placement;
            auto levelY = next_level_y(blockers, startY, item.height, binWidth, binHeight);
            if (levelY.has_value()) {
                levels.push_back({*levelY, item.height});
                auto x = leftmost_feasible_x_for_band(blockers, *levelY, item.height, item.width, binWidth, binHeight);
                if (x.has_value()) {
                    placement = std::make_tuple(*x, static_cast<double>(*levelY), item.width, item.height);
                } else {
                    auto position = find_bottom_left_position_with_obstacles(blockers, binWidth, binHeight, item.width, item.height);
                    if (!position.has_value()) {
                        throw std::runtime_error("Unable to place rectangle with FFDH node packing");
                    }
                    placement = std::make_tuple(position->first, position->second, item.width, item.height);
                }
            } else {
                auto position = find_bottom_left_position_with_obstacles(blockers, binWidth, binHeight, item.width, item.height);
                if (!position.has_value()) {
                    throw std::runtime_error("Unable to place rectangle with FFDH node packing");
                }
                placement = std::make_tuple(position->first, position->second, item.width, item.height);
            }
            placements.push_back(placement);
            append_placement_as_obstacle(blockers, placement);
        }

        return placements;
    }
};
