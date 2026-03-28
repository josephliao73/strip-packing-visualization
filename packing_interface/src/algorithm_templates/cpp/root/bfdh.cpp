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
            int best_level = -1;
            int best_remaining_width = 0;

            for (int i = 0; i < static_cast<int>(levels.size()); ++i) {
                const auto& level = levels[i];
                if (level.usedWidth + item.width > binWidth) {
                    continue;
                }

                int remaining_width = binWidth - (level.usedWidth + item.width);
                if (best_level == -1 || remaining_width < best_remaining_width) {
                    best_level = i;
                    best_remaining_width = remaining_width;
                }
            }

            if (best_level != -1) {
                auto& level = levels[best_level];
                placements.push_back({
                    static_cast<double>(level.usedWidth),
                    static_cast<double>(level.y),
                    item.width,
                    item.height,
                });
                level.usedWidth += item.width;
                continue;
            }

            levels.push_back({currentY, item.height, item.width});
            placements.push_back({0.0, static_cast<double>(currentY), item.width, item.height});
            currentY += item.height;
        }

        return placements;
    }
};
