use crate::app_paths;
use crate::types::{AlgorithmOutput, CodeLanguage, JsonInput, Rectangle, NonEmptySpace};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum RunResult {
    Success {
        output: AlgorithmOutput,
        raw_json: String,
    },
    SuccessWithWarnings {
        output: AlgorithmOutput,
        raw_json: String,
        warnings: Vec<String>,
    },
    Error {
        errors: Vec<String>,
    },
}

pub trait LanguageRunner {
    fn run(&self, code: &str, bin_width: i32, rectangles: &[Rectangle]) -> RunResult;
    fn repack_run(&self, code: &str, bin_height: f32, bin_width: f32, rectangles: &[Rectangle], non_empty_space: &[NonEmptySpace]) -> RunResult;
}

// ─── Python ──────────────────────────────────────────────────────────────────

pub struct PythonRunner;

impl PythonRunner {
    fn get_runner_path() -> PathBuf {
        app_paths::python_runner_path()
    }

    fn get_python_bin() -> std::ffi::OsString {
        app_paths::python_bin_path()
    }
}

impl LanguageRunner for PythonRunner {

    fn run(&self, code: &str, bin_width: i32, rectangles: &[Rectangle]) -> RunResult {
        let rectangles_json: Vec<Vec<i32>> = rectangles
            .iter()
            .map(|r| vec![r.width, r.height, r.quantity])
            .collect();
        let rectangles_str = match serde_json::to_string(&rectangles_json) {
            Ok(s) => s,
            Err(e) => return RunResult::Error {
                errors: vec![format!("Failed to serialize rectangles: {}", e)],
            },
        };

        let solution_path = std::env::temp_dir().join("sol.py");
        if let Err(e) = std::fs::write(&solution_path, code) {
            return RunResult::Error {
                errors: vec![format!("Failed to write solution file: {}", e)],
            };
        }

        let output = std::process::Command::new(Self::get_python_bin())
            .arg(Self::get_runner_path())
            .arg(&solution_path)
            .arg(bin_width.to_string())
            .arg(&rectangles_str)
            .output();

        parse_output(output, "Python")
    }

    fn repack_run(&self, code: &str, bin_height: f32, bin_width: f32, rectangles: &[Rectangle], non_empty_space: &[NonEmptySpace]) -> RunResult {
        let rectangles_str = match serialize_rectangles(rectangles) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let non_empty_space_str = match serde_json::to_string(non_empty_space) {
            Ok(s) => s,
            Err(e) => return RunResult::Error {
                errors: vec![format!("Failed to serialize non-empty space: {}", e)],
            },
        };

        let solution_path = std::env::temp_dir().join("sol.py");
        if let Err(e) = std::fs::write(&solution_path, code) {
            return RunResult::Error {
                errors: vec![format!("Failed to write solution file: {}", e)],
            };
        }

        let output = std::process::Command::new(Self::get_python_bin())
            .arg(Self::get_runner_path())
            .arg(&solution_path)
            .arg((bin_height.max(0.0).round() as i32).to_string())
            .arg((bin_width.max(0.0).round() as i32).to_string())
            .arg(&rectangles_str)
            .arg(&non_empty_space_str)
            .output();

        parse_output(output, "Python")
    }
}

// ─── C++ ─────────────────────────────────────────────────────────────────────

pub struct CppRunner;

impl CppRunner {
    fn get_packing_main() -> &'static str {
        r#"
std::vector<std::tuple<int, int, int>> parseRectangles(const std::string& json) {
    std::vector<std::tuple<int, int, int>> result;
    std::string s = json;
    size_t pos = 0;
    while ((pos = s.find('[', pos)) != std::string::npos) {
        if (pos > 0 && s[pos-1] != '[' && s[pos-1] != ',') { pos++; continue; }
        size_t end = s.find(']', pos);
        if (end == std::string::npos) break;
        std::string inner = s.substr(pos + 1, end - pos - 1);
        if (inner.find('[') != std::string::npos) { pos++; continue; }
        int w, h, q; char comma;
        std::istringstream iss(inner);
        if (iss >> w >> comma >> h >> comma >> q)
            result.push_back({w, h, q});
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
    auto rectangles = parseRectangles(argv[2]);

    Packing packing;
    auto placements = packing.solve(binWidth, rectangles);

    double totalHeight = 0.0;
    for (const auto& p : placements) {
        double top = std::get<1>(p) + std::get<3>(p);
        if (top > totalHeight) totalHeight = top;
    }

    std::cout << std::fixed << std::setprecision(6);
    std::cout << "{\"bin_width\":" << binWidth
              << ",\"total_height\":" << totalHeight
              << ",\"placements\":[";
    for (size_t i = 0; i < placements.size(); i++) {
        if (i > 0) std::cout << ",";
        std::cout << "{\"x\":"      << std::get<0>(placements[i])
                  << ",\"y\":"      << std::get<1>(placements[i])
                  << ",\"width\":"  << std::get<2>(placements[i])
                  << ",\"height\":" << std::get<3>(placements[i]) << "}";
    }
    std::cout << "]}" << std::endl;
    return 0;
}
"#
    }

    fn get_repack_main() -> &'static str {
        r#"
std::vector<std::tuple<int, int, int>> parseRectangles(const std::string& json) {
    std::vector<std::tuple<int, int, int>> result;
    std::string s = json;
    size_t pos = 0;
    while ((pos = s.find('[', pos)) != std::string::npos) {
        if (pos > 0 && s[pos-1] != '[' && s[pos-1] != ',') { pos++; continue; }
        size_t end = s.find(']', pos);
        if (end == std::string::npos) break;
        std::string inner = s.substr(pos + 1, end - pos - 1);
        if (inner.find('[') != std::string::npos) { pos++; continue; }
        int w, h, q; char comma;
        std::istringstream iss(inner);
        if (iss >> w >> comma >> h >> comma >> q)
            result.push_back({w, h, q});
        pos = end + 1;
    }
    return result;
}

double extractField(const std::string& json, const std::string& key) {
    std::string search = "\"" + key + "\":";
    size_t pos = json.find(search);
    if (pos == std::string::npos) return 0.0;
    pos += search.size();
    return std::stod(json.substr(pos));
}

std::vector<Obstacle> parseObstacles(const std::string& json) {
    std::vector<Obstacle> result;
    size_t pos = 0;
    while ((pos = json.find('{', pos)) != std::string::npos) {
        size_t end = json.find('}', pos);
        if (end == std::string::npos) break;
        std::string obj = json.substr(pos, end - pos + 1);
        Obstacle o;
        o.x1 = extractField(obj, "x_1");
        o.x2 = extractField(obj, "x_2");
        o.y1 = extractField(obj, "y_1");
        o.y2 = extractField(obj, "y_2");
        result.push_back(o);
        pos = end + 1;
    }
    return result;
}

int main(int argc, char* argv[]) {
    if (argc != 5) {
        std::cerr << "Usage: " << argv[0]
                  << " <bin_height> <bin_width> <rectangles_json> <non_empty_space_json>"
                  << std::endl;
        return 1;
    }
    int binHeight = std::stoi(argv[1]);
    int binWidth  = std::stoi(argv[2]);
    auto rectangles = parseRectangles(argv[3]);
    auto obstacles  = parseObstacles(argv[4]);

    Repacking repacking;
    auto placements = repacking.solve(binHeight, binWidth, rectangles, obstacles);

    double totalHeight = 0.0;
    for (const auto& p : placements) {
        double top = std::get<1>(p) + std::get<3>(p);
        if (top > totalHeight) totalHeight = top;
    }

    std::cout << std::fixed << std::setprecision(6);
    std::cout << "{\"bin_width\":" << binWidth
              << ",\"total_height\":" << totalHeight
              << ",\"placements\":[";
    for (size_t i = 0; i < placements.size(); i++) {
        if (i > 0) std::cout << ",";
        std::cout << "{\"x\":"      << std::get<0>(placements[i])
                  << ",\"y\":"      << std::get<1>(placements[i])
                  << ",\"width\":"  << std::get<2>(placements[i])
                  << ",\"height\":" << std::get<3>(placements[i]) << "}";
    }
    std::cout << "]}" << std::endl;
    return 0;
}
"#
    }

    fn cpp_includes() -> &'static str {
        "#include <iostream>\n#include <vector>\n#include <tuple>\n#include <string>\n#include <sstream>\n#include <iomanip>\n"
    }

    fn repack_preamble() -> &'static str {
        "#ifndef PACKING_OBSTACLE_DEFINED\n#define PACKING_OBSTACLE_DEFINED\nnamespace packing { struct Obstacle { double x1, x2, y1, y2; }; }\nusing packing::Obstacle;\n#endif\n"
    }

    fn runner_utils_dir() -> PathBuf {
        app_paths::runner_utils_dir()
    }

    fn compile(source_path: &std::path::Path, binary_path: &std::path::Path) -> Option<RunResult> {
        let result = std::process::Command::new("g++")
            .arg("-std=c++17")
            .arg("-O2")
            .arg("-I").arg(Self::runner_utils_dir())
            .arg("-o").arg(binary_path)
            .arg(source_path)
            .output();

        match result {
            Ok(out) if !out.status.success() => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                Some(RunResult::Error {
                    errors: stderr.lines().map(|s| s.to_string()).collect(),
                })
            }
            Err(e) => Some(RunResult::Error {
                errors: vec![format!("Failed to run g++: {}", e)],
            }),
            _ => None,
        }
    }
}

impl LanguageRunner for CppRunner {

    fn run(&self, code: &str, bin_width: i32, rectangles: &[Rectangle]) -> RunResult {
        let rectangles_str = match serialize_rectangles(rectangles) {
            Ok(s) => s,
            Err(e) => return e,
        };

        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("packing_sol.cpp");
        let binary_path = temp_dir.join("packing_sol");

        let full_code = format!("{}\n{}\n{}", Self::cpp_includes(), code, Self::get_packing_main());
        if let Err(e) = std::fs::write(&source_path, &full_code) {
            return RunResult::Error {
                errors: vec![format!("Failed to write source file: {}", e)],
            };
        }

        if let Some(err) = Self::compile(&source_path, &binary_path) {
            return err;
        }

        let output = std::process::Command::new(&binary_path)
            .arg(bin_width.to_string())
            .arg(&rectangles_str)
            .output();

        parse_output(output, "C++ binary")
    }

    fn repack_run(&self, code: &str, bin_height: f32, bin_width: f32, rectangles: &[Rectangle], non_empty_space: &[NonEmptySpace]) -> RunResult {
        let rectangles_str = match serialize_rectangles(rectangles) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let non_empty_space_str = match serde_json::to_string(non_empty_space) {
            Ok(s) => s,
            Err(e) => return RunResult::Error {
                errors: vec![format!("Failed to serialize non-empty space: {}", e)],
            },
        };

        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("repack_sol.cpp");
        let binary_path = temp_dir.join("repack_sol");

        let full_code = format!("{}\n{}\n{}\n{}", Self::cpp_includes(), Self::repack_preamble(), code, Self::get_repack_main());
        if let Err(e) = std::fs::write(&source_path, &full_code) {
            return RunResult::Error {
                errors: vec![format!("Failed to write source file: {}", e)],
            };
        }

        if let Some(err) = Self::compile(&source_path, &binary_path) {
            return err;
        }

        let output = std::process::Command::new(&binary_path)
            .arg((bin_height.max(0.0).round() as i32).to_string())
            .arg((bin_width.max(0.0).round() as i32).to_string())
            .arg(&rectangles_str)
            .arg(&non_empty_space_str)
            .output();

        parse_output(output, "C++ binary")
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn serialize_rectangles(rectangles: &[Rectangle]) -> Result<String, RunResult> {
    let json: Vec<Vec<i32>> = rectangles
        .iter()
        .map(|r| vec![r.width, r.height, r.quantity])
        .collect();
    serde_json::to_string(&json).map_err(|e| RunResult::Error {
        errors: vec![format!("Failed to serialize rectangles: {}", e)],
    })
}

fn validate_packing_output(output: &AlgorithmOutput) -> Result<(), Vec<String>> {
    let mut warnings = Vec::new();

    for (placement_index, placement) in output.placements.iter().enumerate() {
        let x1 = placement.x.into_inner();
        let y1 = placement.y.into_inner();
        let x2 = x1 + placement.width as f32;
        let y2 = y1 + placement.height as f32;

        if x1 < 0.0 || y1 < 0.0 || x2 > output.bin_width as f32 {
            warnings.push(format!(
                "Placement {} is out of bounds: rect=({}, {}, {}, {}), bin_width={}",
                placement_index,
                x1,
                y1,
                x2,
                y2,
                output.bin_width,
            ));
        }

        for (other_index, other) in output.placements.iter().enumerate().skip(placement_index + 1) {
            let ox1 = other.x.into_inner();
            let oy1 = other.y.into_inner();
            let ox2 = ox1 + other.width as f32;
            let oy2 = oy1 + other.height as f32;

            let intersects = x1 < ox2
                && x2 > ox1
                && y1 < oy2
                && y2 > oy1;

            if intersects {
                warnings.push(format!(
                    "Placement {} intersects placement {}: rect_a=({}, {}, {}, {}), rect_b=({}, {}, {}, {})",
                    placement_index,
                    other_index,
                    x1,
                    y1,
                    x2,
                    y2,
                    ox1,
                    oy1,
                    ox2,
                    oy2,
                ));
            }
        }
    }

    if warnings.is_empty() {
        Ok(())
    } else {
        Err(warnings)
    }
}

fn validate_repack_output(output: &AlgorithmOutput, non_empty_space: &[NonEmptySpace]) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for (placement_index, placement) in output.placements.iter().enumerate() {
        let x1 = placement.x.into_inner();
        let y1 = placement.y.into_inner();
        let x2 = x1 + placement.width as f32;
        let y2 = y1 + placement.height as f32;

        for (obstacle_index, obstacle) in non_empty_space.iter().enumerate() {
            let intersects = x1 < obstacle.x_2
                && x2 > obstacle.x_1
                && y1 < obstacle.y_2
                && y2 > obstacle.y_1;

            if intersects {
                errors.push(format!(
                    "Repack placement {} intersects obstacle {}: rect=({}, {}, {}, {}), obstacle=({}, {}, {}, {})",
                    placement_index,
                    obstacle_index,
                    x1,
                    y1,
                    x2,
                    y2,
                    obstacle.x_1,
                    obstacle.y_1,
                    obstacle.x_2,
                    obstacle.y_2,
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn parse_output(output: std::io::Result<std::process::Output>, runner_name: &str) -> RunResult {
    match output {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                match serde_json::from_str::<AlgorithmOutput>(&stdout) {
                    Ok(algo_output) => RunResult::Success {
                        output: algo_output,
                        raw_json: stdout,
                    },
                    Err(e) => RunResult::Error {
                        errors: vec![format!("Failed to parse output: {}", e)],
                    },
                }
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                RunResult::Error {
                    errors: stderr.lines().map(|s| s.to_string()).collect(),
                }
            }
        }
        Err(e) => RunResult::Error {
            errors: vec![format!("Failed to run {}: {}", runner_name, e)],
        },
    }
}

// ─── Public API ──────────────────────────────────────────────────────────────

pub fn get_runner(language: CodeLanguage) -> Box<dyn LanguageRunner> {
    match language {
        CodeLanguage::Python => Box::new(PythonRunner),
        CodeLanguage::Cpp => Box::new(CppRunner),
        CodeLanguage::Java => Box::new(PythonRunner), // Java not yet implemented
    }
}

pub fn run_code_with_testcase(language: CodeLanguage, code: &str, testcase: &JsonInput) -> RunResult {
    match get_runner(language).run(code, testcase.width_of_bin, &testcase.rectangle_list) {
        RunResult::Success { output, raw_json } => match validate_packing_output(&output) {
            Ok(()) => RunResult::Success { output, raw_json },
            Err(warnings) => RunResult::SuccessWithWarnings { output, raw_json, warnings },
        },
        other => other,
    }
}

pub fn run_repack_code_with_testcase(
    language: CodeLanguage,
    code: &str,
    testcase: &JsonInput,
    bin_height: f32,
    non_empty_space: &[NonEmptySpace],
) -> RunResult {
    match get_runner(language).repack_run(
        code,
        bin_height,
        testcase.width_of_bin as f32,
        &testcase.rectangle_list,
        non_empty_space,
    ) {
        RunResult::Success { output, raw_json } => match validate_repack_output(&output, non_empty_space) {
            Ok(()) => RunResult::Success { output, raw_json },
            Err(warnings) => RunResult::SuccessWithWarnings { output, raw_json, warnings },
        },
        error => error,
    }
}
