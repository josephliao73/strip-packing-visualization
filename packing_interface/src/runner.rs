use crate::types::{AlgorithmOutput, CodeLanguage, JsonInput, ParseOutput, Rectangle};
use std::path::PathBuf;

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
}

pub struct PythonRunner;

impl PythonRunner {
    fn get_runner_path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("runner_utils").join("python_runner.py"))
            .unwrap_or_else(|| PathBuf::from("src/runner_utils/python_runner.py"))
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

        // Write user code to temp file
        let temp_dir = std::env::temp_dir();
        let solution_path = temp_dir.join("packing_solution.py");

        if let Err(e) = std::fs::write(&solution_path, code) {
            return RunResult::Error {
                errors: vec![format!("Failed to write solution file: {}", e)],
            };
        }

        let runner_path = Self::get_runner_path();

        let output = std::process::Command::new("python3")
            .arg(&runner_path)
            .arg(&solution_path)
            .arg(bin_width.to_string())
            .arg(&rectangles_str)
            .output();

        match output {
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
                errors: vec![format!("Failed to run Python: {}", e)],
            },
        }
    }
}

pub struct CppRunner;

impl LanguageRunner for CppRunner {
    fn file_extension(&self) -> &'static str {
        "cpp"
    }

    fn run(&self, _code: &str, _bin_width: i32, _rectangles: &[Rectangle]) -> RunResult {
        RunResult::Error {
            errors: vec!["C++ runner not yet implemented".to_string()],
        }
    }
}

pub struct JavaRunner;

impl LanguageRunner for JavaRunner {
    fn file_extension(&self) -> &'static str {
        "java"
    }

    fn run(&self, _code: &str, _bin_width: i32, _rectangles: &[Rectangle]) -> RunResult {
        RunResult::Error {
            errors: vec!["Java runner not yet implemented".to_string()],
        }
    }
}

pub fn get_runner(language: CodeLanguage) -> Box<dyn LanguageRunner> {
    match language {
        CodeLanguage::Python => Box::new(PythonRunner),
        CodeLanguage::Cpp => Box::new(CppRunner),
        CodeLanguage::Java => Box::new(JavaRunner),
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
    let runner = get_runner(language);
    runner.run(code, testcase.width_of_bin, &testcase.rectangle_list)
}
