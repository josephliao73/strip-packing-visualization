use crate::types::CodeLanguage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgorithmTemplate {
    Blank,
    Nfdh,
    Ffdh,
    Fspp,
    ThreeFspp,
}

pub const ROOT_ALGORITHM_TEMPLATES: [AlgorithmTemplate; 5] = [
    AlgorithmTemplate::Blank,
    AlgorithmTemplate::Nfdh,
    AlgorithmTemplate::Ffdh,
    AlgorithmTemplate::Fspp,
    AlgorithmTemplate::ThreeFspp,
];

impl AlgorithmTemplate {
    pub fn label(self) -> &'static str {
        match self {
            AlgorithmTemplate::Blank => "Blank",
            AlgorithmTemplate::Nfdh => "NFDH",
            AlgorithmTemplate::Ffdh => "FFDH",
            AlgorithmTemplate::Fspp => "FSPP",
            AlgorithmTemplate::ThreeFspp => "3FSPP",
        }
    }

    pub fn root_code(self, language: CodeLanguage) -> &'static str {
        match language {
            CodeLanguage::Cpp => cpp_root_code(),
            _ => match self {
                AlgorithmTemplate::Blank => include_str!("algorithm_templates/blank_root.py"),
                AlgorithmTemplate::Nfdh => include_str!("algorithm_templates/nfdh.py"),
                AlgorithmTemplate::Ffdh => include_str!("algorithm_templates/ffdh.py"),
                AlgorithmTemplate::Fspp => include_str!("algorithm_templates/fspp.py"),
                AlgorithmTemplate::ThreeFspp => include_str!("algorithm_templates/three_fspp.py"),
            },
        }
    }
}

impl std::fmt::Display for AlgorithmTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

pub fn default_root_code(language: CodeLanguage) -> &'static str {
    AlgorithmTemplate::Blank.root_code(language)
}

pub fn default_node_code(language: CodeLanguage) -> &'static str {
    match language {
        CodeLanguage::Cpp => cpp_node_code(),
        _ => include_str!("algorithm_templates/node.py"),
    }
}

fn cpp_root_code() -> &'static str {
    include_str!("algorithm_templates/root.cpp")
}

fn cpp_node_code() -> &'static str {
    include_str!("algorithm_templates/node.cpp")
}
