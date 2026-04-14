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
        (void)binHeight;
        (void)binWidth;
        (void)rectangles;
        (void)obstacles;
        std::vector<std::tuple<double, double, int, int>> placements;
        return placements;
    }
};
