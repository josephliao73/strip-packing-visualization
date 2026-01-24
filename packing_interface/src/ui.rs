use crate::config_parser::create_input;
use crate::editor::{build_code_panel, EditorState};
use crate::runner::{run_code, run_code_with_testcase, RunResult};
use iced::widget::{button, checkbox, column, container, row, text, text_input, text_editor, scrollable, slider};
use iced::{Element, Theme, Alignment, Length, Color, Font, time, Subscription};
use std::collections::HashSet;
use std::fmt::format;
use iced::widget::canvas::Canvas;
use crate::types::{AlgorithmOutput, BinCanvas, BottomPanelTab, CodeLanguage, Input, JsonInput, PackingApp, ParseOutput, Placement, Rectangle, RightPanelTab, Settings};
use std::time::Duration;
use ordered_float::OrderedFloat;
use rand::Rng;

impl Default for PackingApp {
    fn default() -> Self {
        Self {
            w_input: String::new(),
            n_input: String::new(),
            k_input: String::new(),
            autofile: false,
            rectangle_data: text_editor::Content::new(),
            error_message: None,
            algorithm_output: None,
            zoom: 1.0,
            visible_rects: 0,
            animating: false,
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
            selected_rects: HashSet::new(),
            active_tab: RightPanelTab::Visualization,
            current_testcase: None,
            testcase_message: None,
            code_editor_content: text_editor::Content::with_text(r#"
import packing_lib
import json
from typing import List, Tuple

class Packing:
    def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:
        """
        Pack rectangles into a bin of given width.

        Args:
            bin_width: Width of the bin
            rectangles: List of (width, height, quantity) tuples

        Returns:
            List of (x, y, width, height) placements for each rectangle
        """
        placements = [[1,1,2,2], [3,3,1,1]]
        # Your packing algorithm here
        return packing_lib.make_output(5, 6, placements)
"#),
            selected_language: CodeLanguage::Python,
            bottom_panel_visible: true,
            bottom_panel_tab: BottomPanelTab::Problems,
            code_errors: Vec::new(),
            code_output_json: None,
            settings: Settings {
                area_select_enabled: true,
                snap_to_rectangles_enabled: true,
            },
            settings_panel_visible: false,
            area_select_start: None,
            area_select_current: None,
            is_area_selecting: false,
        }
    }
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
                                        // Populate form fields from JSON
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
                                        self.error_message = Some(format!(
                                            "✓ Imported JSON: {} rectangle types, bin width {}",
                                            json_input.rectangle_list.len(),
                                            json_input.width_of_bin
                                        ));
                                    }
                                    Err(e) => {
                                        self.error_message = Some(format!("Error parsing JSON: {}", e));
                                    }
                                }
                            } else {
                                // Handle text files as before
                                self.rectangle_data = text_editor::Content::with_text(&contents);
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
                                    self.algorithm_output = Some(output);
                                    self.visible_rects = 0;
                                    self.animating = true;
                                    self.error_message = Some("✓ Successfully imported algorithm output".to_string());
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
            }
            Input::RectangleDragMove(x, y) => {
                if self.dragged_rect.is_some() {
                    let dx = x - self.last_mouse_x;
                    let dy = y - self.last_mouse_y;
                    self.dragged_rect_offset_x += dx;
                    self.dragged_rect_offset_y += dy;
                    self.last_mouse_x = x;
                    self.last_mouse_y = y;
                }
            }
            Input::RectangleDragEnd(is_inside, intersects, new_x, new_y) => {
                if let Some(dragged_idx) = self.dragged_rect && 
                    let Some((final_x, final_y)) = self.try_snap_rectangle(dragged_idx, new_x.into_inner(), new_y.into_inner(), is_inside, intersects) && 
                        let Some(output) = &mut self.algorithm_output && dragged_idx < output.placements.len() {
                                output.placements[dragged_idx].x = OrderedFloat(final_x);
                                output.placements[dragged_idx].y = OrderedFloat(final_y);
                                self.recalculate_bin_height();
                }
                self.dragged_rect = None;
                self.dragged_rect_offset_x = 0.0;
                self.dragged_rect_offset_y = 0.0;
            }
            Input::Tick => {
                if let Some(output) = &self.algorithm_output {
                    let total = output.placements.len();
                    if self.visible_rects < total {
                        self.visible_rects += 1;
                    } else {
                        self.animating = false;
                    }
                } else {
                    self.animating = false;
                }
            }
            Input::SnapAndAdjustHeight => {
                self.recalculate_bin_height();
            }
            Input::RightClickCanvas(clicked_rect) => {
                if let Some(idx) = clicked_rect {
                    let cur_rect: Placement = self.algorithm_output.as_ref().unwrap().placements[idx];
                    if !self.selected_rects.contains(&cur_rect) {
                        self.selected_rects.insert(self.algorithm_output.as_ref().unwrap().placements[idx]);
                    } else {
                        self.selected_rects.remove(&self.algorithm_output.as_ref().unwrap().placements[idx]);
                    }
                }
            }
            Input::TabSelected(tab) => {
                self.active_tab = tab;
            }
            Input::CodeEditorAction(action) => {
                use iced::widget::text_editor::{Action, Edit};

                match &action {
                    Action::Edit(Edit::Enter) => {
                        // Smart enter: maintain indentation and add extra if line ends with ':'
                        let text = self.code_editor_content.text();
                        let (line, _col) = self.code_editor_content.cursor_position();

                        if let Some(current_line) = text.lines().nth(line) {
                            // Get leading whitespace
                            let leading_ws: String = current_line
                                .chars()
                                .take_while(|c| c.is_whitespace())
                                .collect();

                            // Check if line ends with ':' (Python block start)
                            let trimmed = current_line.trim_end();
                            let extra_indent = if trimmed.ends_with(':') {
                                "    " // 4 spaces
                            } else {
                                ""
                            };

                            // Perform the enter action first
                            self.code_editor_content.perform(action);

                            // Then insert the indentation
                            let indent = format!("{}{}", leading_ws, extra_indent);
                            for c in indent.chars() {
                                self.code_editor_content.perform(Action::Edit(Edit::Insert(c)));
                            }
                        } else {
                            self.code_editor_content.perform(action);
                        }
                    }
                    Action::Edit(Edit::Insert('\t')) => {
                        // Replace tab with 4 spaces
                        for _ in 0..4 {
                            self.code_editor_content.perform(Action::Edit(Edit::Insert(' ')));
                        }
                    }
                    Action::Edit(Edit::Backspace) => {
                        // Smart backspace: delete 4 spaces at once if cursor is after indentation
                        let text = self.code_editor_content.text();
                        let (line, col) = self.code_editor_content.cursor_position();

                        let should_delete_tab = if col >= 4 {
                            if let Some(current_line) = text.lines().nth(line) {
                                // Get the characters before the cursor on this line
                                let chars_before: String = current_line.chars().take(col).collect();
                                // Check if the last 4 characters are all spaces
                                chars_before.ends_with("    ")
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if should_delete_tab {
                            // Delete 4 spaces (one tab-width)
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
            Input::RunCode => {
                if let Some(testcase) = &self.current_testcase {
                    let user_code = self.code_editor_content.text();
                    let result = run_code_with_testcase(self.selected_language, &user_code, testcase);
                    println!("{:?}", result);

                    match result {
                        RunResult::Success { output, raw_json } => {
                            self.algorithm_output = Some(output);
                            self.visible_rects = 0;
                            self.animating = true;
                            self.active_tab = RightPanelTab::Visualization;
                            self.error_message = Some("✓ Code executed successfully".to_string());
                            self.code_output_json = Some(raw_json);
                            self.code_errors.clear();
                            self.bottom_panel_tab = BottomPanelTab::Output;
                        }
                        RunResult::Error { errors } => {
                            self.error_message = Some(format!("Execution error:\n{}", errors.join("\n")));
                            self.code_errors = errors;
                            self.code_output_json = None;
                            self.bottom_panel_tab = BottomPanelTab::Problems;
                        }
                    }
                } else {
                    self.error_message = Some("No test case loaded. Import a test case first.".to_string());
                    self.bottom_panel_tab = BottomPanelTab::TestCases;
                }
            }
            Input::BottomPanelTabSelected(tab) => {
                self.bottom_panel_tab = tab;
            }
            Input::ToggleBottomPanel => {
                self.bottom_panel_visible = !self.bottom_panel_visible;
            }
            Input::SaveOutputToFile => {
                if let Some(json) = &self.code_output_json {
                    if let Some(path) = rfd::FileDialog::new()
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
            }
            Input::InsertTab => {
                use iced::widget::text_editor::{Action, Edit};
                // Insert 4 spaces (tab-width) into the code editor
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

                // Constants for random generation
                const MIN_BIN_WIDTH: i32 = 5;
                const MAX_BIN_WIDTH: i32 = 20;
                const MIN_HEIGHT: i32 = 1;
                const MAX_HEIGHT: i32 = 20;
                const MIN_TYPES: usize = 3;
                const MAX_TYPES: usize = 20;
                const MIN_TOTAL_RECTS: i32 = 10;
                const MAX_TOTAL_RECTS: i32 = 100;

                // Generate random bin width
                let bin_width = rng.random_range(MIN_BIN_WIDTH..=MAX_BIN_WIDTH);

                // Generate random number of rectangle types
                let num_types = rng.random_range(MIN_TYPES..=MAX_TYPES);

                // Generate unique rectangle types
                let mut rect_set: HashSet<(i32, i32)> = HashSet::new();
                let mut rectangles: Vec<Rectangle> = Vec::new();

                while rect_set.len() < num_types {
                    let width = rng.random_range(1..=bin_width);
                    let height = rng.random_range(MIN_HEIGHT..=MAX_HEIGHT);

                    if !rect_set.contains(&(width, height)) {
                        rect_set.insert((width, height));
                        rectangles.push(Rectangle {
                            width,
                            height,
                            quantity: 1, // Start with 1, will add more below
                        });
                    }
                }

                // Distribute remaining rectangles randomly
                let target_total = rng.random_range(MIN_TOTAL_RECTS..=MAX_TOTAL_RECTS);
                let mut current_total: i32 = rectangles.len() as i32;

                while current_total < target_total {
                    let idx = rng.random_range(0..rectangles.len());
                    let add = rng.random_range(1..=(target_total - current_total).min(5));
                    rectangles[idx].quantity += add;
                    current_total += add;
                }

                let total_rects: i32 = rectangles.iter().map(|r| r.quantity).sum();

                let testcase = JsonInput {
                    width_of_bin: bin_width,
                    number_of_rectangles: total_rects as usize,
                    number_of_types_of_rectangles: rectangles.len(),
                    autofill_option: false,
                    rectangle_list: rectangles,
                };

                let msg = format!(
                    "✓ Generated: {} rectangles, {} types, bin width {}",
                    testcase.number_of_rectangles,
                    testcase.number_of_types_of_rectangles,
                    testcase.width_of_bin
                );

                self.current_testcase = Some(testcase);
                self.testcase_message = Some(msg);
            }
            Input::ToggleAreaSelectEnabled(enabled) => {
                self.settings.area_select_enabled = enabled;
            }
            Input::ToggleSnapToRectangles(enabled) => {
                self.settings.snap_to_rectangles_enabled = enabled;
            }
            Input::ToggleSettingsPanel => {
                self.settings_panel_visible = !self.settings_panel_visible;
            }
            Input::AreaSelectStart(x, y) => {
                self.is_area_selecting = true;
                self.area_select_start = Some((x, y));
                self.area_select_current = Some((x, y));
            }
            Input::AreaSelectMove(x, y) => {
                if self.is_area_selecting {
                    self.area_select_current = Some((x, y));
                }
            }
            Input::AreaSelectEnd(selected_indices) => {
                // Add all rectangles within the selection area to selected_rects
                if let Some(output) = &self.algorithm_output {
                    for idx in selected_indices {
                        if idx < output.placements.len() {
                            let placement = output.placements[idx];
                            if !self.selected_rects.contains(&placement) {
                                self.selected_rects.insert(placement);
                            }
                        }
                    }
                }
                self.is_area_selecting = false;
                self.area_select_start = None;
                self.area_select_current = None;
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Input> {
        use iced::keyboard;

        let mut subscriptions = vec![];

        if self.animating {
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

        Subscription::batch(subscriptions)
    }

    fn try_snap_rectangle(&self, rect_idx: usize, new_x: f32, new_y: f32, is_inside: bool, intersects: bool) -> Option<(f32, f32)> {
        const SNAP_MARGIN_PERCENTAGE: f32 = 0.05;
        const RECT_SNAP_THRESHOLD: f32 = 5.0; // Snap threshold in bin units for rectangle-to-rectangle snapping

        if let Some(output) = &self.algorithm_output {
            if rect_idx >= output.placements.len() {
                return None;
            }

            let p = &output.placements[rect_idx];
            let rect_width = p.width as f32;
            let rect_height = p.height as f32;
            let snap_margin = rect_width.min(rect_height) * SNAP_MARGIN_PERCENTAGE;

            let bin_width = output.bin_width as f32;
            let bin_height = output.total_height as f32;

            let mut final_x = new_x;
            let mut final_y = new_y;

            // If position is valid, check for rectangle snapping
            if is_inside && !intersects {
                // Try snapping to other rectangles if enabled
                if self.settings.snap_to_rectangles_enabled {
                    let (snapped_x, snapped_y) = self.snap_to_rectangles(rect_idx, new_x, new_y, rect_width, rect_height, RECT_SNAP_THRESHOLD);
                    return Some((snapped_x, snapped_y));
                }
                return Some((new_x, new_y));
            }

            // Try bin edge snapping
            if !intersects && !is_inside {
                let mut snapped = false;

                if new_x < 0.0 && new_x.abs() <= snap_margin {
                    final_x = 0.0;
                    snapped = true;
                } else if new_x + rect_width > bin_width && (new_x + rect_width - bin_width) <= snap_margin {
                    final_x = bin_width - rect_width;
                    snapped = true;
                }

                if new_y < 0.0 && new_y.abs() <= snap_margin {
                    final_y = 0.0;
                    snapped = true;
                } else if new_y + rect_height > bin_height && (new_y + rect_height - bin_height) <= snap_margin {
                    final_y = bin_height - rect_height;
                    snapped = true;
                }

                if snapped {
                    return Some((final_x, final_y));
                }
            }

            None
        } else {
            None
        }
    }

    fn snap_to_rectangles(&self, rect_idx: usize, new_x: f32, new_y: f32, rect_width: f32, rect_height: f32, threshold: f32) -> (f32, f32) {
        let mut final_x = new_x;
        let mut final_y = new_y;
        let mut min_dist_x = threshold;
        let mut min_dist_y = threshold;

        if let Some(output) = &self.algorithm_output {
            // Edges of the dragged rectangle
            let left = new_x;
            let right = new_x + rect_width;
            let bottom = new_y;
            let top = new_y + rect_height;

            for (idx, other) in output.placements.iter().enumerate() {
                if idx == rect_idx {
                    continue;
                }

                let other_left = other.x.into_inner();
                let other_right = other_left + other.width as f32;
                let other_bottom = other.y.into_inner();
                let other_top = other_bottom + other.height as f32;

                // Check horizontal snapping (left edge to right edge, right edge to left edge, etc.)
                // Snap left to other's right
                let dist = (left - other_right).abs();
                if dist < min_dist_x {
                    min_dist_x = dist;
                    final_x = other_right;
                }

                // Snap right to other's left
                let dist = (right - other_left).abs();
                if dist < min_dist_x {
                    min_dist_x = dist;
                    final_x = other_left - rect_width;
                }

                // Snap left to other's left (align)
                let dist = (left - other_left).abs();
                if dist < min_dist_x {
                    min_dist_x = dist;
                    final_x = other_left;
                }

                // Snap right to other's right (align)
                let dist = (right - other_right).abs();
                if dist < min_dist_x {
                    min_dist_x = dist;
                    final_x = other_right - rect_width;
                }

                // Check vertical snapping
                // Snap bottom to other's top
                let dist = (bottom - other_top).abs();
                if dist < min_dist_y {
                    min_dist_y = dist;
                    final_y = other_top;
                }

                // Snap top to other's bottom
                let dist = (top - other_bottom).abs();
                if dist < min_dist_y {
                    min_dist_y = dist;
                    final_y = other_bottom - rect_height;
                }

                // Snap bottom to other's bottom (align)
                let dist = (bottom - other_bottom).abs();
                if dist < min_dist_y {
                    min_dist_y = dist;
                    final_y = other_bottom;
                }

                // Snap top to other's top (align)
                let dist = (top - other_top).abs();
                if dist < min_dist_y {
                    min_dist_y = dist;
                    final_y = other_top - rect_height;
                }
            }
        }

        (final_x, final_y)
    }

    fn recalculate_bin_height(&mut self) {
        if let Some(output) = &mut self.algorithm_output {
            let mut max_height = OrderedFloat(0.0);
            for placement in &output.placements {
                let top = placement.y + placement.height as f32;
                if top > max_height {
                    max_height = top;
                }
            }
            output.total_height = max_height.into_inner();
        }
    }

    fn parse_rectangles(&self) -> Result<ParseOutput, Vec<String>> {
        let text = self.rectangle_data.text();
        let mut rectangles = Vec::new();
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
        
        let text_content = self.rectangle_data.text();
        let total_lines = if text_content.is_empty() || text_content == "\n" { 
            1 
        } else { 
            text_content.chars().filter(|&c| c == '\n').count() + 1
        };

        let cursor = self.rectangle_data.cursor_position();
        let cursor_pos = cursor.0.min(text_content.len());
        let lines_before_cursor = if cursor_pos == 0 || text_content.is_empty() {
            1
        } else {
            text_content[..cursor_pos]
                .chars()
                .filter(|&c| c == '\n')
                .count() + 1
        };

        let line_info = text(format!("Line {} of {}", lines_before_cursor, total_lines))
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

// Settings gear button (always visible)
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

        // Settings popup panel
        let area_select_enabled = self.settings.area_select_enabled;
        let snap_to_rects_enabled = self.settings.snap_to_rectangles_enabled;

        let settings_popup: Element<'_, Input> = if self.settings_panel_visible {
            let area_select_checkbox = checkbox("Area Selection (Right-drag)", area_select_enabled)
                .on_toggle(Input::ToggleAreaSelectEnabled)
                .size(14)
                .font(ui_font)
                .text_size(11);

            let snap_checkbox = checkbox("Snap to Rectangles", snap_to_rects_enabled)
                .on_toggle(Input::ToggleSnapToRectangles)
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

        // Top-right corner with gear and popup
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

let visualization_content = if let Some(output) = &self.algorithm_output {
    let canvas = Canvas::new(BinCanvas {
            output,
            zoom: self.zoom,
            visible_count: self.visible_rects,
            pan_x: self.pan_x,
            pan_y: self.pan_y,
            hovered_rect: self.hovered_rect,
            is_panning: self.is_panning,
            dragged_rect: self.dragged_rect,
            dragged_rect_offset_x: self.dragged_rect_offset_x,
            dragged_rect_offset_y: self.dragged_rect_offset_y,
            animating: self.animating,
            selected_rects: &self.selected_rects,
            is_area_selecting: self.is_area_selecting,
            area_select_start: self.area_select_start,
            area_select_current: self.area_select_current,
            settings: &self.settings,
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
        
        let stats_display = if let Some(output) = &self.algorithm_output {
            let rect_count_text = text(format!("Rectangles: {}/{}", self.visible_rects, output.placements.len()))
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

        // Tab buttons
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

        // Code editor panel (built from editor module)
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
        };
        let code_panel_content = build_code_panel(&editor_state);

        // Right panel content based on active tab
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
