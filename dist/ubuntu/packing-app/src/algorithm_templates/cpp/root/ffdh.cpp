#include "packing_lib.h"
using namespace packing;

class Packing {
public:
    std::vector<std::tuple<double, double, int, int>> solve(
        int binWidth,
        const std::vector<std::tuple<int, int, int>>& rectangles
    ) {
        auto items = sort_by_height(expand_items(rectangles));

        struct Level {
            int y;
            int height;
            int usedWidth;
        };

        std::vector<Level> levels;
        std::vector<std::tuple<double, double, int, int>> placements;
        int currentY = 0;

        for (const auto& item : items) {
            bool placed = false;

            for (auto& level : levels) {
                if (level.usedWidth + item.width > binWidth) {
                    continue;
                }

                placements.push_back({
                    static_cast<double>(level.usedWidth),
                    static_cast<double>(level.y),
                    item.width,
                    item.height,
                });
                level.usedWidth += item.width;
                placed = true;
                break;
            }

            if (placed) {
                continue;
            }

            levels.push_back({currentY, item.height, item.width});
            placements.push_back({0.0, static_cast<double>(currentY), item.width, item.height});
            currentY += item.height;
        }

        return placements;
    }
};
