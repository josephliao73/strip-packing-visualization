// C++ Runner Template for Packing Solutions
// This file wraps user code and handles I/O

#include <iostream>
#include <vector>
#include <string>
#include <sstream>
#include <iomanip>

// User's code will be inserted here
// USER_CODE_PLACEHOLDER

// Simple JSON parsing for rectangles input
std::vector<std::tuple<int, int, int>> parseRectangles(const std::string& json) {
    std::vector<std::tuple<int, int, int>> result;
    std::string s = json;
    size_t pos = 0;

    while ((pos = s.find('[', pos)) != std::string::npos) {
        if (pos > 0 && s[pos-1] != '[' && s[pos-1] != ',') {
            pos++;
            continue;
        }
        size_t end = s.find(']', pos);
        if (end == std::string::npos) break;

        std::string inner = s.substr(pos + 1, end - pos - 1);
        if (inner.find('[') != std::string::npos) {
            pos++;
            continue;
        }

        int w, h, q;
        char comma;
        std::istringstream iss(inner);
        if (iss >> w >> comma >> h >> comma >> q) {
            result.push_back({w, h, q});
        }
        pos = end + 1;
    }
    return result;
}

int main(int argc, char* argv[]) {
    if (argc != 3) {
        std::cerr << "Usage: " << argv[0] << " <bin_width> <rectangles_json>" << std::endl;
        return 1;
    }

    int binWidth = std::stoi(argv[1]);
    std::string rectanglesJson = argv[2];

    auto rectangles = parseRectangles(rectanglesJson);

    Packing packing;
    auto placements = packing.solve(binWidth, rectangles);

    double totalHeight = 0.0;
    for (const auto& p : placements) {
        double top = std::get<1>(p) + std::get<3>(p);
        if (top > totalHeight) totalHeight = top;
    }

    // Output JSON
    std::cout << std::fixed << std::setprecision(6);
    std::cout << "{\"bin_width\":" << binWidth
              << ",\"total_height\":" << totalHeight
              << ",\"placements\":[";

    for (size_t i = 0; i < placements.size(); i++) {
        if (i > 0) std::cout << ",";
        std::cout << "{\"x\":" << std::get<0>(placements[i])
                  << ",\"y\":" << std::get<1>(placements[i])
                  << ",\"width\":" << std::get<2>(placements[i])
                  << ",\"height\":" << std::get<3>(placements[i]) << "}";
    }

    std::cout << "]}" << std::endl;

    return 0;
}
