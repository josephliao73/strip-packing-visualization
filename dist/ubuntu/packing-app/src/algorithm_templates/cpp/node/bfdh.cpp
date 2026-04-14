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
            std::optional<std::tuple<int, int, double>> bestChoice;

            for (const auto& level : levels) {
                auto x = leftmost_feasible_x_for_band(blockers, level.y, level.height, item.width, binWidth, binHeight);
                if (!x.has_value()) {
                    continue;
                }
                int residual = binWidth - (static_cast<int>(*x) + item.width);
                auto candidate = std::make_tuple(residual, level.y, *x);
                if (!bestChoice.has_value() || candidate < *bestChoice) {
                    bestChoice = candidate;
                }
            }

            if (!bestChoice.has_value()) {
                int startY = 0;
                for (const auto& level : levels) {
                    startY = std::max(startY, level.y + level.height);
                }
                auto levelY = next_level_y(blockers, startY, item.height, binWidth, binHeight);
                if (levelY.has_value()) {
                    levels.push_back({*levelY, item.height});
                    auto x = leftmost_feasible_x_for_band(blockers, *levelY, item.height, item.width, binWidth, binHeight);
                    if (x.has_value()) {
                        bestChoice = std::make_tuple(binWidth - (static_cast<int>(*x) + item.width), *levelY, *x);
                    }
                }
            }

            std::tuple<double, double, int, int> placement;
            if (!bestChoice.has_value()) {
                auto position = find_bottom_left_position_with_obstacles(blockers, binWidth, binHeight, item.width, item.height);
                if (!position.has_value()) {
                    throw std::runtime_error("Unable to place rectangle with BFDH node packing");
                }
                placement = std::make_tuple(position->first, position->second, item.width, item.height);
            } else {
                placement = std::make_tuple(std::get<2>(*bestChoice), static_cast<double>(std::get<1>(*bestChoice)), item.width, item.height);
            }
            placements.push_back(placement);
            append_placement_as_obstacle(blockers, placement);
        }

        return placements;
    }
};
