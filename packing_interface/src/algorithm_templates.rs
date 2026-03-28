use crate::types::CodeLanguage;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CREATE_NEW_TEMPLATE_ID: &str = "__create_new__";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AlgorithmTemplateEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: CodeLanguage,
    pub path: Option<String>,
    #[serde(default)]
    pub builtin: bool,
    #[serde(default = "default_true")]
    pub editable: bool,
    #[serde(default = "default_true")]
    pub is_root: bool,
}

impl AlgorithmTemplateEntry {
    pub fn supports_language(&self, language: CodeLanguage) -> bool {
        self.language == language
    }

    pub fn is_create_new(&self) -> bool {
        self.id == CREATE_NEW_TEMPLATE_ID
    }

    pub fn is_read_only(&self) -> bool {
        !self.editable || self.builtin
    }

    pub fn path_for_language(&self, language: CodeLanguage) -> Option<&str> {
        if self.language == language {
            self.path.as_deref()
        } else {
            None
        }
    }
}

impl std::fmt::Display for AlgorithmTemplateEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_create_new() {
            write!(f, "Create New...")
        } else if self.builtin {
            write!(f, "{} [DEFAULT]", self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

pub fn load_root_templates() -> Vec<AlgorithmTemplateEntry> {
    load_templates()
        .into_iter()
        .filter(|template| template.is_root)
        .collect()
}

pub fn templates_for_language(
    templates: &[AlgorithmTemplateEntry],
    language: CodeLanguage,
) -> Vec<AlgorithmTemplateEntry> {
    let mut filtered: Vec<_> = templates
        .iter()
        .filter(|template| template.is_root && template.supports_language(language))
        .cloned()
        .collect();

    if matches!(language, CodeLanguage::Python | CodeLanguage::Cpp) {
        filtered.push(create_new_entry(language));
    }

    filtered
}

pub fn default_root_template_for_language(
    templates: &[AlgorithmTemplateEntry],
    language: CodeLanguage,
) -> AlgorithmTemplateEntry {
    templates
        .iter()
        .find(|template| {
            template.is_root
                && template.supports_language(language)
                && template.name.eq_ignore_ascii_case("Blank")
        })
        .cloned()
        .or_else(|| {
            templates
                .iter()
                .find(|template| template.is_root && template.supports_language(language))
                .cloned()
        })
        .unwrap_or_else(|| {
            fallback_templates()
                .into_iter()
                .find(|template| template.is_root && template.supports_language(language))
                .unwrap_or_else(|| fallback_templates().into_iter().next().unwrap())
        })
}

pub fn load_root_template_code(
    template: &AlgorithmTemplateEntry,
    language: CodeLanguage,
) -> Result<String, String> {
    let relative_path = template.path_for_language(language).ok_or_else(|| {
        format!(
            "Template '{}' does not define a {} file.",
            template.name,
            language_label(language)
        )
    })?;

    let full_path = template_dir().join(relative_path);
    fs::read_to_string(&full_path).map_err(|error| {
        format!(
            "Failed to load template '{}' from {}: {}",
            template.name,
            full_path.display(),
            error
        )
    })
}

pub fn default_root_code(language: CodeLanguage) -> String {
    let templates = load_root_templates();
    let template = default_root_template_for_language(&templates, language);
    load_root_template_code(&template, language).unwrap_or_default()
}

pub fn custom_template_starter(language: CodeLanguage) -> String {
    match language {
        CodeLanguage::Python => {
            [
                "import packing_lib",
                "from typing import List, Tuple",
                "",
                "class Packing:",
                "    def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:",
                "        placements = []",
                "        return packing_lib.output_from_placements(bin_width, placements)",
                "",
            ]
            .join("\n")
        }
        CodeLanguage::Cpp => {
            [
                "#include \"packing_lib.h\"",
                "using namespace packing;",
                "",
                "class Packing {",
                "public:",
                "    std::vector<std::tuple<double, double, int, int>> solve(",
                "        int binWidth,",
                "        const std::vector<std::tuple<int, int, int>>& rectangles",
                "    ) {",
                "        std::vector<std::tuple<double, double, int, int>> placements;",
                "        return placements;",
                "    }",
                "};",
                "",
            ]
            .join("\n")
        }
        CodeLanguage::Java => String::new(),
    }
}

pub fn create_custom_template(
    language: CodeLanguage,
    display_name: &str,
    description: &str,
) -> Result<AlgorithmTemplateEntry, String> {
    if !matches!(language, CodeLanguage::Python | CodeLanguage::Cpp) {
        return Err("Custom templates are only supported for Python and C++.".to_string());
    }

    let display_name = display_name.trim();
    if display_name.is_empty() {
        return Err("Template name is required.".to_string());
    }

    let mut templates = load_templates();
    if templates.iter().any(|template| {
        template.is_root
            && template.language == language
            && template.name.eq_ignore_ascii_case(display_name)
    }) {
        return Err(format!("A template named '{}' already exists.", display_name));
    }

    let template = build_custom_template_entry(&templates, language, display_name, description.trim());
    let code = custom_template_starter(language);

    let relative_path = template
        .path_for_language(language)
        .ok_or_else(|| "New template did not get a file path.".to_string())?;
    let full_path = template_dir().join(relative_path);

    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {}", parent.display(), error))?;
    }

    fs::write(&full_path, code)
        .map_err(|error| format!("Failed to create template file {}: {}", full_path.display(), error))?;

    templates.push(template.clone());
    save_manifest(&templates)?;
    Ok(template)
}

pub fn save_custom_template_code(
    template: &AlgorithmTemplateEntry,
    language: CodeLanguage,
    code: &str,
) -> Result<(), String> {
    if template.is_read_only() {
        return Ok(());
    }

    let relative_path = template.path_for_language(language).ok_or_else(|| {
        format!(
            "Template '{}' has no {} file.",
            template.name,
            language_label(language)
        )
    })?;
    let full_path = template_dir().join(relative_path);
    fs::write(&full_path, code)
        .map_err(|error| format!("Failed to save {}: {}", full_path.display(), error))
}

pub fn default_node_code(language: CodeLanguage) -> &'static str {
    match language {
        CodeLanguage::Cpp => include_str!("algorithm_templates/cpp/node/default.cpp"),
        _ => include_str!("algorithm_templates/python/node/default.py"),
    }
}

fn load_templates() -> Vec<AlgorithmTemplateEntry> {
    let manifest_path = template_dir().join("manifest.json");
    fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|contents| serde_json::from_str::<Vec<AlgorithmTemplateEntry>>(&contents).ok())
        .map(|templates| {
            templates
                .into_iter()
                .filter(template_files_exist)
                .collect::<Vec<_>>()
        })
        .filter(|templates| !templates.is_empty())
        .unwrap_or_else(fallback_templates)
}

fn build_custom_template_entry(
    templates: &[AlgorithmTemplateEntry],
    language: CodeLanguage,
    display_name: &str,
    description: &str,
) -> AlgorithmTemplateEntry {
    let prefix = match language {
        CodeLanguage::Python => "custom_python_",
        CodeLanguage::Cpp => "custom_cpp_",
        CodeLanguage::Java => "custom_",
    };
    let extension = match language {
        CodeLanguage::Python => "py",
        CodeLanguage::Cpp => "cpp",
        CodeLanguage::Java => "txt",
    };

    let mut index = 1usize;
    loop {
        let base_slug = sanitize_template_slug(display_name);
        let id = format!("{}{}_{}", prefix, base_slug, index);
        if templates.iter().any(|template| template.id == id) {
            index += 1;
            continue;
        }

        let relative_path = format!(
            "{}/root/{}.{}",
            language_dir(language),
            id,
            extension
        );
        return AlgorithmTemplateEntry {
            id,
            name: display_name.to_string(),
            description: if description.is_empty() {
                format!("Editable custom {} template.", language_label(language))
            } else {
                description.to_string()
            },
            language,
            path: Some(relative_path),
            builtin: false,
            editable: true,
            is_root: true,
        };
    }
}

fn sanitize_template_slug(name: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_underscore = false;

    for ch in name.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            slug.push(lower);
            previous_was_underscore = false;
        } else if !previous_was_underscore {
            slug.push('_');
            previous_was_underscore = true;
        }
    }

    let slug = slug.trim_matches('_').to_string();
    if slug.is_empty() {
        "custom_template".to_string()
    } else {
        slug
    }
}

fn template_files_exist(template: &AlgorithmTemplateEntry) -> bool {
    if template.is_create_new() {
        return true;
    }

    template
        .path
        .as_ref()
        .map(|path| template_dir().join(path).exists())
        .unwrap_or(false)
}

fn create_new_entry(language: CodeLanguage) -> AlgorithmTemplateEntry {
    AlgorithmTemplateEntry {
        id: CREATE_NEW_TEMPLATE_ID.to_string(),
        name: "Create New".to_string(),
        description: format!(
            "Create a new editable {} template saved under algorithm_templates/{}/root/.",
            language_label(language),
            language_dir(language)
        ),
        language,
        path: None,
        builtin: false,
        editable: false,
        is_root: true,
    }
}

fn save_manifest(templates: &[AlgorithmTemplateEntry]) -> Result<(), String> {
    let manifest_path = template_dir().join("manifest.json");
    let contents = serde_json::to_string_pretty(templates)
        .map_err(|error| format!("Failed to serialize template manifest: {}", error))?;
    fs::write(&manifest_path, format!("{}
", contents))
        .map_err(|error| format!("Failed to write {}: {}", manifest_path.display(), error))
}

fn template_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/algorithm_templates")
}

fn language_dir(language: CodeLanguage) -> &'static str {
    match language {
        CodeLanguage::Cpp => "cpp",
        CodeLanguage::Python => "python",
        CodeLanguage::Java => "java",
    }
}

fn language_label(language: CodeLanguage) -> &'static str {
    match language {
        CodeLanguage::Cpp => "C++",
        CodeLanguage::Python => "Python",
        CodeLanguage::Java => "Java",
    }
}

fn default_true() -> bool {
    true
}

fn fallback_templates() -> Vec<AlgorithmTemplateEntry> {
    vec![
        AlgorithmTemplateEntry {
            id: "blank_python".to_string(),
            name: "Blank".to_string(),
            description: "Minimal starter template using the helper library.".to_string(),
            language: CodeLanguage::Python,
            path: Some("python/root/blank.py".to_string()),
            builtin: true,
            editable: false,
            is_root: true,
        },
        AlgorithmTemplateEntry {
            id: "blank_cpp".to_string(),
            name: "Blank".to_string(),
            description: "Minimal starter template using the helper library.".to_string(),
            language: CodeLanguage::Cpp,
            path: Some("cpp/root/blank.cpp".to_string()),
            builtin: true,
            editable: false,
            is_root: true,
        },
        AlgorithmTemplateEntry {
            id: "nfdh_python".to_string(),
            name: "NFDH".to_string(),
            description: "Next-Fit Decreasing Height strip packing.".to_string(),
            language: CodeLanguage::Python,
            path: Some("python/root/nfdh.py".to_string()),
            builtin: true,
            editable: false,
            is_root: true,
        },
        AlgorithmTemplateEntry {
            id: "nfdh_cpp".to_string(),
            name: "NFDH".to_string(),
            description: "Next-Fit Decreasing Height strip packing.".to_string(),
            language: CodeLanguage::Cpp,
            path: Some("cpp/root/nfdh.cpp".to_string()),
            builtin: true,
            editable: false,
            is_root: true,
        },
        AlgorithmTemplateEntry {
            id: "ffdh_python".to_string(),
            name: "FFDH".to_string(),
            description: "First-Fit Decreasing Height strip packing.".to_string(),
            language: CodeLanguage::Python,
            path: Some("python/root/ffdh.py".to_string()),
            builtin: true,
            editable: false,
            is_root: true,
        },
        AlgorithmTemplateEntry {
            id: "ffdh_cpp".to_string(),
            name: "FFDH".to_string(),
            description: "First-Fit Decreasing Height strip packing.".to_string(),
            language: CodeLanguage::Cpp,
            path: Some("cpp/root/ffdh.cpp".to_string()),
            builtin: true,
            editable: false,
            is_root: true,
        },
        AlgorithmTemplateEntry {
            id: "fspp_python".to_string(),
            name: "FSPP".to_string(),
            description: "Fractional strip-packing template.".to_string(),
            language: CodeLanguage::Python,
            path: Some("python/root/fspp.py".to_string()),
            builtin: true,
            editable: false,
            is_root: true,
        },
        AlgorithmTemplateEntry {
            id: "three_fspp_python".to_string(),
            name: "3FSPP".to_string(),
            description: "Three-strip fractional strip-packing template.".to_string(),
            language: CodeLanguage::Python,
            path: Some("python/root/three_fspp.py".to_string()),
            builtin: true,
            editable: false,
            is_root: true,
        },
        AlgorithmTemplateEntry {
            id: "default_node_python".to_string(),
            name: "Default Node".to_string(),
            description: "Default Python repacking template.".to_string(),
            language: CodeLanguage::Python,
            path: Some("python/node/default.py".to_string()),
            builtin: true,
            editable: false,
            is_root: false,
        },
        AlgorithmTemplateEntry {
            id: "default_node_cpp".to_string(),
            name: "Default Node".to_string(),
            description: "Default C++ repacking template.".to_string(),
            language: CodeLanguage::Cpp,
            path: Some("cpp/node/default.cpp".to_string()),
            builtin: true,
            editable: false,
            is_root: false,
        },
        AlgorithmTemplateEntry {
            id: "bfdh_python".to_string(),
            name: "BFDH".to_string(),
            description: "Best fit decreasing height".to_string(),
            language: CodeLanguage::Cpp,
            path: Some("python/root/bfdh.cpp".to_string()),
            builtin: true,
            editable: false,
            is_root: true,
        },
        AlgorithmTemplateEntry {
            id: "rf_python".to_string(),
            name: "RF".to_string(),
            description: "Reverse Fit".to_string(),
            language: CodeLanguage::Python,
            path: Some("python/root/rf.py".to_string()),
            builtin: true,
            editable: false,
            is_root: true,
        },
        AlgorithmTemplateEntry {
            id: "RF_cpp".to_string(),
            name: "RF".to_string(),
            description: "Reverse fit".to_string(),
            language: CodeLanguage::Cpp,
            path: Some("cpp/root/rf.cpp".to_string()),
            builtin: true,
            editable: false,
            is_root: true,
        },
        AlgorithmTemplateEntry {
            id: "bfdh_python".to_string(),
            name: "BFDH".to_string(),
            description: "Best fit decreasing height".to_string(),
            language: CodeLanguage::Cpp,
            path: Some("cpp/root/rf.cpp".to_string()),
            builtin: true,
            editable: false,
            is_root: true,
        },
    ]
}
