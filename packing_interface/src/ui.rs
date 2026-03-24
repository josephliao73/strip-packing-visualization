use crate::config_parser::create_input;
use crate::editor::{build_code_panel, EditorState};
use crate::runner::{RunResult, run_code, run_code_with_testcase, run_repack_code_with_testcase};
use iced::widget::{button, checkbox, column, container, row, text, text_input, text_editor, scrollable, slider};
use iced::{Element, Theme, Alignment, Length, Color, Font, time, Subscription};
use std::collections::HashSet;
use iced::widget::canvas::Canvas;
use crate::types::{AlgoTab, AlgorithmOutput, BinCanvas, BottomPanelTab, CodeLanguage, Input, JsonInput, MultipleRunResult, PackingApp, ParseOutput, Rectangle, RightPanelTab, SelectionRegion, Settings, NonEmptySpace, Placement};
use std::time::Duration;
use ordered_float::OrderedFloat;
use rand::Rng;

const ROOT_CODE: &str = r#"
import packing_lib
import json
from typing import List, Tuple

class Packing:
    def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:
        items = packing_lib.expand_items(rectangles)

        placements = []
        total_height = 0
        for rect in items:
            w = rect["width"]
            h = rect["height"]

            placements.append([0, total_height, w, h])
            total_height += h

        return packing_lib.make_output(bin_width, total_height, placements)
"#;

const NODE_CODE: &str = r#"
import packing_lib
import json
from typing import List, Tuple

class Repacking:
    def solve(self, bin_height: int, bin_width: int, rectangles: List[Tuple[int, int, int]], non_empty_space: List[Tuple[int, int]]) -> List[Tuple[float, float, int, int]]:
        items = packing_lib.expand_items(rectangles)

        items = packing_lib.sort_by_height(items)

        levels = []
        placements = []
        current_y = 0

        def intersects_obstacle(x, y, w, h):
            for o in non_empty_space:
                ox1, ox2, oy1, oy2 = o["x_1"], o["x_2"], o["y_1"], o["y_2"]
                if x < ox2 and x + w > ox1 and y < oy2 and y + h > oy1:
                    return True
            return False

        for rect in items:
            w = rect["width"]
            h = rect["height"]

            placed = False

            for level in levels:
                if level["used_width"] + w <= bin_width:
                    x = level["used_width"]
                    y = level["y"]
                    if not intersects_obstacle(x, y, w, h):
                        placements.append([x, y, w, h])
                        level["used_width"] += w
                        placed = True
                        break

            if not placed:
                # Find next y that doesn't collide with obstacles
                y = current_y
                while y < bin_height and intersects_obstacle(0, y, w, h):
                    y += 1
                new_level = {
                    "height": h,
                    "used_width": w,
                    "y": y
                }
                levels.append(new_level)

                placements.append([0, y, w, h])
                current_y = y + h

        total_height = sum(level["height"] for level in levels)

        return packing_lib.make_output(bin_width, total_height, placements)
"#;

impl Default for PackingApp {
    fn default() -> Self {
        Self {
            w_input: String::new(),
            n_input: String::new(),
            k_input: String::new(),
            autofile: false,
            rectangle_data: text_editor::Content::new(),
            rect_total_lines: 1,
            rect_cursor_line: 1,
            error_message: None,
            algo_tabs: vec![AlgoTab {
                id: 0,
                name: "Root".to_string(),
                selected_indices: Vec::new(),
                repacked_indices: Vec::new(),
                obstacle_spaces: Vec::new(),
                selection_regions: Vec::new(),
                code: ROOT_CODE.to_string(),
                last_right_panel_tab: RightPanelTab::Visualization,
                algorithm_output: None,
                parent_output: None,
                repack_output: None,
                output_revision: 0,
                hit_grid: None,
                visible_rects: 0,
                animating: false,
            }],
            active_algo_tab_id: 0,
            next_algo_tab_id: 1,
            zoom: 1.0,
            animation_speed: 80.0,
            pan_x: 0.0,
            pan_y: 0.0,
            is_panning: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            hovered_rect: None,
            dragged_rect: None,
            dragged_rect_offset_x: 0.0,
            dragged_rect_offset_y: 0.0,
            snap_preview: None,
            selected_rects: HashSet::new(),
            active_tab: RightPanelTab::Visualization,
            current_testcase: None,
            testcase_message: None,
            code_editor_content: text_editor::Content::with_text(ROOT_CODE),
            selected_language: CodeLanguage::Python,
            bottom_panel_visible: true,
            bottom_panel_tab: BottomPanelTab::Problems,
            code_errors: Vec::new(),
            code_output_json: None,
            settings: Settings {
                area_select_enabled: true,
                snap_to_rectangles_enabled: true,
                auto_minimize_height: false,
            },
            settings_panel_visible: false,
            area_select_list: Vec::new(),
            new_area_select: true, // true = none created so can create, false = one created so
                                   // can't create
            area_select_start: None,
            area_select_current: None,
            is_area_selecting: false,
            context_menu_visible: false,
            context_menu_region: None,
            context_menu_position: (0.0, 0.0),
            num_test_cases_input: String::new(),
            input_size_input: String::new(),
            unique_types_input: String::new(),
            display_visual: false,
            multiple_test_cases: Vec::new(),
            multiple_testcase_message: None,
            multiple_run_results: Vec::new(),
            multiple_results_expanded: Vec::new(),
            bottom_panel_height: 150.0,
            is_resizing_panel: false,
            panel_drag_last_y: 0.0,
        }
    }
}

fn generate_random_test_case(rng: &mut impl rand::Rng, n: i32, input_size: Option<i32>, unique_types: Option<usize>) -> Vec<JsonInput> {
    let mut ret: Vec<JsonInput> = Vec::new();
    const MIN_HEIGHT: i32 = 1;
    const MAX_HEIGHT: i32 = 20;
    const MIN_TYPES: usize = 3;
    const MAX_TYPES: usize = 20;
    const MIN_TOTAL_RECTS: i32 = 10;
    const DEFAULT_TOTAL_RECTS: i32 = 100;

    for _ in 0..n {
	let target_total = input_size.unwrap_or(DEFAULT_TOTAL_RECTS).max(MIN_TOTAL_RECTS);

	// Scale bin width with sqrt(target_total) so larger inputs get wider bins
	let base = ((target_total as f64).sqrt() * 2.0) as i32;
	let min_bin = base.max(5);
	let max_bin = (base * 3).max(min_bin + 5);
	let bin_width = rng.random_range(min_bin..=max_bin);

	let num_types = unique_types.unwrap_or_else(|| rng.random_range(MIN_TYPES..=MAX_TYPES));

	let mut rect_set: HashSet<(i32, i32)> = HashSet::new();
	let mut rectangles: Vec<Rectangle> = Vec::new();

	while rect_set.len() < num_types {
	    let width = rng.random_range(1..=bin_width);
	    let height = rng.random_range(MIN_HEIGHT..=MAX_HEIGHT);
	    if !rect_set.contains(&(width, height)) {
		rect_set.insert((width, height));
		&rectangles.push(Rectangle { width, height, quantity: 1 });
	    }
	}

	let mut current_total: i32 = rectangles.len() as i32;
	while current_total < target_total {
	    let idx = rng.random_range(0..rectangles.len());
	    let add = rng.random_range(1..=(target_total - current_total).min(5));
	    rectangles[idx].quantity += add;
	    current_total += add;
	}

	let total_rects: i32 = rectangles.iter().map(|r| r.quantity).sum();
	ret.push(JsonInput {
	    width_of_bin: bin_width,
	    number_of_rectangles: total_rects as usize,
	    number_of_types_of_rectangles: rectangles.len(),
	    autofill_option: false,
	    rectangle_list: rectangles,
	});
    }
    ret
}

impl PackingApp {
    pub fn update(&mut self, input: Input) {
        match input {
            Input::WChanged(w_input) => {
                self.w_input = w_input;
            }
            Input::NChanged(n_input) => {
                self.n_input = n_input;
            }
            Input::KChanged(k_input) => {
                self.k_input = k_input;
            }
            Input::NumTestCasesChanged(val) => {
                if val.is_empty() || val.parse::<u32>().is_ok() {
                    self.num_test_cases_input = val;
                }
            }
            Input::InputSizeChanged(val) => {
                if val.is_empty() || val.parse::<u32>().is_ok() {
                    self.input_size_input = val;
                }
            }
            Input::UniqueTypesChanged(val) => {
                if val.is_empty() || val.parse::<u32>().is_ok() {
                    self.unique_types_input = val;
                }
            }
	    Input::DisplayVisual(val) => {
		self.display_visual = val;
	    }
            Input::ToggleMultipleResultExpanded(idx) => {
                if let Some(val) = self.multiple_results_expanded.get_mut(idx) {
                    *val = !*val;
                }
            }
            Input::PanelResizeStart => {
                self.is_resizing_panel = true;
                self.panel_drag_last_y = f32::NAN;
            }
            Input::PanelResizeMove(y) => {
                if self.is_resizing_panel {
                    if !self.panel_drag_last_y.is_nan() {
                        let dy = y - self.panel_drag_last_y;
                        self.bottom_panel_height = (self.bottom_panel_height - dy).clamp(50.0, 600.0);
                    }
                    self.panel_drag_last_y = y;
                }
            }
            Input::PanelResizeEnd => {
                self.is_resizing_panel = false;
            }
            Input::CreateNewTab => {
                self.create_new_tab(None);
            }
            Input::DisplayMultipleResult(idx) => {
                if let Some(result) = self.multiple_run_results.get(idx) {
                    // If already displayed and tab still exists, just switch to it
                    if let Some(existing_tab_id) = result.tab_id {
                        if self.algo_tabs.iter().any(|t| t.id == existing_tab_id) {
                            self.set_active_algo_tab(existing_tab_id);
                            self.active_tab = RightPanelTab::Visualization;
                            return ();
                        }
                    }
                    if let Some(output) = result.output.clone() {
                        let placement_count = output.placements.len();
                        let current_tab_has_output = self.active_algo_tab()
                            .map(|t| t.algorithm_output.is_some())
                            .unwrap_or(false);
                        let assigned_tab_id = if current_tab_has_output {
                            self.create_new_tab(Some(&output))
                        } else {
                            let current_id = self.active_algo_tab_id;
                            if let Some(tab) = self.active_algo_tab_mut() {
                                tab.algorithm_output = Some(output);
                                tab.output_revision = tab.output_revision.wrapping_add(1);
                                tab.visible_rects = placement_count;
                                tab.animating = false;
                            }
                            current_id
                        };
                        self.multiple_run_results[idx].tab_id = Some(assigned_tab_id);
                        self.active_tab = RightPanelTab::Visualization;
                        self.rebuild_hit_grid();
                    }
                }
            }
            Input::AutofillChanged(autofile) => {
                self.autofile = autofile;
            }
            Input::ImportPressed => {
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("Supported files", &["txt", "in", "csv", "json"])
                    .pick_file()
                {
                    match std::fs::read_to_string(&file_path) {
                        Ok(contents) => {
                            // Check if it's a JSON file
                            if file_path.extension().map_or(false, |ext| ext == "json") {
                                match serde_json::from_str::<JsonInput>(&contents) {
                                    Ok(json_input) => {
                                        self.w_input = json_input.width_of_bin.to_string();
                                        self.n_input = json_input.number_of_rectangles.to_string();
                                        self.k_input = json_input.number_of_types_of_rectangles.to_string();
                                        self.autofile = json_input.autofill_option;

                                        // Convert rectangles to text format (X Y Q per line)
                                        let rect_text: String = json_input.rectangle_list
                                            .iter()
                                            .map(|r| format!("{} {} {}", r.width, r.height, r.quantity))
                                            .collect::<Vec<_>>()
                                            .join("\n");

                                        self.rectangle_data = text_editor::Content::with_text(&rect_text);
                                        self.update_rectangle_line_info();
                                        self.error_message = Some(format!(
                                            "Imported JSON: {} rectangle types, bin width {}",
                                            json_input.rectangle_list.len(),
                                            json_input.width_of_bin
                                        ));
                                    }
                                    Err(e) => {
                                        self.error_message = Some(format!("Error parsing JSON: {}", e));
                                    }
                                }
                            } else {
                                self.rectangle_data = text_editor::Content::with_text(&contents);
                                self.update_rectangle_line_info();
                                self.error_message = None;
                            }
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Error reading file: {}", e));
                        }
                    }
                }
            }
            Input::ImportOutputJsonPressed => {
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("JSON files", &["json"])
                    .pick_file()
                {
                    match std::fs::read_to_string(&file_path) {
                        Ok(contents) => {
                            match serde_json::from_str::<AlgorithmOutput>(&contents) {
                                Ok(output) => {
                                    if let Some(tab) = self.active_algo_tab_mut() {
                                        tab.algorithm_output = Some(output);
                                        tab.parent_output = None;
                                        tab.repack_output = None;
                                        tab.repacked_indices.clear();
                                        tab.obstacle_spaces.clear();
                                        tab.visible_rects = 0;
                                        tab.animating = true;
                                        tab.output_revision = tab.output_revision.wrapping_add(1);
                                        self.selected_rects.clear();
                                        self.rebuild_hit_grid();
                                        self.error_message = Some("✓ Successfully imported algorithm output".to_string());
                                    }
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("Error parsing JSON: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Error reading file: {}", e));
                        }
                    }
                }
            }
            Input::RectangleDataAction(action) => {
                self.rectangle_data.perform(action);
                self.update_rectangle_line_info();
            }

            Input::ExportAlgorithmInput => {
                match self.parse_rectangles() {
                    Ok(output) => {
                        let json = match serde_json::to_string_pretty(&create_input(&output)) {
                            Ok(j) => j,
                            Err(e) => {
                                self.error_message = Some(format!("Failed to serialize JSON: {e}"));
                                return;
                            }
                        };

                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON file", &["json"])
                            .set_file_name("algorithm_input.json")
                            .save_file()
                        {
                            match std::fs::write(&path, json) {
                                Ok(_) => {
                                    self.error_message = Some(format!(
                                        "✓ Successfully parsed {} rectangles and saved to {}",
                                        output.rects.len(),
                                        path.display()
                                    ));
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("Failed to save file: {e}"));
                                }
                            }
                        } else {
                            self.error_message = Some("Export cancelled.".to_string());
                        }
                    }
                    Err(errors) => {
                        self.error_message = Some(errors.join("\n"));
                    }
                }
            }
           Input::ZoomChanged(factor) => {
               let new_zoom = (self.zoom * factor).clamp(0.1, 10.0);
               self.zoom = new_zoom;
            }
            Input::AnimationSpeedChanged(speed) => {
                self.animation_speed = speed.clamp(10.0, 500.0);
            }
            Input::PanStart(x, y) => {
                self.is_panning = true;
                self.last_mouse_x = x;
                self.last_mouse_y = y;
                // Hide context menu when starting to pan
                self.context_menu_visible = false;
                self.context_menu_region = None;
            }
            Input::PanMove(x, y) => {
                if self.is_panning {
                    let dx = x - self.last_mouse_x;
                    let dy = y - self.last_mouse_y;
                    self.pan_x += dx;
                    self.pan_y += dy;
                    self.last_mouse_x = x;
                    self.last_mouse_y = y;
                }
                self.hovered_rect = None;
            }
            Input::PanEnd => {
                self.is_panning = false;
            }
            Input::RectangleHovered(rect_idx) => {
                self.hovered_rect = rect_idx;
            }
            Input::RectangleDragStart(rect_idx, x, y) => {
                self.dragged_rect = Some(rect_idx);
                self.last_mouse_x = x;
                self.last_mouse_y = y;
                self.dragged_rect_offset_x = 0.0;
                self.dragged_rect_offset_y = 0.0;
                self.snap_preview = None;
                self.context_menu_visible = false;
                self.context_menu_region = None;
            }
            Input::RectangleDragMove(x, y, scale) => {
                if let Some(dragged_idx) = self.dragged_rect {
                    let dx = x - self.last_mouse_x;
                    let dy = y - self.last_mouse_y;
                    self.dragged_rect_offset_x += dx;
                    self.dragged_rect_offset_y += dy;
                    self.last_mouse_x = x;
                    self.last_mouse_y = y;

                    if self.settings.snap_to_rectangles_enabled {
                        if let Some(output) = self.active_algo_tab().and_then(|t| t.algorithm_output.as_ref()) {
                            if dragged_idx < output.placements.len() {
                                let p = &output.placements[dragged_idx];
                                let bin_x = p.x.into_inner() + (self.dragged_rect_offset_x / scale);
                                let bin_y = p.y.into_inner() - (self.dragged_rect_offset_y / scale);
                                self.snap_preview = self.try_snap_rectangle(dragged_idx, bin_x, bin_y, true, false);
                            } else {
                                self.snap_preview = None;
                            }
                        } else {
                            self.snap_preview = None;
                        }
                    } else {
                        self.snap_preview = None;
                    }
                }
            }
            Input::RectangleDragEnd(is_inside, intersects, new_x, new_y) => {
                let mut height_changed = false;
                if let Some(dragged_idx) = self.dragged_rect
                    && let Some((final_x, final_y)) = self.try_snap_rectangle(dragged_idx, new_x.into_inner(), new_y.into_inner(), is_inside, intersects)
                    && let Some(tab) = self.active_algo_tab_mut()
                    && let Some(output) = &mut tab.algorithm_output
                    && dragged_idx < output.placements.len()
                {
                    output.placements[dragged_idx].x = OrderedFloat(final_x);
                    output.placements[dragged_idx].y = OrderedFloat(final_y);
                    height_changed = self.recalculate_bin_height();
                }

                // If height changed, clear selection regions for current tab
                if height_changed {
                    if let Some(tab) = self.algo_tabs.iter_mut().find(|t| t.id == self.active_algo_tab_id) {
                        tab.selection_regions.clear();
                        self.new_area_select = true;
                    }
                }

                self.dragged_rect = None;
                self.dragged_rect_offset_x = 0.0;
                self.dragged_rect_offset_y = 0.0;
                self.snap_preview = None;
            }
            Input::Tick => {
                if let Some(tab) = self.active_algo_tab_mut() {
                    if let Some(output) = &tab.algorithm_output {
                        let total = output.placements.len();
                        if tab.visible_rects < total {
                            tab.visible_rects += 1;
                        } else {
                            tab.animating = false;
                        }
                    } else {
                        tab.animating = false;
                    }
                }
            }
            Input::SnapAndAdjustHeight => {
                self.recalculate_bin_height();
            }
            Input::RightClickCanvas(clicked_rect) => {
                self.context_menu_visible = false;
                self.context_menu_region = None;
                if let Some(idx) = clicked_rect {
                    if !self.selected_rects.contains(&idx) {
                        self.selected_rects.insert(idx);
                    } else {
                        self.selected_rects.remove(&idx);
                    }
                    if self.active_algo_tab_id != 0 {
                        self.update_active_tab_selection_from_current();
                    }
                }
            }
            Input::LeftClickCanvas() => {
                self.context_menu_visible = false;
                self.context_menu_region = None;
            }
            Input::TabSelected(tab) => {
                self.active_tab = tab;
                if let Some(current_tab) = self.algo_tabs.iter_mut().find(|t| t.id == self.active_algo_tab_id) {
                    current_tab.last_right_panel_tab = tab;
                }
            }
            Input::AlgoTabSelected(tab_id) => {
                self.set_active_algo_tab(tab_id);
            }
            Input::RemoveAlgoTab(tab_id) => {
                if tab_id != 0 {
                    self.algo_tabs.retain(|t| t.id != tab_id);
                    if self.active_algo_tab_id == tab_id {
                        self.set_active_algo_tab(0);
                    }
                }
            }
            Input::CodeEditorAction(action) => {
                use iced::widget::text_editor::{Action, Edit};

                match &action {
                    Action::Edit(Edit::Enter) => {
                        let text = self.code_editor_content.text();
                        let (line, _col) = self.code_editor_content.cursor_position();

                        if let Some(current_line) = text.lines().nth(line) {
                            let leading_ws: String = current_line
                                .chars()
                                .take_while(|c| c.is_whitespace())
                                .collect();

                            let trimmed = current_line.trim_end();
                            let extra_indent = if trimmed.ends_with(':') {
                                "    "
                            } else {
                                ""
                            };

                            self.code_editor_content.perform(action);

                            let indent = format!("{}{}", leading_ws, extra_indent);
                            for c in indent.chars() {
                                self.code_editor_content.perform(Action::Edit(Edit::Insert(c)));
                            }
                        } else {
                            self.code_editor_content.perform(action);
                        }
                    }
                    Action::Edit(Edit::Insert('\t')) => {
                        for _ in 0..4 {
                            self.code_editor_content.perform(Action::Edit(Edit::Insert(' ')));
                        }
                    }
                    Action::Edit(Edit::Backspace) => {
                        let text = self.code_editor_content.text();
                        let (line, col) = self.code_editor_content.cursor_position();

                        let should_delete_tab = if col >= 4 {
                            if let Some(current_line) = text.lines().nth(line) {
                                let chars_before: String = current_line.chars().take(col).collect();
                                chars_before.ends_with("    ")
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if should_delete_tab {
                            for _ in 0..4 {
                                self.code_editor_content.perform(Action::Edit(Edit::Backspace));
                            }
                        } else {
                            self.code_editor_content.perform(action);
                        }
                    }
                    _ => {
                        self.code_editor_content.perform(action);
                    }
                }
            }
            Input::LanguageSelected(lang) => {
                self.selected_language = lang;
            }
            Input::RunCode(which_tab) => {
                if which_tab == 1 {
                    let is_root = self.active_algo_tab_id == 0;
                    if is_root {
                        if let Some(testcase) = self.current_testcase.clone() {
                            self.run_with_testcase(&testcase);
                        } else {
                            self.error_message = Some("No test case loaded. Import a test case first.".to_string());
                            self.bottom_panel_tab = BottomPanelTab::TestCases;
                        }
                    } else {
			let tab_data = self.algo_tabs.iter().find(|t| t.id == self.active_algo_tab_id).cloned();
			if let Some(tab) = tab_data {
			    if let Some(inherited_region) = tab.selection_regions.iter().find(|r| r.is_inherited) {
				if let Some(output) = tab.parent_output.as_ref().or(tab.algorithm_output.as_ref()) {
				    let (selected_indices, non_selected_indices) = self.split_region_rectangles(output, inherited_region);
				    eprintln!(
					"Repack region: x={} y={} w={} h={} selected={} non_selected={}",
					inherited_region.bin_x,
					inherited_region.bin_y,
					inherited_region.bin_w,
					inherited_region.bin_h,
					selected_indices.len(),
					non_selected_indices.len()
				    );
				    eprintln!("Selected indices: {:?}", selected_indices);
				    eprintln!("Non-selected indices: {:?}", non_selected_indices);

				    let mut rectangles: Vec<Rectangle> = Vec::new();
				    for &idx in &selected_indices {
					let p = &output.placements[idx];
					rectangles.push(Rectangle { width: p.width, height: p.height, quantity: 1 });
					eprintln!(
					    "Selected rect idx {} -> x={} y={} w={} h={}",
					    idx,
					    p.x.into_inner(),
					    p.y.into_inner(),
					    p.width,
					    p.height
					);
				    }

				    let mut non_empty_space: Vec<NonEmptySpace> = Vec::new();
				    let mut obstacle_spaces: Vec<NonEmptySpace> = Vec::new();
				    for &idx in &non_selected_indices {
					let p = &output.placements[idx];
					eprintln!(
					    "Obstacle rect idx {} -> x={} y={} w={} h={}",
					    idx,
					    p.x.into_inner(),
					    p.y.into_inner(),
					    p.width,
					    p.height
					);
					let rect_x1 = p.x.into_inner();
					let rect_y1 = p.y.into_inner();
					let rect_x2 = rect_x1 + p.width as f32;
					let rect_y2 = rect_y1 + p.height as f32;

					let region_x1 = inherited_region.bin_x;
					let region_y1 = inherited_region.bin_y;
					let region_x2 = inherited_region.bin_x + inherited_region.bin_w;
					let region_y2 = inherited_region.bin_y + inherited_region.bin_h;

					let inter_x1 = rect_x1.max(region_x1);
					let inter_y1 = rect_y1.max(region_y1);
					let inter_x2 = rect_x2.min(region_x2);
					let inter_y2 = rect_y2.min(region_y2);

					if inter_x2 > inter_x1 && inter_y2 > inter_y1 {
					    let space = NonEmptySpace {
						x_1: inter_x1 - region_x1,
						y_1: inter_y1 - region_y1,
						x_2: inter_x2 - region_x1,
						y_2: inter_y2 - region_y1,
					    };
					    eprintln!("Obstacle region-local idx {} -> {:?}", idx, space);
					    non_empty_space.push(space);
					    obstacle_spaces.push(NonEmptySpace {
						x_1: inter_x1,
						y_1: inter_y1,
						x_2: inter_x2,
						y_2: inter_y2,
					    });
					}
				    }

				    if !rectangles.is_empty() {
					let user_code = self.code_editor_content.text();
					let temp_testcase = JsonInput {
					    width_of_bin: inherited_region.bin_w.max(0.0) as i32,
					    number_of_rectangles: rectangles.len(),
					    number_of_types_of_rectangles: rectangles.len(),
					    autofill_option: false,
					    rectangle_list: rectangles,
					};

					let result = run_repack_code_with_testcase(
					    self.selected_language,
					    &user_code,
					    &temp_testcase,
					    inherited_region.bin_h.max(0.0),
					    &non_empty_space,
					);

					match result {
					    RunResult::Success { output: new_output, raw_json } => {
						eprintln!(
						    "Repack output: bin_width={} total_height={} placements={}",
						    new_output.bin_width,
						    new_output.total_height,
						    new_output.placements.len()
						);
						for (i, p) in new_output.placements.iter().enumerate() {
						    eprintln!(
							"Repacked rect {} -> x={} y={} w={} h={}",
							i,
							p.x.into_inner(),
							p.y.into_inner(),
							p.width,
							p.height
						    );
						}
						let parent_snapshot = self
						    .active_algo_tab()
						    .and_then(|tab| tab.parent_output.clone().or_else(|| tab.algorithm_output.clone()));
						let composed_output = parent_snapshot
						    .as_ref()
							.map(|parent| self.compose_repack_output(parent, &new_output, &selected_indices, inherited_region))
							.unwrap_or_else(|| new_output.clone());

						if let Some(tab) = self.active_algo_tab_mut() {
						    tab.repack_output = Some(new_output);
						    tab.algorithm_output = Some(composed_output);
						    tab.repacked_indices = selected_indices.clone();
						    tab.obstacle_spaces = obstacle_spaces.clone();
						    tab.visible_rects = 0;
						    tab.animating = true;
						    tab.output_revision = tab.output_revision.wrapping_add(1);
						}
						if self.settings.auto_minimize_height {
						    self.apply_auto_minimize_height();
						}
						self.selected_rects.clear();
						self.rebuild_hit_grid();
						self.active_tab = RightPanelTab::Visualization;
						if let Some(tab) = self.algo_tabs.iter_mut().find(|t| t.id == self.active_algo_tab_id) {
						    tab.last_right_panel_tab = self.active_tab;
						}
						self.error_message = Some("✓ Code executed successfully".to_string());
						self.code_output_json = Some(raw_json);
						self.code_errors.clear();
						self.bottom_panel_tab = BottomPanelTab::Output;
						self.new_area_select = true;
					    }
					    RunResult::Error { errors } => {
						self.error_message = Some(format!("Execution error:\n{}", errors.join("\n")));
						self.code_errors = errors;
						self.code_output_json = None;
						self.bottom_panel_tab = BottomPanelTab::Problems;
					    }
					}
				    } else {
					self.error_message = Some("Select at least one rectangle to repack.".to_string());
				    }
				}
			    } else {
				self.error_message = Some("No inherited region found.".to_string());
			    }
			}
		    }
                } else {
                    let test_cases = self.multiple_test_cases.clone();
                    let user_code = self.code_editor_content.text();
                    let language = self.selected_language;
                    let results: Vec<MultipleRunResult> = test_cases.iter().map(|testcase| {
                        let run_result = run_code_with_testcase(language, &user_code, testcase);
                        let (height, output) = match run_result {
                            RunResult::Success { output, .. } => (Some(output.total_height), Some(output)),
                            RunResult::Error { .. } => (None, None),
                        };
                        MultipleRunResult { testcase: testcase.clone(), height, output, tab_id: None }
                    }).collect();
                    let n = results.len();
                    self.multiple_results_expanded = vec![false; n];
                    self.multiple_run_results = results;
                    self.bottom_panel_tab = BottomPanelTab::Output;
                }
            }
            Input::BottomPanelTabSelected(tab) => {
                self.bottom_panel_tab = tab;
            }
            Input::ToggleBottomPanel => {
                self.bottom_panel_visible = !self.bottom_panel_visible;
            }
            Input::SaveOutputToFile => {
                if let Some(json) = &self.code_output_json && let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON file", &["json"])
                    .set_file_name("algorithm_output.json")
                    .save_file()
                {
                    match std::fs::write(&path, json) {
                        Ok(_) => {
                            self.error_message = Some(format!("✓ Output saved to {}", path.display()));
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to save file: {}", e));
                        }
                    }
                }
            
            }
            Input::InsertTab => {
                use iced::widget::text_editor::{Action, Edit};
                for _ in 0..4 {
                    self.code_editor_content.perform(Action::Edit(Edit::Insert(' ')));
                }
            }
            Input::ImportTestCase => {
                if let Some(file_path) = rfd::FileDialog::new().add_filter("JSON files", &["json"]).pick_file() {
                    match std::fs::read_to_string(&file_path) {
                        Ok(contents) => {
                            match serde_json::from_str::<JsonInput>(&contents) {
                                Ok(output) => {
                                    let msg = format!("✓ Loaded: {} rectangles, bin width {}",
                                        output.rectangle_list.len(),
                                        output.width_of_bin);
                                    self.current_testcase = Some(output);
                                    self.testcase_message = Some(msg);
                                }
                                Err(e) => {
                                    self.testcase_message = Some(format!("Error parsing JSON: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            self.testcase_message = Some(format!("Error reading file: {}", e));
                        }
                    }
                }
            }
            Input::GenerateTestCase => {
                let mut rng = rand::rng();
                let input_size = self.input_size_input.parse::<i32>().ok();
                let unique_types = self.unique_types_input.parse::<usize>().ok();
                let testcase = generate_random_test_case(&mut rng, 1, input_size, unique_types);
                let msg = format!(
                    "Generated: {} rectangles, {} types, bin width {}",
                    testcase[0].number_of_rectangles,
                    testcase[0].number_of_types_of_rectangles,
                    testcase[0].width_of_bin
                );
                self.current_testcase = Some(testcase[0].clone());
                self.testcase_message = Some(msg);
            }
            Input::GenerateMultipleTestCases(n) => {
                let mut rng = rand::rng();
                let input_size = self.input_size_input.parse::<i32>().ok();
                let unique_types = self.unique_types_input.parse::<usize>().ok();
                self.multiple_test_cases = generate_random_test_case(&mut rng, n, input_size, unique_types);
                let msg = format!("Generated: {} test cases", self.multiple_test_cases.len());
                self.multiple_testcase_message = Some(msg);
            }
            Input::ToggleAreaSelectEnabled(enabled) => {
                self.settings.area_select_enabled = enabled;
            }
            Input::ToggleSnapToRectangles(enabled) => {
                self.settings.snap_to_rectangles_enabled = enabled;
            }
            Input::ToggleAutoMinimizeHeight(enabled) => {
                self.settings.auto_minimize_height = enabled;
                if enabled {
                    self.apply_auto_minimize_height();
                }
            }
            Input::ToggleSettingsPanel => {
                self.settings_panel_visible = !self.settings_panel_visible;
            }
            Input::AreaSelectStart(x, y) => {
                self.is_area_selecting = true;
                self.area_select_start = Some((x, y));
                self.area_select_current = Some((x, y));
                self.context_menu_visible = false;
                self.context_menu_region = None;
            }
            Input::AreaSelectMove(x, y) => {
                if self.is_area_selecting {
                    self.area_select_current = Some((x, y));
                }
            }
            Input::AreaSelectEnd(_selected_indices, bin_x, bin_y, bin_w, bin_h) => {
                let current_tab_idx = self.algo_tabs.iter().position(|t| t.id == self.active_algo_tab_id);

                if let Some(tab_idx) = current_tab_idx {
                    let is_root = self.active_algo_tab_id == 0;
                    let max_regions = if is_root { 1 } else { 2 };

                    let current_new_regions = self.algo_tabs[tab_idx].selection_regions.iter()
                        .filter(|r| !r.is_inherited)
                        .count();

                    let can_create = self.new_area_select && bin_w > 0.0 && bin_h > 0.0 && current_new_regions < max_regions;

                    if can_create {
                        let mut final_x = bin_x;
                        let mut final_y = bin_y;
                        let mut valid = true;

                        for existing in &self.algo_tabs[tab_idx].selection_regions {
                            let new_x1 = final_x;
                            let new_y1 = final_y;
                            let new_x2 = final_x + bin_w;
                            let new_y2 = final_y + bin_h;

                            let ex1 = existing.bin_x;
                            let ey1 = existing.bin_y;
                            let ex2 = existing.bin_x + existing.bin_w;
                            let ey2 = existing.bin_y + existing.bin_h;

                            if new_x1 < ex2 && new_x2 > ex1 && new_y1 < ey2 && new_y2 > ey1 {
                                let overlap_x = (new_x2.min(ex2) - new_x1.max(ex1)).max(0.0);
                                let overlap_y = (new_y2.min(ey2) - new_y1.max(ey1)).max(0.0);

                                let overlap_x_pct = overlap_x / bin_w;
                                let overlap_y_pct = overlap_y / bin_h;

                                if overlap_x_pct <= 0.2 || overlap_y_pct <= 0.2 {
                                    let push_left = new_x2 - ex1;
                                    let push_right = ex2 - new_x1;
                                    let push_down = new_y2 - ey1;
                                    let push_up = ey2 - new_y1;

                                    let mut min_push = f32::MAX;
                                    let mut push_dir = 0;

                                    if push_left > 0.0 && push_left < min_push {
                                        min_push = push_left;
                                        push_dir = 0;
                                    }
                                    if push_right > 0.0 && push_right < min_push {
                                        min_push = push_right;
                                        push_dir = 1;
                                    }
                                    if push_down > 0.0 && push_down < min_push {
                                        min_push = push_down;
                                        push_dir = 2;
                                    }
                                    if push_up > 0.0 && push_up < min_push {
                                        push_dir = 3;
                                    }

                                    match push_dir {
                                        0 => final_x = ex1 - bin_w,
                                        1 => final_x = ex2,
                                        2 => final_y = ey1 - bin_h,
                                        3 => final_y = ey2,
                                        _ => {}
                                    }
                                } else {
                                    valid = false;
                                    break;
                                }
                            }
                        }

                        if valid {
                            let final_x1 = final_x;
                            let final_y1 = final_y;
                            let final_x2 = final_x + bin_w;
                            let final_y2 = final_y + bin_h;

                            for existing in &self.algo_tabs[tab_idx].selection_regions {
                                let ex1 = existing.bin_x;
                                let ey1 = existing.bin_y;
                                let ex2 = existing.bin_x + existing.bin_w;
                                let ey2 = existing.bin_y + existing.bin_h;

                                if final_x1 < ex2 && final_x2 > ex1 && final_y1 < ey2 && final_y2 > ey1 {
                                    valid = false;
                                    break;
                                }
                            }
                        }

                        if valid {
                            let region = SelectionRegion {
                                is_inherited: false,
                                bin_x: final_x,
                                bin_y: final_y,
                                bin_w,
                                bin_h,
                                selected_indices: Vec::new(),
                            };
                            self.algo_tabs[tab_idx].selection_regions.push(region);
                        }
                    }
                }
                self.new_area_select = false;
                self.is_area_selecting = false;
                self.area_select_start = None;
                self.area_select_current = None;
            }
            Input::ShowRegionContextMenu(region_idx, x, y) => {
                self.context_menu_visible = true;
                self.context_menu_region = Some(region_idx);
                self.context_menu_position = (x, y);
            }
            Input::HideContextMenu => {
                self.context_menu_visible = false;
                self.context_menu_region = None;
            }
            Input::RemoveSelectionRegion(region_idx) => {
                if let Some(tab) = self.algo_tabs.iter_mut().find(|t| t.id == self.active_algo_tab_id) && region_idx < tab.selection_regions.len() {
                    tab.selection_regions.remove(region_idx);
                
                }
                self.new_area_select = true;
                self.context_menu_visible = false;
                self.context_menu_region = None;
            }
            Input::RepackSelectionRegion(region_idx) => {
                let current_tab_idx = self.algo_tabs.iter().position(|t| t.id == self.active_algo_tab_id);

                if let Some(tab_idx) = current_tab_idx && region_idx < self.algo_tabs[tab_idx].selection_regions.len() {
                    let region = self.algo_tabs[tab_idx].selection_regions[region_idx].clone();

                    if region.is_inherited {
                        self.context_menu_visible = false;
                        self.context_menu_region = None;
                        return;
                    }

                    let current_tab_name = &self.algo_tabs[tab_idx].name;
                    let is_root = current_tab_name == "Root";

                    let prefix = if is_root {
                        "Node ".to_string()
                    } else {
                        format!("{}-", current_tab_name)
                    };

                    let child_count = self.algo_tabs.iter()
                        .filter(|t| {
                            if is_root {
                                t.name.starts_with("Node ") && !t.name.contains('-')
                            } else {
                                t.name.starts_with(&prefix) &&
                                t.name[prefix.len()..].chars().all(|c| c.is_ascii_digit())
                            }
                        })
                        .count();

                    let new_name = format!("{}{}", prefix, child_count + 1);
                        let new_code: String = NODE_CODE.to_string();

                        self.algo_tabs[tab_idx].code = new_code.clone(); 

                        let tab_id = self.next_algo_tab_id;
                        self.next_algo_tab_id = self.next_algo_tab_id.wrapping_add(1);

                        let inherited_region = SelectionRegion {
                            is_inherited: true,
                            bin_x: region.bin_x,
                            bin_y: region.bin_y,
                            bin_w: region.bin_w,
                            bin_h: region.bin_h,
                            selected_indices: Vec::new(),
                        };

                        // New tab inherits parent's code and visualization
                        let (parent_output, parent_revision, parent_visible, parent_animating) = if let Some(parent_tab) = self.algo_tabs.iter().find(|t| t.id == self.active_algo_tab_id) {
                            (parent_tab.algorithm_output.clone(), parent_tab.output_revision, parent_tab.visible_rects, parent_tab.animating)
                        } else {
                            (None, 0, 0, false)
                        };

                        self.algo_tabs.push(AlgoTab {
                            id: tab_id,
                            name: new_name,
                            selected_indices: Vec::new(),
                            repacked_indices: Vec::new(),
                            obstacle_spaces: Vec::new(),
                            selection_regions: vec![inherited_region],
                            code: new_code,
                            last_right_panel_tab: RightPanelTab::Visualization,
                            algorithm_output: parent_output.clone(),
                            parent_output,
                            repack_output: None,
                            output_revision: parent_revision,
                            hit_grid: None,
                            visible_rects: parent_visible,
                            animating: parent_animating,
                        });

                    self.algo_tabs[tab_idx].selection_regions.remove(region_idx);

                    self.set_active_algo_tab(tab_id);
                
                }
                self.new_area_select = true;
                self.context_menu_visible = false;
                self.context_menu_region = None;
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Input> {
        use iced::keyboard;

        let mut subscriptions = vec![];

        if self.active_algo_tab().map(|t| t.animating).unwrap_or(false) {
            subscriptions.push(
                time::every(Duration::from_millis(self.animation_speed as u64)).map(|_| Input::Tick)
            );
        }

        if self.active_tab == RightPanelTab::CodeEditor {
            subscriptions.push(
                keyboard::on_key_press(|key, modifiers| {
                    if key == keyboard::Key::Named(keyboard::key::Named::Tab) && modifiers.is_empty() {
                        Some(Input::InsertTab)
                    } else {
                        None
                    }
                })
            );
        }

        if self.is_resizing_panel {
            subscriptions.push(
                iced::event::listen_with(|event, _status, _id| {
                    match event {
                        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(_))
                        | iced::Event::Mouse(iced::mouse::Event::ButtonPressed(_)) => {
                            Some(Input::PanelResizeEnd)
                        }
                        _ => None,
                    }
                })
            );
        }

        Subscription::batch(subscriptions)
    }


fn try_snap_rectangle(
    &self,
    rect_idx: usize,
    new_x: f32,
    new_y: f32,
    _is_inside: bool,
    _intersects: bool,
) -> Option<(f32, f32)> {
    const SNAP_MARGIN_PERCENTAGE: f32 = 0.05;

    const EDGE_SNAP_THRESHOLD: f32 = 10.0; 
    const VERTICAL_SNAP_THRESHOLD: f32 = 12.0;

    let output = self.active_algo_tab().and_then(|t| t.algorithm_output.as_ref())?;
    if rect_idx >= output.placements.len() {
        return None;
    }

    let p = &output.placements[rect_idx];
    let rect_w = p.width as f32;
    let rect_h = p.height as f32;

    let bin_w = output.bin_width as f32;
    let bin_h = output.total_height;

    let snap_margin = rect_w.min(rect_h) * SNAP_MARGIN_PERCENTAGE;
const OUT_OF_BOUNDS_TOLERANCE: f32 = 0.10;

if !self.settings.snap_to_rectangles_enabled {
    let tol_x = rect_w * OUT_OF_BOUNDS_TOLERANCE;
    let tol_y = rect_h * OUT_OF_BOUNDS_TOLERANCE;

    let dx_neg = (0.0 - new_x).max(0.0);
    let dx_pos = ((new_x + rect_w) - bin_w).max(0.0);

    let dy_neg = (0.0 - new_y).max(0.0);
    let dy_pos = ((new_y + rect_h) - bin_h).max(0.0);

    if dx_neg > tol_x || dx_pos > tol_x || dy_neg > tol_y || dy_pos > tol_y {
        return None;
    }

    let x = new_x.clamp(0.0, bin_w - rect_w);
    let y = new_y.clamp(0.0, bin_h - rect_h);

    let intersects_any = |x: f32, y: f32| -> bool {
        for (i, q) in output.placements.iter().enumerate() {
            if i == rect_idx {
                continue;
            }
            let ox = q.x.into_inner();
            let oy = q.y.into_inner();
            let ow = q.width as f32;
            let oh = q.height as f32;

            let overlap_x = x < ox + ow && x + rect_w > ox;
            let overlap_y = y < oy + oh && y + rect_h > oy;

            if overlap_x && overlap_y {
                return true;
            }
        }
        false
    };

    if intersects_any(x, y) {
        return None;
    }

    return Some((x, y));
}


    let (sx, sy) = self.snap_to_rectangles(
        rect_idx,
        new_x,
        new_y,
        rect_w,
        rect_h,
        EDGE_SNAP_THRESHOLD.max(VERTICAL_SNAP_THRESHOLD) + snap_margin,
    );

    Some((sx, sy))
}



fn snap_to_rectangles(
    &self,
    rect_idx: usize,
    new_x: f32,
    new_y: f32,
    rect_w: f32,
    rect_h: f32,
    threshold: f32,
) -> (f32, f32) {
    const HEIGHT_EPS: f32 = 1e-3;

    let output = match self.active_algo_tab().and_then(|t| t.algorithm_output.as_ref()) {
        Some(o) => o,
        None => return (new_x, new_y),
    };

    let bin_w = output.bin_width as f32;
    let bin_h = output.total_height;

    let other_max_top = output
        .placements
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != rect_idx)
        .map(|(_, q)| q.y.into_inner() + q.height as f32)
        .fold(0.0_f32, f32::max);

    let intersects_any = |x: f32, y: f32| -> bool {
        for (i, q) in output.placements.iter().enumerate() {
            if i == rect_idx {
                continue;
            }
            let ox = q.x.into_inner();
            let oy = q.y.into_inner();
            let ow = q.width as f32;
            let oh = q.height as f32;

            let overlap_x = x < ox + ow && x + rect_w > ox;
            let overlap_y = y < oy + oh && y + rect_h > oy;

            if overlap_x && overlap_y {
                return true;
            }
        }
        false
    };

    let is_valid = |x: f32, y: f32| -> bool {
        if x < 0.0 || x + rect_w > bin_w {
            return false;
        }
        if y < 0.0 || y + rect_h > bin_h {
            return false;
        }
        !intersects_any(x, y)
    };

    let drop_to_support = |x: f32, start_y: f32| -> f32 {
        let mut support_y = 0.0_f32; // floor
        for (i, q) in output.placements.iter().enumerate() {
            if i == rect_idx {
                continue;
            }
            let ox = q.x.into_inner();
            let oy = q.y.into_inner();
            let ow = q.width as f32;
            let oh = q.height as f32;

            let overlap_x = x < ox + ow && x + rect_w > ox;
            if !overlap_x {
                continue;
            }

            let top = oy + oh;
            if top <= start_y + 1e-3 {
                support_y = support_y.max(top);
            }
        }
        support_y
    };

    let nearest_side_gap = |x: f32, y: f32| -> (f32, f32, Option<f32>, Option<f32>) {
        let y0 = y;
        let y1 = y + rect_h;

        let mut best_left_gap = x;
        let mut best_left_snap_x = Some(0.0);

        let mut best_right_gap = bin_w - (x + rect_w);
        let mut best_right_snap_x = Some(bin_w - rect_w);

        for (i, q) in output.placements.iter().enumerate() {
            if i == rect_idx {
                continue;
            }
            let ox = q.x.into_inner();
            let oy = q.y.into_inner();
            let ow = q.width as f32;
            let oh = q.height as f32;

            let overlap_y = y0 < oy + oh && y1 > oy;
            if !overlap_y {
                continue;
            }

            let other_left = ox;
            let other_right = ox + ow;

            if other_right <= x + 1e-3 {
                let gap = x - other_right;
                if gap < best_left_gap {
                    best_left_gap = gap;
                    best_left_snap_x = Some(other_right);
                }
            }

            if other_left >= (x + rect_w) - 1e-3 {
                let gap = other_left - (x + rect_w);
                if gap < best_right_gap {
                    best_right_gap = gap;
                    best_right_snap_x = Some(other_left - rect_w);
                }
            }
        }

        (best_left_gap, best_right_gap, best_left_snap_x, best_right_snap_x)
    };

    let mut x_candidates: Vec<f32> = vec![new_x, 0.0, bin_w - rect_w];

    for (i, q) in output.placements.iter().enumerate() {
        if i == rect_idx {
            continue;
        }
        let ox = q.x.into_inner();
        let oy = q.y.into_inner();
        let ow = q.width as f32;
        let oh = q.height as f32;

        let candidates = [
            ox,          
            ox + ow - rect_w,
            ox + ow,        
            ox - rect_w,   
        ];

        for &xc in &candidates {
            if (xc - new_x).abs() <= threshold {
                x_candidates.push(xc);
            }
        }

        let _ = (oy, oh);
    }

    for xc in &mut x_candidates {
        *xc = xc.clamp(0.0, bin_w - rect_w);
    }
    x_candidates.sort_by(|a, b| a.partial_cmp(b).unwrap());
    x_candidates.dedup_by(|a, b| (*a - *b).abs() < 1e-3);

    let mut y_starts: Vec<f32> = vec![new_y, 0.0];


    for (i, q) in output.placements.iter().enumerate() {
        if i == rect_idx {
            continue;
        }
        let other_bottom = q.y.into_inner();
        let other_top = other_bottom + q.height as f32;

        let candidates = [
            other_top,                 
            other_bottom - rect_h,    
            other_bottom,            
            other_top - rect_h,     
        ];

        for &ys in &candidates {
            if (ys - new_y).abs() <= threshold {
                y_starts.push(ys);
            }
        }
    }

    for ys in &mut y_starts {
        *ys = ys.clamp(0.0, bin_h - rect_h);
    }
    y_starts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    y_starts.dedup_by(|a, b| (*a - *b).abs() < 1e-3);

    let mut best_height = f32::INFINITY;
    let mut best_bucket: Vec<(f32, f32)> = Vec::new();

    for &xc in &x_candidates {
        for &ys in &y_starts {
            let dropped = drop_to_support(xc, ys);
            if !is_valid(xc, dropped) {
                continue;
            }

            let resulting_height = other_max_top.max(dropped + rect_h);
            if resulting_height + HEIGHT_EPS < best_height {
                best_height = resulting_height;
                best_bucket.clear();
                best_bucket.push((xc, dropped));
            } else if (resulting_height - best_height).abs() <= HEIGHT_EPS {
                best_bucket.push((xc, dropped));
            }
        }

        let dropped = drop_to_support(xc, new_y.clamp(0.0, bin_h - rect_h));
        if is_valid(xc, dropped) {
            let resulting_height = other_max_top.max(dropped + rect_h);
            if resulting_height + HEIGHT_EPS < best_height {
                best_height = resulting_height;
                best_bucket.clear();
                best_bucket.push((xc, dropped));
            } else if (resulting_height - best_height).abs() <= HEIGHT_EPS {
                best_bucket.push((xc, dropped));
            }
        }
    }

    if best_bucket.is_empty() {
        let xc = new_x.clamp(0.0, bin_w - rect_w);
        let dropped = drop_to_support(xc, new_y.clamp(0.0, bin_h - rect_h));
        if is_valid(xc, dropped) {
            return (xc, dropped);
        }
        return (new_x, new_y);
    }

    let mut best = best_bucket[0];
    let mut best_score = (f32::INFINITY, f32::INFINITY); 

    for &(xc, yc) in &best_bucket {
        let (left_gap, right_gap, left_snap_x, right_snap_x) = nearest_side_gap(xc, yc);
        let (_, preferred_snap_x) = if left_gap <= right_gap {
            (left_gap, left_snap_x)
        } else {
            (right_gap, right_snap_x)
        };

        let mut final_x = xc;
        if let Some(snap_x) = preferred_snap_x {
            let sx = snap_x.clamp(0.0, bin_w - rect_w);
            if is_valid(sx, yc) {
                final_x = sx;
            }
        }

        let (lg2, rg2, _, _) = nearest_side_gap(final_x, yc);
        let min_gap2 = lg2.min(rg2);

        let dx = final_x - new_x;
        let dy = yc - new_y;
        let dist2 = dx * dx + dy * dy;

        let score = (min_gap2, dist2);
        if score < best_score {
            best_score = score;
            best = (final_x, yc);
        }
    }
    println!("{:?}", best);

    best
}

    /// Recalculates bin height and returns true if height changed
    fn recalculate_bin_height(&mut self) -> bool {
        let height_changed = if let Some(tab) = self.active_algo_tab_mut() {
            if let Some(output) = &mut tab.algorithm_output {
                let old_height = output.total_height;
                let mut max_height = OrderedFloat(0.0);
                for placement in &output.placements {
                    let top = placement.y + placement.height as f32;
                    if top > max_height {
                        max_height = top;
                    }
                }
                output.total_height = max_height.into_inner();
                (output.total_height - old_height).abs() > 0.001
            } else {
                false
            }
        } else {
            false
        };
        if let Some(tab) = self.active_algo_tab_mut() {
            tab.output_revision = tab.output_revision.wrapping_add(1);
        }
        self.rebuild_hit_grid();
        height_changed
    }

    fn update_rectangle_line_info(&mut self) {
        let text = self.rectangle_data.text();
        let total_lines = if text.is_empty() || text == "\n" {
            1
        } else {
            text.chars().filter(|&c| c == '\n').count() + 1
        };
        let (line, _col) = self.rectangle_data.cursor_position();
        let cursor_line = (line + 1).min(total_lines);

        self.rect_total_lines = total_lines;
        self.rect_cursor_line = cursor_line;
    }

    fn rebuild_hit_grid(&mut self) {
        use crate::types::HitGrid;

        let Some(tab) = self.active_algo_tab_mut() else {
            return;
        };
        let Some(output) = &tab.algorithm_output else {
            tab.hit_grid = None;
            return;
        };

        let bin_w = output.bin_width.max(1) as f32;
        let bin_h = output.total_height.max(1.0);
        let cell_size = (bin_w.max(bin_h) / 40.0).clamp(5.0, 50.0);
        let cols = (bin_w / cell_size).ceil().max(1.0) as usize;
        let rows = (bin_h / cell_size).ceil().max(1.0) as usize;
        let mut cells: Vec<Vec<usize>> = vec![Vec::new(); cols * rows];

        for (idx, p) in output.placements.iter().enumerate() {
            let x0 = p.x.into_inner().max(0.0);
            let y0 = p.y.into_inner().max(0.0);
            let x1 = (p.x.into_inner() + p.width as f32).min(bin_w);
            let y1 = (p.y.into_inner() + p.height as f32).min(bin_h);

            let min_col = (x0 / cell_size).floor() as usize;
            let max_col = (x1 / cell_size).floor().min((cols - 1) as f32) as usize;
            let min_row = (y0 / cell_size).floor() as usize;
            let max_row = (y1 / cell_size).floor().min((rows - 1) as f32) as usize;

            for row in min_row..=max_row {
                let row_offset = row * cols;
                for col in min_col..=max_col {
                    cells[row_offset + col].push(idx);
                }
            }
        }

        tab.hit_grid = Some(HitGrid {
            cell_size,
            cols,
            rows,
            cells,
        });
    }

    fn run_with_testcase(&mut self, testcase: &JsonInput) {
        let user_code = self.code_editor_content.text();
        let result = run_code_with_testcase(self.selected_language, &user_code, testcase);
        match result {
            RunResult::Success { output, raw_json } => {
                if let Some(tab) = self.active_algo_tab_mut() {
                    tab.algorithm_output = Some(output);
                    tab.parent_output = None;
                    tab.repack_output = None;
                    tab.repacked_indices.clear();
                    tab.obstacle_spaces.clear();
                    tab.visible_rects = 0;
                    tab.animating = true;
                    tab.output_revision = tab.output_revision.wrapping_add(1);
                }
                if self.settings.auto_minimize_height {
                    self.apply_auto_minimize_height();
                }
                self.selected_rects.clear();
                self.rebuild_hit_grid();
                self.active_tab = RightPanelTab::Visualization;
                if let Some(tab) = self.algo_tabs.iter_mut().find(|t| t.id == self.active_algo_tab_id) {
                    tab.last_right_panel_tab = self.active_tab;
                }
                self.error_message = Some("✓ Code executed successfully".to_string());
                self.code_output_json = Some(raw_json);
                self.code_errors.clear();
                self.bottom_panel_tab = BottomPanelTab::Output;
                self.new_area_select = true;
                if let Some(tab) = self.algo_tabs.iter_mut().find(|t| t.id == self.active_algo_tab_id) {
                    tab.selection_regions.clear();
                }
            }
            RunResult::Error { errors } => {
                self.error_message = Some(format!("Execution error:\n{}", errors.join("\n")));
                self.code_errors = errors;
                self.code_output_json = None;
                self.bottom_panel_tab = BottomPanelTab::Problems;
            }
        }
    }

    fn active_algo_tab(&self) -> Option<&AlgoTab> {
        self.algo_tabs.iter().find(|t| t.id == self.active_algo_tab_id)
    }

    fn active_algo_tab_mut(&mut self) -> Option<&mut AlgoTab> {
        self.algo_tabs.iter_mut().find(|t| t.id == self.active_algo_tab_id)
    }

    fn compose_repack_output(
        &self,
        parent_output: &AlgorithmOutput,
        repack_output: &AlgorithmOutput,
        selected_indices: &[usize],
        region: &SelectionRegion,
    ) -> AlgorithmOutput {
        let mut placements = parent_output.placements.clone();
        let offset_x = region.bin_x;
        let offset_y = region.bin_y;
        if repack_output.placements.len() == selected_indices.len() {
            for (src_idx, &target_idx) in selected_indices.iter().enumerate() {
                if target_idx < placements.len() {
                    let mut placement = repack_output.placements[src_idx].clone();
                    placement.x = OrderedFloat(placement.x.into_inner() + offset_x);
                    placement.y = OrderedFloat(placement.y.into_inner() + offset_y);
                    let max_x = (offset_x + region.bin_w) - placement.width as f32;
                    let max_y = (offset_y + region.bin_h) - placement.height as f32;
                    let clamped_x = placement.x.into_inner().clamp(offset_x, max_x.max(offset_x));
                    let clamped_y = placement.y.into_inner().clamp(offset_y, max_y.max(offset_y));
                    placement.x = OrderedFloat(clamped_x);
                    placement.y = OrderedFloat(clamped_y);
                    placements[target_idx] = placement;
                }
            }
        } else {
            return repack_output.clone();
        }

        if self.settings.auto_minimize_height {
            Self::gravity_collapse(parent_output.bin_width as f32, &mut placements);
        }

        let mut max_height = 0.0_f32;
        for placement in &placements {
            let top = placement.y.into_inner() + placement.height as f32;
            if top > max_height {
                max_height = top;
            }
        }

        AlgorithmOutput {
            bin_width: parent_output.bin_width,
            total_height: max_height,
            placements,
        }
    }

    fn gravity_collapse(bin_width: f32, placements: &mut [Placement]) {
        let mut order: Vec<usize> = (0..placements.len()).collect();
        order.sort_by(|&a, &b| {
            let ay = placements[a].y.into_inner();
            let by = placements[b].y.into_inner();
            ay.partial_cmp(&by)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    let ax = placements[a].x.into_inner();
                    let bx = placements[b].x.into_inner();
                    ax.partial_cmp(&bx).unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        let mut placed = vec![false; placements.len()];
        for &idx in &order {
            let rect_w = placements[idx].width as f32;
            let rect_h = placements[idx].height as f32;
            let mut x = placements[idx].x.into_inner();
            if bin_width.is_finite() {
                x = x.clamp(0.0, (bin_width - rect_w).max(0.0));
            }

            let mut support_y = 0.0_f32;
            for &j in &order {
                if !placed[j] || j == idx {
                    continue;
                }
                let jx = placements[j].x.into_inner();
                let jy = placements[j].y.into_inner();
                let jw = placements[j].width as f32;
                let jh = placements[j].height as f32;

                let overlap_x = x < jx + jw && x + rect_w > jx;
                if overlap_x {
                    support_y = support_y.max(jy + jh);
                }
            }

            placements[idx].x = OrderedFloat(x);
            placements[idx].y = OrderedFloat(support_y);
            placed[idx] = true;
        }
    }

    fn apply_auto_minimize_height(&mut self) {
        if let Some(tab) = self.active_algo_tab_mut() && let Some(output) = &mut tab.algorithm_output {
            let bin_width = output.bin_width as f32;
            Self::gravity_collapse(bin_width, &mut output.placements);
            let mut max_height = 0.0_f32;
            for placement in &output.placements {
                let top = placement.y.into_inner() + placement.height as f32;
                if top > max_height {
                    max_height = top;
                }
            }
            output.total_height = max_height;
            tab.output_revision = tab.output_revision.wrapping_add(1);
            self.rebuild_hit_grid();
        }
    }

    fn create_new_tab(&mut self, output: Option<&AlgorithmOutput>) -> u64 {
	let root_count = self.algo_tabs.iter().filter(|t| t.name.starts_with("Root")).count();
	let name = format!("Root {}", root_count + 1);
	let new_id = self.next_algo_tab_id;
	self.next_algo_tab_id = self.next_algo_tab_id.wrapping_add(1);
	self.algo_tabs.push(AlgoTab {
	    id: new_id,
	    name,
	    selected_indices: Vec::new(),
	    repacked_indices: Vec::new(),
	    obstacle_spaces: Vec::new(),
	    selection_regions: Vec::new(),
	    code: self.code_editor_content.text(),
	    last_right_panel_tab: RightPanelTab::Visualization,
	    algorithm_output: output.cloned(),
	    parent_output: None,
	    repack_output: None,
	    output_revision: if output.is_some() { 1 } else { 0 },
	    hit_grid: None,
	    visible_rects: output.map(|o| o.placements.len()).unwrap_or(0),
	    animating: false,
	});
	self.set_active_algo_tab(new_id);
	new_id
    }

    fn set_active_algo_tab(&mut self, tab_id: u64) {
        // Save current tab's code before switching
        let current_code = self.code_editor_content.text();
        if let Some(current_tab) = self.algo_tabs.iter_mut().find(|t| t.id == self.active_algo_tab_id) {
            current_tab.code = current_code;
            current_tab.last_right_panel_tab = self.active_tab;
        }

        self.active_algo_tab_id = tab_id;

        // Reset area select state for new tab (each tab can create regions independently)
        self.new_area_select = true;

        // Load new tab's code
        if let Some(new_tab) = self.algo_tabs.iter().find(|t| t.id == tab_id) {
            self.code_editor_content = text_editor::Content::with_text(&new_tab.code);
            self.selected_rects.clear();
            for idx in &new_tab.selected_indices {
                self.selected_rects.insert(*idx);
            }
            self.active_tab = new_tab.last_right_panel_tab;
            if new_tab.algorithm_output.is_some() {
                self.rebuild_hit_grid();
            }
        } else {
            self.selected_rects.clear();
        }
    }

    fn update_active_tab_selection_from_current(&mut self) {
        if self.active_algo_tab_id == 0 {
            return;
        }
        if let Some(tab) = self.algo_tabs.iter_mut().find(|t| t.id == self.active_algo_tab_id) {
            tab.selected_indices = self.selected_rects.iter().copied().collect();
            tab.selected_indices.sort_unstable();
        }
    }

    fn split_region_rectangles(&self, output: &AlgorithmOutput, region: &SelectionRegion) -> (Vec<usize>, Vec<usize>) {
        let mut selected = Vec::new();
        let mut non_selected = Vec::new();

        let region_min_x = region.bin_x;
        let region_max_x = region.bin_x + region.bin_w;
        let region_min_y = region.bin_y;
        let region_max_y = region.bin_y + region.bin_h;

        for &idx in &self.selected_rects {
            if idx < output.placements.len() {
                selected.push(idx);
            }
        }
        selected.sort_unstable();

        for (idx, p) in output.placements.iter().enumerate() {
            if self.selected_rects.contains(&idx) {
                continue;
            }

            let rect_min_x = p.x.into_inner();
            let rect_max_x = rect_min_x + p.width as f32;
            let rect_min_y = p.y.into_inner();
            let rect_max_y = rect_min_y + p.height as f32;

            let intersects = rect_max_x >= region_min_x
                && rect_min_x <= region_max_x
                && rect_max_y >= region_min_y
                && rect_min_y <= region_max_y;

            if intersects {
                non_selected.push(idx);
            }
        }

        (selected, non_selected)
    }

    fn parse_rectangles(&self) -> Result<ParseOutput, Vec<String>> {
        let text = self.rectangle_data.text();
        let mut rectangles: Vec<Rectangle> = Vec::new();
        let mut errors = Vec::new();
        let mut w_val: i32 = -1;
        let mut total: i32 = 0;
        let mut Ntemp: i32 = -1;
        let mut Ktemp: i32 = -1;
        let mut set: HashSet<(i32, i32)> = HashSet::new();
        let mut min_height: i32 = i32::MAX;
        let mut max_height: i32 = i32::MIN;

        if self.w_input.is_empty() {
            errors.push("Enter a value for the width of the bin".to_string());
            return Err(errors);
        }

        if let Ok(w) = self.w_input.parse::<i32>() {
            if w < 0 {
                errors.push("Emter a positive value for the width of the bin".to_string());
                return Err(errors);
            }
            w_val = w;
        } else {
            errors.push("Enter an integer value for the width of the bin".to_string());
            return Err(errors);           
        }

        if !self.n_input.is_empty() {
            if let Ok(n) = self.n_input.parse::<i32>() {
                if n < 0 {
                    errors.push("Enter an integer value for the quantity of rectangles".to_string());
                    return Err(errors);
                }
                Ntemp = n;
            } else {
                errors.push("Enter an integer value for the quantity of rectangles".to_string());
            }
        }


        if !self.k_input.is_empty() {
            if let Ok(k) = self.k_input.parse::<i32>() {
                if k < 0 {
                    errors.push("Enter an integer value for the types f rectangles".to_string());
                    return Err(errors);
                }
                Ktemp = k;
            } else {
                errors.push("Enter an integer value for the types of rectangles".to_string());
            }
        }


        if !errors.is_empty() {
            return Err(errors)
        }

        for (line_num, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() { continue; }
            
            let parts: Vec<&str> = line.split_whitespace().collect();


            if parts.len() != 3 {
                errors.push(format!("Line {}: Expected exactly 3 space-separated values (X Y Q), found {}", 
                    line_num + 1, parts.len()));
                continue;
            }

            let x_result = parts[0].parse::<i32>();
            let y_result = parts[1].parse::<i32>();
            let q_result = parts[2].parse::<i32>();

            match (x_result, y_result, q_result) {
                (Ok(x), Ok(y), Ok(q)) => {
                    if x <= 0 || y <= 0 || q <= 0 {
                        errors.push(format!("Line {}: X, Y, and Q must be positive", line_num + 1));
                        continue;
                    }

                    min_height = i32::min(y, min_height);
                    max_height = i32::max(y, max_height);

                    total += q;
                    if x > w_val {
                        errors.push(format!("Line {}: '{}' is greater than the width {}", line_num+1, parts[0], w_val));
                    } else if let Some(existing) = rectangles.iter_mut().find(|r| r.width == x && r.height == y) {
                        existing.quantity += q;
                        set.insert((x, y));
                    } else {
                        rectangles.push(Rectangle {
                            width: x,
                            height: y,
                            quantity: q
                        });
                        set.insert((x, y));
                    }
                }
                (Err(_), _, _) => {
                    errors.push(format!("Line {}: '{}' is not a valid integer for X", 
                        line_num + 1, parts[0]));
                }
                (_, Err(_), _) => {
                    errors.push(format!("Line {}: '{}' is not a valid integer for Y", 
                        line_num + 1, parts[1]));
                }
                (_, _, Err(_)) => {
                    errors.push(format!("Line {}: '{}' is not a valid integer for Q", 
                        line_num + 1, parts[2]));
                }
            }
        }

        if !self.autofile && Ntemp != -1 && Ntemp != total {
            errors.push(format!("The quantity of rectangles is NOT the same as the input. {} rectangles found, {} expected.", total, Ntemp));
        }

        if !self.autofile && Ktemp != -1 && Ktemp != set.len() as i32 {
            errors.push(format!("The number of types of rectangles is NOT the same as the input. {} types found, {} expected.", set.len(), Ktemp));
        }

        if self.autofile {
            let actual_n = total;
            let actual_k = set.len() as i32;
            
            if Ktemp != -1 && actual_k > Ktemp {
                errors.push(format!("The number of types of rectangles is greater than the input. {} types found, {} expected.", actual_k, Ktemp));
            }
            
            if Ntemp != -1 && Ktemp != -1 {
                let n_difference = Ntemp - actual_n;
                let k_difference = Ktemp - actual_k;
                
                if n_difference > 0 && k_difference > n_difference {
                    errors.push(format!(
                        "Autofill impossible: Need to add {} rectangles but only {} type slots available. \
                        (Input N={}, Actual N={}, Input K={}, Actual K={})",
                        n_difference, k_difference, Ntemp, actual_n, Ktemp, actual_k
                    ));
                }
                
                if n_difference < 0 {
                    errors.push(format!(
                        "Autofill impossible: Already have {} rectangles but input N={} (cannot remove rectangles)",
                        actual_n, Ntemp
                    ));
                }
            }
        }
        
        if errors.is_empty() {
            Ok(ParseOutput {width: w_val, quantity: Ntemp, types: Ktemp, autofill: self.autofile, rects: rectangles, input_types: set.len() as i32,min_height, max_height})
        } else {
            Err(errors)
        }
    }

    pub fn view(&self) -> Element<'_, Input> {
        let ui_font = Font::default();
        
        let title = text("Rectangle Packing Configuration")
            .size(22)
            .font(ui_font);
        
        let header = column![
            title,
        ]
        .spacing(4);
        
        let w_label = text("Bin Width")
            .size(12)
            .font(ui_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.533, 0.533, 0.627)),
                }
            });
        
        let w_input = text_input("e.g., 100", &self.w_input)
            .on_input(Input::WChanged)
            .size(13)
            .padding(10)
            .width(Length::Fill)
            .font(ui_font);

        let w_input_container = container(w_input)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.1, 0.1, 0.13).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.2, 0.2, 0.26),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            });
        
        let n_label = text("Number of Rectangles")
            .size(12)
            .font(ui_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.533, 0.533, 0.627)),
                }
            });

        let n_input = text_input("Optional", &self.n_input)
            .on_input(Input::NChanged)
            .size(13)
            .padding(10)
            .width(Length::Fill)
            .font(ui_font);

        let n_input_container = container(n_input)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.1, 0.1, 0.13).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.2, 0.2, 0.26),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            });
        
        let k_label = text("Number of Rectangle Types")
            .size(12)
            .font(ui_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.533, 0.533, 0.627)),
                }
            });

        let k_input = text_input("Optional", &self.k_input)
            .on_input(Input::KChanged)
            .size(13)
            .padding(10)
            .width(Length::Fill)
            .font(ui_font);

        let k_input_container = container(k_input)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.1, 0.1, 0.13).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.2, 0.2, 0.26),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            });
        
        let autofill_checkbox = if self.n_input.is_empty() {
            checkbox("Autofill remaining values", self.autofile)
                .size(10)
                .font(ui_font)
        } else {
            checkbox("Autofill remaining values", self.autofile)
                .on_toggle(Input::AutofillChanged)
                .size(10)
                .font(ui_font)
        };
        
        let autofill_container = container(autofill_checkbox)
            .padding([8, 0]);
        
        let divider = container(
            container(text(""))
                .width(Length::Fill)
                .height(1)
                .style(|_theme: &Theme| {
                    container::Style {
                        background: Some(Color::from_rgb(0.165, 0.165, 0.22).into()),
                        ..Default::default()
                    }
                })
        );
        
        let import_button = button(
            container(
                text("Import Configuration")
                    .size(13)
                    .font(ui_font)
            )
            .center_x(Length::Fill)
        )
        .on_press(Input::ImportPressed)
        .padding(12)
        .width(Length::Fill)
        .style(|_theme: &Theme, status| {
            let base_bg = Color::from_rgb(0.14, 0.14, 0.17);
            let hover_bg = Color::from_rgb(0.18, 0.18, 0.22);

            button::Style {
                background: Some(match status {
                    button::Status::Hovered => hover_bg.into(),
                    _ => base_bg.into(),
                }),
                border: iced::Border {
                    color: Color::from_rgb(0.24, 0.24, 0.28),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: Color::from_rgb(0.75, 0.75, 0.78),
                ..Default::default()
            }
        });
        
        
        let rectangle_label = text("Rectangle Values")
            .size(12)
            .font(ui_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.533, 0.533, 0.627)),
                }
            });

        let rectangle_hint = text("Format: X Y Q (space-separated)")
            .size(10)
            .font(ui_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.4, 0.4, 0.47)),
                }
            });
        
        let editor_header = column![
            rectangle_label,
            rectangle_hint,
        ]
        .spacing(2);
        
        let rectangle_editor = text_editor(&self.rectangle_data)
            .on_action(Input::RectangleDataAction)
            .height(180)
            .padding(12)
            .size(13)
            .font(ui_font);

        let editor_container = container(rectangle_editor)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.06, 0.06, 0.08).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.2, 0.2, 0.26),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            });
        
        let line_info = text(format!("Line {} of {}", self.rect_cursor_line, self.rect_total_lines))
            .size(10)
            .font(ui_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.4, 0.4, 0.47)),
                }
            });

        let editor_with_info = column![
            editor_container,
            line_info,
        ]
        .spacing(8);
        
        let export_button = button(
            container(
                text("Export Algorithm Input")
                    .size(13)
                    .font(ui_font)
            )
            .center_x(Length::Fill)
        )
        .on_press(Input::ExportAlgorithmInput)
        .padding(12)
        .width(Length::Fill)
        .style(|_theme: &Theme, status| {
            let base_bg = Color::from_rgb(0.22, 0.22, 0.26);
            let hover_bg = Color::from_rgb(0.28, 0.28, 0.32);

            button::Style {
                background: Some(match status {
                    button::Status::Hovered => hover_bg.into(),
                    _ => base_bg.into(),
                }),
                border: iced::Border {
                    color: Color::from_rgb(0.32, 0.32, 0.38),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: Color::from_rgb(0.9, 0.9, 0.92),
                ..Default::default()
            }
        });
        
        let message_display = if let Some(msg) = &self.error_message {
            let is_success = msg.starts_with("✓");
            let (bg_color, border_color, text_color) = if is_success {
                (
                    Color::from_rgb(0.06, 0.15, 0.1),
                    Color::from_rgb(0.15, 0.45, 0.25),
                    Color::from_rgb(0.4, 0.85, 0.5)
                )
            } else {
                (
                    Color::from_rgb(0.18, 0.08, 0.08),
                    Color::from_rgb(0.55, 0.18, 0.18),
                    Color::from_rgb(1.0, 0.55, 0.55)
                )
            };

            container(
                scrollable(
                    text(msg)
                        .size(11)
                        .font(ui_font)
                        .style(move |_theme: &Theme| {
                            text::Style {
                                color: Some(text_color),
                            }
                        })
                )
                .height(Length::Fixed(45.0))
            )
            .padding(12)
            .width(Length::Fill)
            .style(move |_theme: &Theme| {
                container::Style {
                    background: Some(bg_color.into()),
                    border: iced::Border {
                        color: border_color,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            })
        } else {
            container(text("").size(1))
        };

        let import_output_json_button = button(
            container(
                text("Import Output JSON")
                    .size(13)
                    .font(ui_font)
            )
            .center_x(Length::Fill)
        )
        .on_press(Input::ImportOutputJsonPressed)
        .padding(12)
        .width(Length::Fill)
        .style(|_theme: &Theme, status| {
            let base_bg = Color::from_rgb(0.18, 0.18, 0.22);
            let hover_bg = Color::from_rgb(0.24, 0.24, 0.28);

            button::Style {
                background: Some(match status {
                    button::Status::Hovered => hover_bg.into(),
                    _ => base_bg.into(),
                }),
                border: iced::Border {
                    color: Color::from_rgb(0.28, 0.28, 0.34),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: Color::from_rgb(0.85, 0.85, 0.88),
                ..Default::default()
            }
        });

        let animation_speed_label = text("Animation Speed (ms)")
            .size(12)
            .font(ui_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.533, 0.533, 0.627)),
                }
            });

        let animation_speed_slider = slider(10.0..=500.0, self.animation_speed, Input::AnimationSpeedChanged)
            .width(Length::Fill)
            .step(10.0);

        let animation_speed_value = text(format!("{:.0}ms", self.animation_speed))
            .size(11)
            .font(ui_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.7, 0.7, 0.75)),
                }
            });

        let animation_speed_container = container(
            column![
                animation_speed_label,
                column![].height(4),
                row![
                    animation_speed_slider,
                    column![].width(8),
                    animation_speed_value,
                ].spacing(0).align_y(Alignment::Center),
            ].spacing(4)
        );

        let import_output_json_container = container(import_output_json_button)
            .style(|_theme: &Theme| {
                container::Style {
                    background: None,
                    border: iced::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            });

        let settings_panel_visible = self.settings_panel_visible;
        let gear_button = button(
            container(
                column![
                    row![
                        container(text("")).width(2).height(2).style(|_theme: &Theme| {
                            container::Style {
                                background: Some(Color::from_rgb(0.6, 0.6, 0.65).into()),
                                border: iced::Border { radius: 1.0.into(), ..Default::default() },
                                ..Default::default()
                            }
                        }),
                        column![].width(2),
                        container(text("")).width(2).height(2).style(|_theme: &Theme| {
                            container::Style {
                                background: Some(Color::from_rgb(0.6, 0.6, 0.65).into()),
                                border: iced::Border { radius: 1.0.into(), ..Default::default() },
                                ..Default::default()
                            }
                        }),
                    ],
                    column![].height(2),
                    row![
                        container(text("")).width(2).height(2).style(|_theme: &Theme| {
                            container::Style {
                                background: Some(Color::from_rgb(0.6, 0.6, 0.65).into()),
                                border: iced::Border { radius: 1.0.into(), ..Default::default() },
                                ..Default::default()
                            }
                        }),
                        column![].width(2),
                        container(text("")).width(2).height(2).style(|_theme: &Theme| {
                            container::Style {
                                background: Some(Color::from_rgb(0.6, 0.6, 0.65).into()),
                                border: iced::Border { radius: 1.0.into(), ..Default::default() },
                                ..Default::default()
                            }
                        }),
                    ],
                    column![].height(2),
                    row![
                        container(text("")).width(2).height(2).style(|_theme: &Theme| {
                            container::Style {
                                background: Some(Color::from_rgb(0.6, 0.6, 0.65).into()),
                                border: iced::Border { radius: 1.0.into(), ..Default::default() },
                                ..Default::default()
                            }
                        }),
                        column![].width(2),
                        container(text("")).width(2).height(2).style(|_theme: &Theme| {
                            container::Style {
                                background: Some(Color::from_rgb(0.6, 0.6, 0.65).into()),
                                border: iced::Border { radius: 1.0.into(), ..Default::default() },
                                ..Default::default()
                            }
                        }),
                    ],
                ]
            )
            .padding(4)
        )
        .on_press(Input::ToggleSettingsPanel)
        .padding(4)
        .style(move |_theme: &Theme, status| {
            let base_bg = if settings_panel_visible {
                Color::from_rgb(0.2, 0.25, 0.35)
            } else {
                Color::from_rgba(0.12, 0.12, 0.15, 0.9)
            };
            let hover_bg = Color::from_rgba(0.22, 0.22, 0.28, 0.95);

            button::Style {
                background: Some(match status {
                    button::Status::Hovered => hover_bg.into(),
                    _ => base_bg.into(),
                }),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.3, 0.35),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: Color::from_rgb(0.75, 0.75, 0.8),
                ..Default::default()
            }
        });

        let area_select_enabled = self.settings.area_select_enabled;
        let snap_to_rects_enabled = self.settings.snap_to_rectangles_enabled;
        let auto_minimize_enabled = self.settings.auto_minimize_height;

        let settings_popup: Element<'_, Input> = if self.settings_panel_visible {
            let area_select_checkbox = checkbox("Area Selection (Right-drag)", area_select_enabled)
                .on_toggle(Input::ToggleAreaSelectEnabled)
                .size(14)
                .font(ui_font)
                .text_size(11);

            let snap_checkbox = checkbox("Snap to Edge", snap_to_rects_enabled)
                .on_toggle(Input::ToggleSnapToRectangles)
                .size(14)
                .font(ui_font)
                .text_size(11);

            let auto_minimize_checkbox = checkbox("Auto Minimize Height", auto_minimize_enabled)
                .on_toggle(Input::ToggleAutoMinimizeHeight)
                .size(14)
                .font(ui_font)
                .text_size(11);

            container(
                column![
                    text("Settings").size(12).font(ui_font).style(|_theme: &Theme| {
                        text::Style {
                            color: Some(Color::from_rgb(0.7, 0.7, 0.75)),
                        }
                    }),
                    column![].height(8),
                    area_select_checkbox,
                    column![].height(4),
                    snap_checkbox,
                    column![].height(4),
                    auto_minimize_checkbox,
                ]
                .spacing(4)
            )
            .padding(12)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.08, 0.08, 0.1).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.25, 0.25, 0.32),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
        } else {
            column![].into()
        };

        let settings_corner: Element<'_, Input> = column![
            row![
                column![].width(Length::Fill),
                gear_button,
            ],
            settings_popup,
        ]
        .spacing(4)
        .align_x(Alignment::End)
        .into();

let current_tab_regions: &[SelectionRegion] = self.algo_tabs.iter()
    .find(|t| t.id == self.active_algo_tab_id)
    .map(|t| t.selection_regions.as_slice())
    .unwrap_or(&[]);

let visualization_content = if let Some(tab) = self.active_algo_tab() && let Some(output) = &tab.algorithm_output {
    let canvas = Canvas::new(BinCanvas {
            output,
            output_revision: tab.output_revision,
            hit_grid: tab.hit_grid.as_ref(),
            zoom: self.zoom,
            visible_count: tab.visible_rects,
            pan_x: self.pan_x,
            pan_y: self.pan_y,
            hovered_rect: self.hovered_rect,
            is_panning: self.is_panning,
            dragged_rect: self.dragged_rect,
            dragged_rect_offset_x: self.dragged_rect_offset_x,
            dragged_rect_offset_y: self.dragged_rect_offset_y,
            snap_preview: self.snap_preview,
            animating: tab.animating,
            selected_rects: &self.selected_rects,
            repacked_indices: Some(tab.repacked_indices.as_slice()),
            obstacle_spaces: Some(tab.obstacle_spaces.as_slice()),
            is_area_selecting: self.is_area_selecting,
            area_select_start: self.area_select_start,
            area_select_current: self.area_select_current,
            settings: &self.settings,
            selection_regions: current_tab_regions,
        })
        .width(Length::Fill)
        .height(Length::Fill);
    let height_display = container(
        text(format!("Total Height: {}", output.total_height))
            .size(12)
            .font(ui_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.85, 0.87, 0.9)),
                }
            })
    )
    .padding(10)
    .style(|_theme: &Theme| {
        container::Style {
            background: Some(Color::from_rgb(0.1, 0.1, 0.13).into()),
            border: iced::Border {
                color: Color::from_rgb(0.2, 0.2, 0.26),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    });

    let dimensions_display = if let Some(hovered_idx) = self.hovered_rect {
        if hovered_idx < output.placements.len() {
            let placement = &output.placements[hovered_idx];
            container(
                text(format!("Width: {} | Height: {}", placement.width, placement.height))
                    .size(12)
                    .font(ui_font)
                    .style(|_theme: &Theme| {
                        text::Style {
                            color: Some(Color::from_rgb(0.85, 0.87, 0.9)),
                        }
                    })
            )
            .padding(10)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.1, 0.1, 0.13).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.35, 0.35, 0.4),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            })
        } else {
            container(text("").size(12))
        }
    } else {
        container(text("Hover over a rectangle to see dimensions").size(12).style(|_theme: &Theme| {
            text::Style {
                color: Some(Color::from_rgb(0.45, 0.47, 0.52)),
            }
        }))
        .padding(10)
        .style(|_theme: &Theme| {
            container::Style {
                background: Some(Color::from_rgb(0.1, 0.1, 0.13).into()),
                border: iced::Border {
                    color: Color::from_rgb(0.2, 0.2, 0.26),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        })
    };

    let context_menu_overlay: Element<'_, Input> = if self.context_menu_visible {
        if let Some(region_idx) = self.context_menu_region {
            let is_inherited = self.algo_tabs.iter()
                .find(|t| t.id == self.active_algo_tab_id)
                .and_then(|tab| tab.selection_regions.get(region_idx))
                .map(|r| r.is_inherited)
                .unwrap_or(false);

            let remove_button = button(
                container(
                    text("Remove")
                        .size(12)
                        .font(ui_font)
                )
                .center_x(Length::Fill)
            )
            .on_press(Input::RemoveSelectionRegion(region_idx))
            .padding([8, 16])
            .width(Length::Fill)
            .style(|_theme: &Theme, status| {
                let base_bg = Color::from_rgb(0.12, 0.12, 0.15);
                let hover_bg = Color::from_rgb(0.18, 0.18, 0.22);
                button::Style {
                    background: Some(match status {
                        button::Status::Hovered => hover_bg.into(),
                        _ => base_bg.into(),
                    }),
                    text_color: Color::from_rgb(0.9, 0.9, 0.92),
                    ..Default::default()
                }
            });

            let menu_content: Element<'_, Input> = if is_inherited {
                column![remove_button].spacing(2).width(120).into()
            } else {
                let repack_button = button(
                    container(
                        text("Repack Region")
                            .size(12)
                            .font(ui_font)
                    )
                    .center_x(Length::Fill)
                )
                .on_press(Input::RepackSelectionRegion(region_idx))
                .padding([8, 16])
                .width(Length::Fill)
                .style(|_theme: &Theme, status| {
                    let base_bg = Color::from_rgb(0.12, 0.12, 0.15);
                    let hover_bg = Color::from_rgb(0.18, 0.18, 0.22);
                    button::Style {
                        background: Some(match status {
                            button::Status::Hovered => hover_bg.into(),
                            _ => base_bg.into(),
                        }),
                        text_color: Color::from_rgb(0.9, 0.9, 0.92),
                        ..Default::default()
                    }
                });

                column![remove_button, repack_button].spacing(2).width(120).into()
            };

            container(menu_content)
            .padding(4)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.1, 0.1, 0.13).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.3, 0.35),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
        } else {
            column![].into()
        }
    } else {
        column![].into()
    };

    column![
        container(
            iced::widget::stack![
                container(canvas)
                    .width(Length::Fill)
                    .height(Length::Fill),
                container(settings_corner)
                    .width(Length::Fill)
                    .padding(8)
                    .align_right(Length::Fill),
                column![
                    column![].height(Length::Fixed(self.context_menu_position.1)),
                    row![
                        column![].width(Length::Fixed(self.context_menu_position.0)),
                        context_menu_overlay,
                    ],
                ]
                .width(Length::Shrink)
                .height(Length::Shrink),
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill),
        column![].height(16),
        row![
            dimensions_display.width(Length::Fill),
            height_display.width(Length::Fill),
        ]
        .spacing(16)
        .width(Length::Fill)
        .align_y(Alignment::Center),
    ]
    .align_x(Alignment::Center)
    .spacing(8)
        } else {
            column![
                container(
                    iced::widget::stack![
                        container(
                            column![
                                text("Visualization Area")
                                    .size(16)
                                    .font(ui_font)
                                    .style(|_theme: &Theme| {
                                        text::Style {
                                            color: Some(Color::from_rgb(0.533, 0.533, 0.627)),
                                        }
                                    }),
                                column![].height(8),
                                text("Import Output JSON or Run Custom Algorithm to see the packing result")
                                    .size(12)
                                    .font(ui_font)
                                    .style(|_theme: &Theme| {
                                        text::Style {
                                            color: Some(Color::from_rgb(0.4, 0.4, 0.47)),
                                        }
                                    }),
                            ]
                            .align_x(Alignment::Center)
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill),
                        container(settings_corner)
                            .width(Length::Fill)
                            .padding(8)
                            .align_right(Length::Fill),
                    ]
                )
                .width(Length::Fill)
                .height(Length::Fill),
            ]
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
        };
        
        let stats_display = if let Some(tab) = self.active_algo_tab() && let Some(output) = &tab.algorithm_output {
            let rect_count_text = text(format!("Rectangles: {}/{}", tab.visible_rects, output.placements.len()))
                .size(11)
                .font(ui_font)
                .style(|_theme: &Theme| {
                    text::Style {
                        color: Some(Color::from_rgb(0.6, 0.6, 0.65)),
                    }
                });

            let zoom_text = text(format!("Zoom: {:.0}%", self.zoom * 100.0))
                .size(11)
                .font(ui_font)
                .style(|_theme: &Theme| {
                    text::Style {
                        color: Some(Color::from_rgb(0.6, 0.6, 0.65)),
                    }
                });

            container(
                row![
                    rect_count_text,
                    column![].width(Length::Fill),
                    zoom_text,
                ].spacing(8).width(Length::Fill)
            )
            .padding(10)
            .width(Length::Fill)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.1, 0.1, 0.13).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.2, 0.2, 0.26),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            })
        } else {
            container(text("").size(1))
        };

        let input_section = column![
            header,
            column![].height(16),
            column![
                w_label,
                column![].height(6),
                w_input_container,
            ].spacing(0),
            column![].height(12),
            column![
                n_label,
                column![].height(6),
                n_input_container,
            ].spacing(0),
            column![].height(12),
            column![
                k_label,
                column![].height(6),
                k_input_container,
            ].spacing(0),
            column![].height(12),
            autofill_container,
            column![].height(16),
            divider,
            column![].height(16),
            row![
                import_button,
            ].spacing(8),
            column![].height(16),
            editor_header,
            column![].height(6),
            editor_with_info,
            column![].height(12),
            export_button,
            column![].height(12),
            message_display,
        ]
        .spacing(0)
        .padding(20)
        .align_x(Alignment::Start);

        let input_container = container(input_section)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.08, 0.08, 0.1).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.16, 0.16, 0.2),
                        width: 1.0,
                        radius: 10.0.into(),
                    },
                    ..Default::default()
                }
            });

        let output_section = column![
            import_output_json_container,
            column![].height(12),
            animation_speed_container,
            column![].height(12),
            stats_display,
        ]
        .spacing(0)
        .padding(16)
        .align_x(Alignment::Start);

        let output_container = container(output_section)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.08, 0.08, 0.1).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.16, 0.16, 0.2),
                        width: 1.0,
                        radius: 10.0.into(),
                    },
                    ..Default::default()
                }
            });

        let left_panel = column![
            input_container,
            column![].height(12),
            output_container,
        ]
        .spacing(0);

        let left_panel_scrollable = scrollable(left_panel)
            .width(Length::Fill)
            .height(Length::Fill);

        let left_panel_container = container(left_panel_scrollable)
            .width(Length::FillPortion(1))
            .height(Length::Fill);

        let viz_tab_active = self.active_tab == RightPanelTab::Visualization;
        let code_tab_active = self.active_tab == RightPanelTab::CodeEditor;

        let viz_tab = button(
            column![
                text("Visualization").size(12).font(ui_font),
                container(text(""))
                    .width(Length::Fill)
                    .height(2)
                    .style(move |_theme: &Theme| {
                        container::Style {
                            background: if viz_tab_active {
                                Some(Color::from_rgb(0.6, 0.6, 0.65).into())
                            } else {
                                None
                            },
                            ..Default::default()
                        }
                    }),
            ]
            .spacing(6)
            .align_x(Alignment::Center)
        )
        .on_press(Input::TabSelected(RightPanelTab::Visualization))
        .padding([10, 20])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered => Color::from_rgb(0.1, 0.1, 0.12),
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(bg.into()),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
                text_color: if viz_tab_active {
                    Color::from_rgb(0.9, 0.9, 0.92)
                } else {
                    Color::from_rgb(0.5, 0.5, 0.55)
                },
                ..Default::default()
            }
        });

        let code_tab = button(
            column![
                text("Code").size(12).font(ui_font),
                container(text(""))
                    .width(Length::Fill)
                    .height(2)
                    .style(move |_theme: &Theme| {
                        container::Style {
                            background: if code_tab_active {
                                Some(Color::from_rgb(0.6, 0.6, 0.65).into())
                            } else {
                                None
                            },
                            ..Default::default()
                        }
                    }),
            ]
            .spacing(6)
            .align_x(Alignment::Center)
        )
        .on_press(Input::TabSelected(RightPanelTab::CodeEditor))
        .padding([10, 20])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered => Color::from_rgb(0.1, 0.1, 0.12),
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(bg.into()),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
                text_color: if code_tab_active {
                    Color::from_rgb(0.9, 0.9, 0.92)
                } else {
                    Color::from_rgb(0.5, 0.5, 0.55)
                },
                ..Default::default()
            }
        });

        let algo_tab_bar = {
            let tabs = self.algo_tabs.clone();
            let active_id = self.active_algo_tab_id;
            row(tabs.into_iter().map(|tab| {
                let is_active = tab.id == active_id;
                let is_root = tab.id == 0;

                let tab_label = text(tab.name).size(11).font(ui_font);

                let close_btn: Element<'_, Input> = if !is_root {
                    button(text("×").size(10).font(ui_font))
                        .on_press(Input::RemoveAlgoTab(tab.id))
                        .padding([2, 4])
                        .style(|_theme: &Theme, status| {
                            button::Style {
                                background: Some(match status {
                                    button::Status::Hovered => Color::from_rgba(1.0, 0.4, 0.4, 0.4).into(),
                                    _ => Color::TRANSPARENT.into(),
                                }),
                                border: iced::Border { radius: 3.0.into(), ..Default::default() },
                                text_color: Color::from_rgb(0.6, 0.6, 0.65),
                                ..Default::default()
                            }
                        })
                        .into()
                } else {
                    container(text("")).width(0).into()
                };

                let tab_content = row![
                    tab_label,
                    close_btn,
                ]
                .spacing(6)
                .align_y(Alignment::Center);

                let tab_btn: Element<'_, Input> = button(tab_content)
                    .on_press(Input::AlgoTabSelected(tab.id))
                    .padding([8, 12])
                    .style(move |_theme: &Theme, status| {
                        let (bg, border_color) = match status {
                            button::Status::Hovered => if is_active {
                                (Color::from_rgb(0.1, 0.1, 0.12), Color::from_rgb(0.18, 0.18, 0.22))
                            } else {
                                (Color::from_rgb(0.08, 0.08, 0.1), Color::from_rgb(0.14, 0.14, 0.18))
                            },
                            _ => if is_active {
                                (Color::from_rgb(0.09, 0.09, 0.11), Color::from_rgb(0.16, 0.16, 0.2))
                            } else {
                                (Color::from_rgb(0.055, 0.055, 0.07), Color::from_rgb(0.1, 0.1, 0.13))
                            },
                        };
                        button::Style {
                            background: Some(bg.into()),
                            border: iced::Border {
                                color: border_color,
                                width: 1.0,
                                radius: iced::border::Radius {
                                    top_left: 6.0,
                                    top_right: 6.0,
                                    bottom_left: 0.0,
                                    bottom_right: 0.0,
                                },
                            },
                            text_color: if is_active {
                                Color::from_rgb(0.92, 0.92, 0.94)
                            } else {
                                Color::from_rgb(0.55, 0.55, 0.6)
                            },
                            ..Default::default()
                        }
                    })
                    .into();

                tab_btn
            }))
            .push(
                button(text("+").size(14).font(ui_font))
                    .on_press(Input::CreateNewTab)
                    .padding([6, 10])
                    .style(|_theme: &Theme, status| button::Style {
                        background: Some(match status {
                            button::Status::Hovered => Color::from_rgb(0.10, 0.10, 0.13).into(),
                            _ => Color::TRANSPARENT.into(),
                        }),
                        border: iced::Border { radius: 4.0.into(), ..Default::default() },
                        text_color: Color::from_rgb(0.5, 0.5, 0.55),
                        ..Default::default()
                    })
            )
            .spacing(1)
            .align_y(Alignment::End)
        };

        let tab_bar = container(
            row![
                viz_tab,
                code_tab,
            ]
            .spacing(0)
        )
        .style(|_theme: &Theme| {
            container::Style {
                background: Some(Color::from_rgb(0.055, 0.055, 0.07).into()),
                border: iced::Border {
                    color: Color::from_rgb(0.12, 0.12, 0.15),
                    width: 0.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fill)
        .padding([4, 8]);

        let editor_state = EditorState {
            code_editor_content: &self.code_editor_content,
            selected_language: self.selected_language,
            bottom_panel_visible: self.bottom_panel_visible,
            bottom_panel_tab: self.bottom_panel_tab,
            code_errors: &self.code_errors,
            code_output_json: self.code_output_json.as_deref(),
            show_visualization_button: false,
            testcase_message: self.testcase_message.as_deref(),
            testcase: self.current_testcase.as_ref(),
            is_root: self.active_algo_tab()
                .map(|t| !t.selection_regions.iter().any(|r| r.is_inherited))
                .unwrap_or(true),
            num_test_cases_input: &self.num_test_cases_input,
            input_size_input: &self.input_size_input,
            unique_types_input: &self.unique_types_input,
            display_visual: self.display_visual,
            multiple_testcase_message: self.multiple_testcase_message.as_deref(),
            multiple_run_results: &self.multiple_run_results,
            multiple_results_expanded: &self.multiple_results_expanded,
            bottom_panel_height: self.bottom_panel_height,
        };
        let code_panel_content = {
            use iced::widget::mouse_area;
            let inner = build_code_panel(&editor_state);
            mouse_area(inner)
                .on_move(|p| Input::PanelResizeMove(p.y))
                .on_release(Input::PanelResizeEnd)
        };

        let right_panel_content: Element<'_, Input> = match self.active_tab {
            RightPanelTab::Visualization => {
                container(visualization_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(16)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            }
            RightPanelTab::CodeEditor => {
                container(code_panel_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(16)
                    .into()
            }
        };

        let right_panel = column![
            container(algo_tab_bar)
                .width(Length::Fill)
                .padding([0, 6])
                .style(|_theme: &Theme| {
                    container::Style {
                        background: Some(Color::from_rgb(0.04, 0.04, 0.05).into()),
                        border: iced::Border {
                            color: Color::from_rgb(0.1, 0.1, 0.12),
                            width: 1.0,
                            radius: iced::border::Radius {
                                top_left: 8.0,
                                top_right: 8.0,
                                bottom_left: 0.0,
                                bottom_right: 0.0,
                            },
                        },
                        ..Default::default()
                    }
                }),
            tab_bar,
            container(right_panel_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| {
                    container::Style {
                        background: Some(Color::from_rgb(0.06, 0.06, 0.08).into()),
                        border: iced::Border {
                            color: Color::from_rgb(0.16, 0.16, 0.2),
                            width: 1.0,
                            radius: 10.0.into(),
                        },
                        ..Default::default()
                    }
                }),
        ]
        .spacing(0)
        .width(Length::FillPortion(2))
        .height(Length::Fill);

        let main_content = row![
            left_panel_container,
            right_panel,
        ]
        .spacing(16)
        .height(Length::Fill);
        
        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.039, 0.039, 0.047).into()),
                    text_color: Some(Color::from_rgb(0.91, 0.91, 0.94)),
                    ..Default::default()
                }
            })
            .into()
    }
}
