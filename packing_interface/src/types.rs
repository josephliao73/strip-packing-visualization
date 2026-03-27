use crate::algorithm_templates::AlgorithmTemplateEntry;
use serde::{Serialize, Deserialize};
use iced::widget::{text_editor};
use std::collections::{HashMap, HashSet};
use ordered_float::OrderedFloat;

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub area_select_enabled: bool,
    pub snap_to_rectangles_enabled: bool,
    pub auto_minimize_height: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightPanelTab {
    Visualization,
    CodeEditor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomPanelTab {
    Output,
    TestCases,
    MultipleTestCases,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeLanguage {
    Python,
    Cpp,
    Java,
}

impl std::fmt::Display for CodeLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeLanguage::Python => write!(f, "Python"),
            CodeLanguage::Cpp => write!(f, "C++"),
            CodeLanguage::Java => write!(f, "Java"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTab {
    CreateSingleTestCase,
    ImportGenerateTestCases,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Rectangle {
    pub width: i32,
    pub height: i32,
    pub quantity: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub struct NonEmptySpace {
    pub x_1: f32,
    pub x_2: f32,
    pub y_1: f32,
    pub y_2: f32,
}

#[derive(Debug, Clone)]
pub struct MultipleRunResult {
    pub testcase: JsonInput,
    pub height: Option<f32>,
    pub output: Option<AlgorithmOutput>,
    pub tab_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonInput {
    pub width_of_bin: i32,
    pub number_of_rectangles: usize,
    pub number_of_types_of_rectangles: usize,
    pub autofill_option: bool,
    pub rectangle_list: Vec<Rectangle>
}

#[derive(Debug, Clone)]
pub enum Input {
    WChanged(String),
    NChanged(String),
    KChanged(String),
    AutofillChanged(bool),
    ImportPressed,
    ImportOutputJsonPressed,
    RectangleDataAction(text_editor::Action),
    ExportAlgorithmInput,
    ZoomChanged(f32),
    Tick,
    AnimationSpeedChanged(f32),
    PanStart(f32, f32),
    PanMove(f32, f32),
    PanEnd,
    RectangleHovered(Option<usize>),
    RectangleDragStart(usize, f32, f32),
    RectangleDragMove(f32, f32, f32),  // screen x, screen y, scale
    RectangleDragEnd(bool, bool, OrderedFloat<f32>, OrderedFloat<f32>),
    RightClickCanvas(Option<usize>),
    TabSelected(RightPanelTab),
    AlgoTabSelected(u64),
    RemoveAlgoTab(u64),
    CodeEditorAction(text_editor::Action),
    LanguageSelected(CodeLanguage),
    AlgorithmTemplateSelected(AlgorithmTemplateEntry),
    TemplateNameChanged(String),
    TemplateDescriptionChanged(String),
    ConfirmCreateTemplate,
    CancelCreateTemplate,
    TemplateReadOnlyHover(bool),
    RunCode(i32),
    SaveOutputToFile,
    InsertTab,
    ImportTestCase,
    GenerateTestCase,
    GenerateMultipleTestCases(i32),
    ToggleAreaSelectEnabled(bool),
    ToggleSnapToRectangles(bool),
    ToggleAutoMinimizeHeight(bool),
    NumTestCasesChanged(String),
    InputSizeChanged(String),
    UniqueTypesChanged(String),
    SingleInputSizeChanged(String),
    SingleUniqueTypesChanged(String),
    SingleBinWidthChanged(String),
    BatchBinWidthChanged(String),
    ToggleMultipleResultExpanded(usize),
    AreaSelectStart(f32, f32),
    AreaSelectMove(f32, f32),
    AreaSelectEnd(Vec<usize>, f32, f32, f32, f32),
    ShowRegionContextMenu(usize, f32, f32),  // region index, x, y position
    HideContextMenu,
    RemoveSelectionRegion(usize),
    RepackSelectionRegion(usize),
    PanelResizeStart,
    PanelResizeMove(f32),
    PanelResizeEnd,
    DisplayMultipleResult(usize),
    CreateNewTab,
    WorkspaceTabSelected(WorkspaceTab),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, PartialOrd, Hash, Eq, Copy)]
pub struct Placement {
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlgorithmOutput {
    pub bin_width: i32,
    pub total_height: f32,
    pub placements: Vec<Placement>,
}

#[derive(Debug, Clone)]
pub struct SelectionRegion {
    pub is_inherited: bool,  // true = inherited from parent tab, false = newly created
    // Stored in bin coordinates (logical units, not screen pixels)
    pub bin_x: f32,
    pub bin_y: f32,
    pub bin_w: f32,
    pub bin_h: f32,
    pub selected_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct RootTabState {
    pub w_input: String,
    pub n_input: String,
    pub k_input: String,
    pub autofile: bool,
    pub rectangle_text: String,
    pub current_testcase: Option<JsonInput>,
    pub testcase_message: Option<String>,
    pub num_test_cases_input: String,
    pub input_size_input: String,
    pub unique_types_input: String,
    pub single_input_size_input: String,
    pub single_unique_types_input: String,
    pub single_bin_width_input: String,
    pub batch_bin_width_input: String,
    pub multiple_test_cases: Vec<JsonInput>,
    pub multiple_testcase_message: Option<String>,
    pub multiple_run_results: Vec<MultipleRunResult>,
    pub multiple_results_expanded: Vec<bool>,
    pub batch_run_in_progress: bool,
    pub batch_run_total: usize,
    pub batch_run_completed: usize,
    pub batch_run_failures: usize,
    pub batch_run_code: Option<String>,
    pub batch_run_language: Option<CodeLanguage>,
}

impl Default for RootTabState {
    fn default() -> Self {
        Self {
            w_input: String::new(),
            n_input: String::new(),
            k_input: String::new(),
            autofile: false,
            rectangle_text: String::new(),
            current_testcase: None,
            testcase_message: None,
            num_test_cases_input: String::new(),
            input_size_input: String::new(),
            unique_types_input: String::new(),
            single_input_size_input: String::new(),
            single_unique_types_input: String::new(),
            single_bin_width_input: String::new(),
            batch_bin_width_input: String::new(),
            multiple_test_cases: Vec::new(),
            multiple_testcase_message: None,
            multiple_run_results: Vec::new(),
            multiple_results_expanded: Vec::new(),
            batch_run_in_progress: false,
            batch_run_total: 0,
            batch_run_completed: 0,
            batch_run_failures: 0,
            batch_run_code: None,
            batch_run_language: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AlgoTab {
    pub id: u64,
    pub name: String,
    pub selected_indices: Vec<usize>,
    pub repacked_indices: Vec<usize>,
    pub obstacle_spaces: Vec<NonEmptySpace>,
    pub selection_regions: Vec<SelectionRegion>,
    pub code: String,
    pub language: CodeLanguage,
    pub algorithm_template: Option<AlgorithmTemplateEntry>,
    pub python_code: String,
    pub cpp_code: String,
    pub python_template: Option<AlgorithmTemplateEntry>,
    pub cpp_template: Option<AlgorithmTemplateEntry>,
    pub python_drafts: HashMap<String, String>,
    pub cpp_drafts: HashMap<String, String>,
    pub last_right_panel_tab: RightPanelTab,
    pub algorithm_output: Option<AlgorithmOutput>,
    pub parent_output: Option<AlgorithmOutput>,
    pub repack_output: Option<AlgorithmOutput>,
    pub output_revision: u64,
    pub hit_grid: Option<HitGrid>,
    pub visible_rects: usize,
    pub animating: bool,
    pub root_state: Option<RootTabState>,
}

#[derive(Debug, Clone)]
pub struct HitGrid {
    pub cell_size: f32,
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<Vec<usize>>,
}

impl AlgoTab {
    pub fn template_for_language(&self, language: CodeLanguage) -> Option<AlgorithmTemplateEntry> {
        match language {
            CodeLanguage::Python => self.python_template.clone(),
            CodeLanguage::Cpp => self.cpp_template.clone(),
            CodeLanguage::Java => None,
        }
    }

    pub fn set_template_for_language(
        &mut self,
        language: CodeLanguage,
        template: Option<AlgorithmTemplateEntry>,
    ) {
        match language {
            CodeLanguage::Python => self.python_template = template,
            CodeLanguage::Cpp => self.cpp_template = template,
            CodeLanguage::Java => {}
        }
    }

    pub fn code_for_language(&self, language: CodeLanguage) -> String {
        match language {
            CodeLanguage::Python => self.python_code.clone(),
            CodeLanguage::Cpp => self.cpp_code.clone(),
            CodeLanguage::Java => self.code.clone(),
        }
    }

    pub fn set_code_for_language(&mut self, language: CodeLanguage, code: String) {
        match language {
            CodeLanguage::Python => self.python_code = code,
            CodeLanguage::Cpp => self.cpp_code = code,
            CodeLanguage::Java => self.code = code,
        }
    }

    pub fn drafts_for_language(&self, language: CodeLanguage) -> &HashMap<String, String> {
        match language {
            CodeLanguage::Python => &self.python_drafts,
            CodeLanguage::Cpp => &self.cpp_drafts,
            CodeLanguage::Java => &self.python_drafts,
        }
    }

    pub fn drafts_for_language_mut(&mut self, language: CodeLanguage) -> &mut HashMap<String, String> {
        match language {
            CodeLanguage::Python => &mut self.python_drafts,
            CodeLanguage::Cpp => &mut self.cpp_drafts,
            CodeLanguage::Java => &mut self.python_drafts,
        }
    }
}

pub struct PackingApp {
    pub lang_map: HashMap<String, bool>,
    pub w_input: String,
    pub n_input: String,
    pub k_input: String,
    pub autofile: bool,
    pub rectangle_data: text_editor::Content,
    pub rect_total_lines: usize,
    pub rect_cursor_line: usize,
    pub error_message: Option<String>,
    pub algo_tabs: Vec<AlgoTab>,
    pub active_algo_tab_id: u64,
    pub next_algo_tab_id: u64,
    pub zoom: f32,
    pub animation_speed: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub is_panning: bool,
    pub last_mouse_x: f32,
    pub last_mouse_y: f32,
    pub hovered_rect: Option<usize>,
    pub dragged_rect: Option<usize>,
    pub dragged_rect_offset_x: f32,
    pub dragged_rect_offset_y: f32,
    pub snap_preview: Option<(f32, f32)>,  // (x, y) in bin coordinates where rect will snap
    pub selected_rects: HashSet<usize>,
    pub active_tab: RightPanelTab,
    pub current_testcase: Option<JsonInput>,
    pub testcase_message: Option<String>,
    pub code_editor_content: text_editor::Content,
    pub selected_language: CodeLanguage,
    pub python_template_menu_selection: Option<AlgorithmTemplateEntry>,
    pub cpp_template_menu_selection: Option<AlgorithmTemplateEntry>,
    pub available_templates: Vec<AlgorithmTemplateEntry>,
    pub python_template_options: Vec<AlgorithmTemplateEntry>,
    pub cpp_template_options: Vec<AlgorithmTemplateEntry>,
    pub create_template_modal_open: bool,
    pub create_template_name_input: String,
    pub create_template_description_input: String,
    pub create_template_language: Option<CodeLanguage>,
    pub template_read_only_hovered: bool,
    // Bottom panel state
    pub bottom_panel_visible: bool,
    pub bottom_panel_tab: BottomPanelTab,
    pub code_errors: Vec<String>,
    pub code_output_json: Option<String>,
    pub settings: Settings,
    pub settings_panel_visible: bool,
    // Area selection state
    pub area_select_list: Vec<(f32, f32)>,
    pub new_area_select: bool,
    pub area_select_start: Option<(f32, f32)>,
    pub area_select_current: Option<(f32, f32)>,
    pub is_area_selecting: bool,
    // Context menu for selection regions (regions are now stored per-tab in AlgoTab)
    pub context_menu_visible: bool,
    pub context_menu_region: Option<usize>,
    pub context_menu_position: (f32, f32),
    pub num_test_cases_input: String,
    pub input_size_input: String,
    pub unique_types_input: String,
    pub single_input_size_input: String,
    pub single_unique_types_input: String,
    pub single_bin_width_input: String,
    pub batch_bin_width_input: String,
    pub display_visual: bool,
    pub multiple_test_cases: Vec<JsonInput>,
    pub multiple_testcase_message: Option<String>,
    pub multiple_run_results: Vec<MultipleRunResult>,
    pub multiple_results_expanded: Vec<bool>,
    pub batch_run_in_progress: bool,
    pub batch_run_total: usize,
    pub batch_run_completed: usize,
    pub batch_run_failures: usize,
    pub batch_run_code: Option<String>,
    pub batch_run_language: Option<CodeLanguage>,
    pub bottom_panel_height: f32,
    pub is_resizing_panel: bool,
    pub panel_drag_last_y: f32,
    pub workspace_tab: WorkspaceTab,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParseOutput {
    pub width: i32,
    pub quantity: i32,
    pub types: i32,
    pub autofill: bool,
    pub rects: Vec<Rectangle>,
    pub input_types: i32,
    pub min_height: i32,
    pub max_height: i32,
}

pub struct BinCanvas<'a>  {
    pub output: &'a AlgorithmOutput,
    pub output_revision: u64,
    pub hit_grid: Option<&'a HitGrid>,
    pub zoom: f32,
    pub visible_count: usize,
    pub pan_x: f32,
    pub pan_y: f32,
    pub hovered_rect: Option<usize>,
    pub is_panning: bool,
    pub dragged_rect: Option<usize>,
    pub dragged_rect_offset_x: f32,
    pub dragged_rect_offset_y: f32,
    pub snap_preview: Option<(f32, f32)>,  // (x, y) in bin coordinates
    pub animating: bool,
    pub selected_rects: &'a HashSet<usize>,
    pub repacked_indices: Option<&'a [usize]>,
    pub obstacle_spaces: Option<&'a [NonEmptySpace]>,
    pub is_area_selecting: bool,
    pub area_select_start: Option<(f32, f32)>,
    pub area_select_current: Option<(f32, f32)>,
    pub settings: &'a Settings,
    pub selection_regions: &'a [SelectionRegion],
}
