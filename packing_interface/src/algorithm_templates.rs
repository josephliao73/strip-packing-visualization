use crate::types::CodeLanguage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const CREATE_NEW_TEMPLATE_ID: &str = "__create_new__";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AlgorithmTemplateEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub python_path: Option<String>,
    pub cpp_path: Option<String>,
    #[serde(default)]
    pub builtin: bool,
    #[serde(default = "default_true")]
    pub editable: bool,
}

impl AlgorithmTemplateEntry {
    pub fn supports_language(&self, language: CodeLanguage) -> bool {
        match language {
            CodeLanguage::Cpp => self.cpp_path.is_some(),
            CodeLanguage::Python => self.python_path.is_some(),
            CodeLanguage::Java => false,
        }
    }

    pub fn is_create_new(&self) -> bool {
        self.id == CREATE_NEW_TEMPLATE_ID
    }

    pub fn is_read_only(&self) -> bool {
        !self.editable || self.builtin
    }

    pub fn path_for_language(&self, language: CodeLanguage) -> Option<&str> {
        match language {
            CodeLanguage::Cpp => self.cpp_path.as_deref(),
            CodeLanguage::Python => self.python_path.as_deref(),
            CodeLanguage::Java => None,
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
        .unwrap_or_else(fallback_root_templates)
}

pub fn load_root_template_map() -> HashMap<String, AlgorithmTemplateEntry> {
    load_root_templates()
        .into_iter()
        .map(|template| (template.id.clone(), template))
        .collect()
}

pub fn templates_for_language(
    templates: &[AlgorithmTemplateEntry],
    language: CodeLanguage,
) -> Vec<AlgorithmTemplateEntry> {
    let mut filtered: Vec<_> = templates
        .iter()
        .filter(|template| template.supports_language(language))
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
        .find(|template| template.id == "blank" && template.supports_language(language))
        .cloned()
        .or_else(|| {
            templates
                .iter()
                .find(|template| template.supports_language(language))
                .cloned()
        })
        .unwrap_or_else(|| {
            fallback_root_templates()
                .into_iter()
                .find(|template| template.supports_language(language))
                .unwrap_or_else(|| fallback_root_templates().into_iter().next().unwrap())
        })
}

pub fn load_root_template_code(
    template: &AlgorithmTemplateEntry,
    language: CodeLanguage,
) -> Result<String, String> {
    let relative_path = match language {
        CodeLanguage::Cpp => template.cpp_path.as_ref(),
        _ => template.python_path.as_ref(),
    }
    .ok_or_else(|| {
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

    let mut templates = load_root_templates();
    if templates
        .iter()
        .any(|template| template.name.eq_ignore_ascii_case(display_name))
    {
        return Err(format!("A template named '{}' already exists.", display_name));
    }

    let template = build_custom_template_entry(&templates, language, display_name, description.trim());
    let code = custom_template_starter(language);

    let relative_path = template
        .path_for_language(language)
        .ok_or_else(|| "New template did not get a file path.".to_string())?;
    let full_path = template_dir().join(relative_path);

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
        CodeLanguage::Cpp => include_str!("algorithm_templates/node.cpp"),
        _ => include_str!("algorithm_templates/node.py"),
    }
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

        let file_name = format!("{}.{}", id, extension);
        return AlgorithmTemplateEntry {
            id,
            name: display_name.to_string(),
            description: if description.is_empty() {
                format!("Editable custom {} template.", language_label(language))
            } else {
                description.to_string()
            },
            python_path: (language == CodeLanguage::Python).then_some(file_name.clone()),
            cpp_path: (language == CodeLanguage::Cpp).then_some(file_name),
            builtin: false,
            editable: true,
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
    let dir = template_dir();
    let mut found = false;

    if let Some(path) = &template.python_path {
        found = true;
        if !dir.join(path).exists() {
            return false;
        }
    }

    if let Some(path) = &template.cpp_path {
        found = true;
        if !dir.join(path).exists() {
            return false;
        }
    }

    found || template.is_create_new()
}

fn create_new_entry(language: CodeLanguage) -> AlgorithmTemplateEntry {
    AlgorithmTemplateEntry {
        id: CREATE_NEW_TEMPLATE_ID.to_string(),
        name: "Create New".to_string(),
        description: format!(
            "Create a new editable {} template saved under algorithm_templates/.",
            language_label(language)
        ),
        python_path: None,
        cpp_path: None,
        builtin: false,
        editable: false,
    }
}

fn save_manifest(templates: &[AlgorithmTemplateEntry]) -> Result<(), String> {
    let manifest_path = template_dir().join("manifest.json");
    let contents = serde_json::to_string_pretty(templates)
        .map_err(|error| format!("Failed to serialize template manifest: {}", error))?;
    fs::write(&manifest_path, format!("{}\n", contents))
        .map_err(|error| format!("Failed to write {}: {}", manifest_path.display(), error))
}

fn template_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/algorithm_templates")
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

fn fallback_root_templates() -> Vec<AlgorithmTemplateEntry> {
    vec![
        AlgorithmTemplateEntry {
            id: "blank".to_string(),
            name: "Blank".to_string(),
            description: "Minimal starter template using the helper library.".to_string(),
            python_path: Some("blank_root.py".to_string()),
            cpp_path: Some("blank_root.cpp".to_string()),
            builtin: true,
            editable: false,
        },
        AlgorithmTemplateEntry {
            id: "nfdh".to_string(),
            name: "NFDH".to_string(),
            description: "Next-Fit Decreasing Height strip packing.".to_string(),
            python_path: Some("nfdh.py".to_string()),
            cpp_path: Some("nfdh.cpp".to_string()),
            builtin: true,
            editable: false,
        },
        AlgorithmTemplateEntry {
            id: "ffdh".to_string(),
            name: "FFDH".to_string(),
            description: "First-Fit Decreasing Height strip packing.".to_string(),
            python_path: Some("ffdh.py".to_string()),
            cpp_path: Some("ffdh.cpp".to_string()),
            builtin: true,
            editable: false,
        },
        AlgorithmTemplateEntry {
            id: "fspp".to_string(),
            name: "FSPP".to_string(),
            description: "Fractional strip-packing template.".to_string(),
            python_path: Some("fspp.py".to_string()),
            cpp_path: None,
            builtin: true,
            editable: false,
        },
        AlgorithmTemplateEntry {
            id: "three_fspp".to_string(),
            name: "3FSPP".to_string(),
            description: "Three-strip fractional strip-packing template.".to_string(),
            python_path: Some("three_fspp.py".to_string()),
            cpp_path: None,
            builtin: true,
            editable: false,
        },
    ]
}
