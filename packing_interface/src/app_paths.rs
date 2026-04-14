use std::path::{Path, PathBuf};

fn executable_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
}

fn candidate_roots() -> Vec<PathBuf> {
    let mut roots = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR"))];

    if let Some(exe_dir) = executable_dir() {
        roots.push(exe_dir.clone());
        if let Some(parent) = exe_dir.parent() {
            roots.push(parent.to_path_buf());
        }
    }

    roots.sort();
    roots.dedup();
    roots
}

fn find_existing_path(relative: &str) -> Option<PathBuf> {
    candidate_roots()
        .into_iter()
        .map(|root| root.join(relative))
        .find(|path| path.exists())
}

pub fn template_dir() -> PathBuf {
    find_existing_path("src/algorithm_templates")
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/algorithm_templates"))
}

pub fn runner_utils_dir() -> PathBuf {
    find_existing_path("src/runner_utils")
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/runner_utils"))
}

pub fn python_runner_path() -> PathBuf {
    runner_utils_dir().join("python_runner.py")
}

pub fn python_bin_path() -> std::ffi::OsString {
    candidate_roots()
        .into_iter()
        .map(|root| root.join(".venv/bin/python3"))
        .find(|path| path.exists())
        .map(PathBuf::into_os_string)
        .unwrap_or_else(|| std::ffi::OsString::from("python3"))
}
