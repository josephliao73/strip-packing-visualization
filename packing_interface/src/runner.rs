use crate::types::{AlgorithmOutput, CodeLanguage, JsonInput, ParseOutput, Rectangle, NonEmptySpace};
use std::{path::PathBuf};

#[derive(Debug, Clone)]
pub enum RunResult {
    Success {
        output: AlgorithmOutput,
        raw_json: String,
    },
    Error {
        errors: Vec<String>,
    },
}

pub trait LanguageRunner {
    fn file_extension(&self) -> &'static str;
    fn run(&self, code: &str, bin_width: i32, rectangles: &[Rectangle]) -> RunResult;
    fn repack_run(&self, code: &str, bin_height: f32, bin_width: f32, rectangles: &[Rectangle], non_empty_space: &[NonEmptySpace] ) -> RunResult;
}

pub struct PythonRunner;

impl PythonRunner {
    fn get_runner_path() -> PathBuf {
        std::env::current_dir().unwrap().join("./runner_utils/python_runner.py")
    }
}

impl LanguageRunner for PythonRunner {
    fn file_extension(&self) -> &'static str {
        "py"
    }

    fn run(&self, code: &str, bin_width: i32, rectangles: &[Rectangle]) -> RunResult {
        let rectangles_json: Vec<Vec<i32>> = rectangles
            .iter()
            .map(|r| vec![r.width, r.height, r.quantity])
            .collect();
        let rectangles_str = match serde_json::to_string(&rectangles_json) {
            Ok(s) => s,
            Err(e) => {
                return RunResult::Error {
                    errors: vec![format!("Failed to serialize rectangles: {}", e)],
                }
            }
        };

        let temp_dir = std::env::temp_dir();
        println!("TEMP DIR: {:?}", temp_dir);
        let solution_path = temp_dir.join("./sol.py");

        if let Err(e) = std::fs::write(&solution_path, code) {
            return RunResult::Error {
                errors: vec![format!("Failed to write solution file: {}", e)],
            };
        }
        println!("REC LIST");
        dbg!(rectangles);
        let runner_path = Self::get_runner_path();

        let output = std::process::Command::new("python3")
            .arg(&runner_path)
            .arg(&solution_path)
            .arg(bin_width.to_string())
            .arg(&rectangles_str)
            .output();

        // Debug probe removed: this runner requires CLI args.

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    println!("STDOUT {:?}", stdout);
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
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    RunResult::Error {
                        errors: stderr.lines().map(|s| s.to_string()).collect(),
                    }
                }
            }
            Err(e) => RunResult::Error {
                errors: vec![format!("Failed to run Python: {}", e)],
            },
        }
    }
    fn repack_run (&self, code: &str, bin_height: f32, bin_width: f32, rectangles: &[Rectangle], non_empty_space: &[NonEmptySpace])-> RunResult {
        let rectangles_json: Vec<Vec<i32>> = rectangles
            .iter()
            .map(|r| vec![r.width, r.height, r.quantity])
            .collect();
        let rectangles_str = match serde_json::to_string(&rectangles_json) {
            Ok(s) => s,
            Err(e) => {
                return RunResult::Error {
                    errors: vec![format!("Failed to serialize rectangles: {}", e)],
                }
            }
        };

        let non_empty_space_str = match serde_json::to_string(&non_empty_space) {
            Ok(s) => s,
            Err(e) => {
                return RunResult::Error {
                    errors: vec![format!("Failed to serialize non-empty space: {}", e)],
                }
            }
        };

        let temp_dir = std::env::temp_dir();
        println!("TEMP DIR: {:?}", temp_dir);
        let solution_path = temp_dir.join("./sol.py");

        if let Err(e) = std::fs::write(&solution_path, code) {
            return RunResult::Error {
                errors: vec![format!("Failed to write solution file: {}", e)],
            };
        }
        println!("REC LIST");
        dbg!(rectangles);
        let runner_path = Self::get_runner_path();

        let output = std::process::Command::new("python3")
            .arg(&runner_path)
            .arg(&solution_path)
            .arg((bin_height.max(0.0).round() as i32).to_string())
            .arg((bin_width.max(0.0).round() as i32).to_string())
            .arg(&rectangles_str)
            .arg(&non_empty_space_str)
            .output();

        // Debug probe removed: this runner requires CLI args.

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    println!("STDOUT {:?}", stdout);
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
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    RunResult::Error {
                        errors: stderr.lines().map(|s| s.to_string()).collect(),
                    }
                }
            }
            Err(e) => RunResult::Error {
                errors: vec![format!("Failed to run Python: {}", e)],
            },
        }
    }
}

pub struct CppRunner;

impl CppRunner {
    fn get_template() -> &'static str {
        r#"#include <iostream>
#include <vector>
#include <tuple>
#include <string>
#include <sstream>
#include <iomanip>

// User's code inserted below
"#
    }

    fn get_main_code() -> &'static str {
        r#"
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
"#
    }
}

/*
impl LanguageRunner for CppRunner {
    fn file_extension(&self) -> &'static str {
        "cpp"
    }

    fn run(&self, code: &str, bin_width: i32, rectangles: &[Rectangle]) -> RunResult {
        let rectangles_json: Vec<Vec<i32>> = rectangles
            .iter()
            .map(|r| vec![r.width, r.height, r.quantity])
            .collect();
        let rectangles_str = match serde_json::to_string(&rectangles_json) {
            Ok(s) => s,
            Err(e) => {
                return RunResult::Error {
                    errors: vec![format!("Failed to serialize rectangles: {}", e)],
                }
            }
        };

        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("packing_solution.cpp");
        let binary_path = temp_dir.join("packing_solution_cpp");

        let full_code = format!("{}\n{}\n{}", Self::get_template(), code, Self::get_main_code());

        if let Err(e) = std::fs::write(&source_path, &full_code) {
            return RunResult::Error {
                errors: vec![format!("Failed to write source file: {}", e)],
            };
        }

        let compile_output = std::process::Command::new("g++")
            .arg("-std=c++17")
            .arg("-O2")
            .arg("-o")
            .arg(&binary_path)
            .arg(&source_path)
            .output();

        match compile_output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return RunResult::Error {
                        errors: stderr.lines().map(|s| s.to_string()).collect(),
                    };
                }
            }
            Err(e) => {
                return RunResult::Error {
                    errors: vec![format!("Failed to run g++: {}", e)],
                };
            }
        }

        let run_output = std::process::Command::new(&binary_path)
            .arg(bin_width.to_string())
            .arg(&rectangles_str)
            .output();

        match run_output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
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
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    RunResult::Error {
                        errors: stderr.lines().map(|s| s.to_string()).collect(),
                    }
                }
            }
            Err(e) => RunResult::Error {
                errors: vec![format!("Failed to run compiled binary: {}", e)],
            },
        }
    }
}

pub struct JavaRunner;

impl JavaRunner {
    fn get_runner_code() -> &'static str {
        r#"import java.util.*;
import java.util.regex.*;

public class PackingRunner {
    public static void main(String[] args) {
        if (args.length != 2) {
            System.err.println("Usage: java PackingRunner <bin_width> <rectangles_json>");
            System.exit(1);
        }

        int binWidth = Integer.parseInt(args[0]);
        String rectanglesJson = args[1];

        List<int[]> rectangles = parseRectangles(rectanglesJson);

        Packing packing = new Packing();
        List<double[]> placements = packing.solve(binWidth, rectangles);

        double totalHeight = 0.0;
        for (double[] p : placements) {
            double top = p[1] + p[3];
            if (top > totalHeight) totalHeight = top;
        }

        StringBuilder sb = new StringBuilder();
        sb.append("{\"bin_width\":").append(binWidth);
        sb.append(",\"total_height\":").append(totalHeight);
        sb.append(",\"placements\":[");

        for (int i = 0; i < placements.size(); i++) {
            if (i > 0) sb.append(",");
            double[] p = placements.get(i);
            sb.append("{\"x\":").append(p[0]);
            sb.append(",\"y\":").append(p[1]);
            sb.append(",\"width\":").append((int)p[2]);
            sb.append(",\"height\":").append((int)p[3]).append("}");
        }

        sb.append("]}");
        System.out.println(sb.toString());
    }

    private static List<int[]> parseRectangles(String json) {
        List<int[]> result = new ArrayList<>();
        Pattern pattern = Pattern.compile("\\[(\\d+)\\s*,\\s*(\\d+)\\s*,\\s*(\\d+)\\]");
        Matcher matcher = pattern.matcher(json);

        while (matcher.find()) {
            int w = Integer.parseInt(matcher.group(1));
            int h = Integer.parseInt(matcher.group(2));
            int q = Integer.parseInt(matcher.group(3));
            result.add(new int[]{w, h, q});
        }

        return result;
    }
}
"#
    }
}

impl LanguageRunner for JavaRunner {
    fn file_extension(&self) -> &'static str {
        "java"
    }

    fn run(&self, code: &str, bin_width: i32, rectangles: &[Rectangle]) -> RunResult {
        let rectangles_json: Vec<Vec<i32>> = rectangles
            .iter()
            .map(|r| vec![r.width, r.height, r.quantity])
            .collect();
        let rectangles_str = match serde_json::to_string(&rectangles_json) {
            Ok(s) => s,
            Err(e) => {
                return RunResult::Error {
                    errors: vec![format!("Failed to serialize rectangles: {}", e)],
                }
            }
        };

        let temp_dir = std::env::temp_dir();
        let java_dir = temp_dir.join("packing_java");

        // Create temp directory for Java files
        if let Err(e) = std::fs::create_dir_all(&java_dir) {
            return RunResult::Error {
                errors: vec![format!("Failed to create temp directory: {}", e)],
            };
        }

        let packing_path = java_dir.join("Packing.java");
        if let Err(e) = std::fs::write(&packing_path, code) {
            return RunResult::Error {
                errors: vec![format!("Failed to write Packing.java: {}", e)],
            };
        }

        let runner_path = java_dir.join("PackingRunner.java");
        if let Err(e) = std::fs::write(&runner_path, Self::get_runner_code()) {
            return RunResult::Error {
                errors: vec![format!("Failed to write PackingRunner.java: {}", e)],
            };
        }

        let compile_output = std::process::Command::new("javac")
            .current_dir(&java_dir)
            .arg("Packing.java")
            .arg("PackingRunner.java")
            .output();

        match compile_output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return RunResult::Error {
                        errors: stderr.lines().map(|s| s.to_string()).collect(),
                    };
                }
            }
            Err(e) => {
                return RunResult::Error {
                    errors: vec![format!("Failed to run javac: {}", e)],
                };
            }
        }

        let run_output = std::process::Command::new("java")
            .current_dir(&java_dir)
            .arg("PackingRunner")
            .arg(bin_width.to_string())
            .arg(&rectangles_str)
            .output();

        match run_output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
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
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    RunResult::Error {
                        errors: stderr.lines().map(|s| s.to_string()).collect(),
                    }
                }
            }
            Err(e) => RunResult::Error {
                errors: vec![format!("Failed to run java: {}", e)],
            },
        }
    }
}
*/
pub fn get_runner(language: CodeLanguage) -> Box<dyn LanguageRunner> {
    match language {
        CodeLanguage::Python => Box::new(PythonRunner),
        CodeLanguage::Cpp => Box::new(PythonRunner),
        CodeLanguage::Java => Box::new(PythonRunner),
    }
}

pub fn run_code(
    language: CodeLanguage,
    code: &str,
    parsed: &ParseOutput,
) -> RunResult {
    let runner = get_runner(language);
    runner.run(code, parsed.width, &parsed.rects)
}

pub fn run_code_with_testcase(
    language: CodeLanguage,
    code: &str,
    testcase: &JsonInput,
) -> RunResult {
    println!("{:?}", language);
    let runner = get_runner(language);
    runner.run(code, testcase.width_of_bin, &testcase.rectangle_list)
}

pub fn run_repack_code_with_testcase(
    language: CodeLanguage,
    code: &str,
    testcase: &JsonInput,
    bin_height: f32,
    non_empty_space: &[NonEmptySpace]
) -> RunResult {
    println!("{:?}", language);
    let runner = get_runner(language);
    runner.repack_run(code, bin_height, testcase.width_of_bin as f32, &testcase.rectangle_list, non_empty_space)
}
