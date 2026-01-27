use iced::widget::canvas::{self};
use iced::widget::canvas::event::Event;
use iced::mouse;
use iced::{Color};
use crate::types::{Input, BinCanvas, HitGrid};
use iced::widget::canvas::{Frame, Path, Stroke, Fill};
use iced::{Point, Size};
use std::cell::Cell;
use std::time::{Duration, Instant};
use ordered_float::OrderedFloat;


impl<'a> BinCanvas<'a> {
    fn find_rectangle_at_point(&self, x: f32, y: f32, bounds: &iced::Rectangle, scale: f32, origin_x: f32, origin_y: f32, bin_w_units: f32, bin_h_units: f32) -> Option<usize> {
        let total = self.output.placements.len();
        let count = self.visible_count.min(total);

        let local_x = x - bounds.x;
        let local_y = y - bounds.y;

        let draw_w = bin_w_units * scale;
        let draw_h = bin_h_units * scale;

        if local_x < origin_x || local_x > origin_x + draw_w || local_y < origin_y || local_y > origin_y + draw_h {
            return None;
        }

        let candidates = self.candidates_for_point(local_x, local_y, scale, origin_x, origin_y, bin_w_units, bin_h_units, count);
        for idx in candidates.into_iter().rev() {
            let p = &self.output.placements[idx];
            let w = p.width as f32 * scale;
            let h = p.height as f32 * scale;
            let rect_x = origin_x + p.x.into_inner() * scale;
            let rect_y = origin_y + (bin_h_units - (p.y.into_inner() + p.height as f32)) * scale;

            if local_x >= rect_x && local_x <= rect_x + w && local_y >= rect_y && local_y <= rect_y + h {
                return Some(idx);
            }
        }
        None
    }

    fn candidates_for_point(&self, local_x: f32, local_y: f32, scale: f32, origin_x: f32, origin_y: f32, bin_w_units: f32, bin_h_units: f32, count: usize) -> Vec<usize> {
        let Some(grid) = self.hit_grid else {
            return (0..count).collect();
        };

        let bin_x = (local_x - origin_x) / scale;
        let bin_y = bin_h_units - (local_y - origin_y) / scale;

        if bin_x < 0.0 || bin_y < 0.0 || bin_x > bin_w_units || bin_y > bin_h_units {
            return Vec::new();
        }

        self.candidates_from_grid(grid, bin_x, bin_y, bin_x, bin_y, count)
    }

    fn candidates_from_grid(&self, grid: &HitGrid, x0: f32, y0: f32, x1: f32, y1: f32, count: usize) -> Vec<usize> {
        let min_col = (x0 / grid.cell_size).floor().max(0.0) as usize;
        let max_col = (x1 / grid.cell_size).floor().min((grid.cols - 1) as f32) as usize;
        let min_row = (y0 / grid.cell_size).floor().max(0.0) as usize;
        let max_row = (y1 / grid.cell_size).floor().min((grid.rows - 1) as f32) as usize;

        let mut out = Vec::new();
        for row in min_row..=max_row {
            let row_offset = row * grid.cols;
            for col in min_col..=max_col {
                for &idx in &grid.cells[row_offset + col] {
                    if idx >= count {
                        continue;
                    }
                    if !out.contains(&idx) {
                        out.push(idx);
                    }
                }
            }
        }
        if out.is_empty() {
            (0..count).collect()
        } else {
            out
        }
    }

    pub fn find_region_at_point(&self, x: f32, y: f32, bounds: &iced::Rectangle, scale: f32, origin_x: f32, origin_y: f32, bin_h_units: f32) -> Option<usize> {
        let local_x = x - bounds.x;
        let local_y = y - bounds.y;

        for (idx, region) in self.selection_regions.iter().enumerate().rev() {
            // Convert bin coordinates to screen coordinates
            let screen_x = origin_x + region.bin_x * scale;
            let screen_y = origin_y + (bin_h_units - region.bin_y - region.bin_h) * scale;
            let screen_w = region.bin_w * scale;
            let screen_h = region.bin_h * scale;

            if local_x >= screen_x && local_x <= screen_x + screen_w &&
               local_y >= screen_y && local_y <= screen_y + screen_h {
                return Some(idx);
            }
        }
        None
    }
}

#[derive(Clone, Copy, PartialEq)]
struct CacheKey {
    output_revision: u64,
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
    visible_count: usize,
    dragged_rect: Option<usize>,
}

pub struct CanvasState {
    base_cache: canvas::Cache,
    last_key: Cell<Option<CacheKey>>,
    last_hover_time: Option<Instant>,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            base_cache: canvas::Cache::new(),
            last_key: Cell::new(None),
            last_hover_time: None,
        }
    }
}

impl<'a> iced::widget::canvas::Program<Input> for BinCanvas<'a> {
    type State = CanvasState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {

        let bin_w_units = self.output.bin_width as f32;
        let bin_h_units = self.output.total_height;

        if bin_w_units <= 0.0 || bin_h_units <= 0.0 {
            let frame = Frame::new(renderer, bounds.size());
            return vec![frame.into_geometry()];
        }

        let fit_x = bounds.width / bin_w_units;
        let fit_y = bounds.height / bin_h_units;
        let base_scale = fit_x.min(fit_y);
        let scale = base_scale * self.zoom;

        let draw_w = bin_w_units * scale;
        let draw_h = bin_h_units * scale;

        let origin_x = (bounds.width - draw_w) / 2.0 + self.pan_x;
        let origin_y = (bounds.height - draw_h) / 2.0 + self.pan_y;

        let total = self.output.placements.len();
        let count = self.visible_count.min(total);

        let cache_key = CacheKey {
            output_revision: self.output_revision,
            zoom: self.zoom,
            pan_x: self.pan_x,
            pan_y: self.pan_y,
            visible_count: count,
            dragged_rect: self.dragged_rect,
        };

        if state.last_key.get() != Some(cache_key) {
            state.base_cache.clear();
            state.last_key.set(Some(cache_key));
        }

        let base_geometry = state.base_cache.draw(renderer, bounds.size(), |frame| {
            // Draw bin border
            let bin_path = Path::rectangle(
                Point::new(origin_x, origin_y),
                Size::new(draw_w, draw_h),
            );
            frame.stroke(&bin_path, Stroke::default()
                .with_color(Color::from_rgb(1.0, 0.6, 0.15))
                .with_width(2.0));

            let stroke_border = Stroke::default().with_color(Color::from_rgba(0.0, 0.0, 0.0, 0.4)).with_width(1.0);

            for (idx, p) in self.output.placements.iter().enumerate().take(count) {
                if self.dragged_rect == Some(idx) {
                    continue;
                }

                let w = p.width as f32 * scale;
                let h = p.height as f32 * scale;
                let x_px = origin_x + p.x.into_inner() * scale;
                let y_px = origin_y + (bin_h_units - (p.y.into_inner() + p.height as f32)) * scale;

                if x_px + w < 0.0 || x_px > bounds.width || y_px + h < 0.0 || y_px > bounds.height {
                    continue;
                }

                let rect_path = Path::rectangle(Point::new(x_px, y_px), Size::new(w, h));
                let color = color_from_dimensions(p.width, p.height);
                frame.fill(&rect_path, Fill::from(color));
                frame.stroke(&rect_path, stroke_border.clone());
            }
        });

        let mut frame = Frame::new(renderer, bounds.size());

        let stroke_selected = Stroke::default().with_color(Color::from_rgb(0.2, 0.9, 1.0)).with_width(2.5);
        let stroke_hovered = Stroke::default().with_color(Color::from_rgb(1.0, 1.0, 1.0)).with_width(2.0);

        if !self.selected_rects.is_empty() {
            for idx in self.selected_rects.iter().copied() {
                if idx >= count || self.dragged_rect == Some(idx) {
                    continue;
                }
                let p = &self.output.placements[idx];
                let w = p.width as f32 * scale;
                let h = p.height as f32 * scale;
                let x_px = origin_x + p.x.into_inner() * scale;
                let y_px = origin_y + (bin_h_units - (p.y.into_inner() + p.height as f32)) * scale;
                if x_px + w < 0.0 || x_px > bounds.width || y_px + h < 0.0 || y_px > bounds.height {
                    continue;
                }
                let rect_path = Path::rectangle(Point::new(x_px, y_px), Size::new(w, h));
                frame.stroke(&rect_path, stroke_selected.clone());
            }
        }

        if let Some(hovered_idx) = self.hovered_rect && hovered_idx < count && self.dragged_rect != Some(hovered_idx) && !self.selected_rects.contains(&hovered_idx) {
            let p = &self.output.placements[hovered_idx];
            let w = p.width as f32 * scale;
            let h = p.height as f32 * scale;
            let x_px = origin_x + p.x.into_inner() * scale;
            let y_px = origin_y + (bin_h_units - (p.y.into_inner() + p.height as f32)) * scale;
            if x_px + w >= 0.0 && x_px <= bounds.width && y_px + h >= 0.0 && y_px <= bounds.height {
                let rect_path = Path::rectangle(Point::new(x_px, y_px), Size::new(w, h));
                frame.stroke(&rect_path, stroke_hovered.clone());
            }
        
        }

        if let Some(dragged_idx) = self.dragged_rect && dragged_idx < count {
                let p = &self.output.placements[dragged_idx];
                let w = p.width as f32 * scale;
                let h = p.height as f32 * scale;
                let x_px = origin_x + p.x.into_inner() * scale + self.dragged_rect_offset_x;
                let y_px = origin_y
                    + (bin_h_units - (p.y.into_inner() + p.height as f32)) * scale + self.dragged_rect_offset_y;


                if x_px + w >= 0.0 && x_px <= bounds.width && y_px + h >= 0.0 && y_px <= bounds.height {
                    let rect_path = Path::rectangle(Point::new(x_px, y_px), Size::new(w, h));
                    let color = color_from_dimensions(p.width, p.height);
                    frame.fill(&rect_path, Fill::from(color));

                    let bin_rect = iced::Rectangle {
                        x: origin_x,
                        y: origin_y,
                        width: bin_w_units * scale,
                        height: bin_h_units * scale,
                    };

                let is_inside = is_inside(&bin_rect, x_px, y_px, w, h);
                let mut intersects = false;

                let dragged_bin_x = (x_px - origin_x) / scale;
                let dragged_bin_y = bin_h_units - (y_px - origin_y) / scale - (p.height as f32);
                let dragged_x1 = dragged_bin_x + p.width as f32;
                let dragged_y1 = dragged_bin_y + p.height as f32;

                let candidates = if let Some(grid) = self.hit_grid {
                    self.candidates_from_grid(grid, dragged_bin_x, dragged_bin_y, dragged_x1, dragged_y1, count)
                } else {
                    (0..count).collect()
                };

                for idx in candidates {
                    if idx == dragged_idx {
                        continue;
                    }
                    let other = &self.output.placements[idx];
                    let other_width = other.width as f32 * scale;
                    let other_height = other.height as f32 * scale;
                    let other_x = origin_x + other.x.into_inner() * scale;
                    let other_y = origin_y + (bin_h_units - (other.y.into_inner() + other.height as f32)) * scale;

                    intersects = !(x_px + w <= other_x ||
                                     x_px >= other_x + other_width ||
                                     y_px + h <= other_y ||
                                     y_px >= other_y + other_height);

                    if intersects {
                        break;
                    }
                }

                    let stroke_color = if is_inside && !intersects {
                        Color::from_rgb(0.3, 1.0, 0.4)
                    } else {
                        Color::from_rgb(1.0, 0.35, 0.35)
                    };
                    frame.stroke(&rect_path, Stroke::default().with_color(stroke_color).with_width(2.0));

                    if let Some((snap_x, snap_y)) = self.snap_preview {
                        let rect_h = p.height as f32;
                        let preview_x = origin_x + snap_x * scale;
                        let preview_y = origin_y + (bin_h_units - snap_y - rect_h) * scale;
                        let preview_path = Path::rectangle(
                            Point::new(preview_x, preview_y),
                            Size::new(w, h)
                        );
                        frame.stroke(
                            &preview_path,
                            Stroke::default()
                                .with_color(Color::from_rgba(0.3, 0.9, 1.0, 0.7))
                                .with_width(2.0)
                        );
                    }
                }

            }

        for region in self.selection_regions.iter() {
            let sel_x = origin_x + region.bin_x * scale;
            let sel_y = origin_y + (bin_h_units - region.bin_y - region.bin_h) * scale;
            let sel_w = region.bin_w * scale;
            let sel_h = region.bin_h * scale;

            let sel_path = Path::rectangle(Point::new(sel_x, sel_y), Size::new(sel_w, sel_h));

            let (fill_color, stroke_color) = region_color(region.is_inherited);
            frame.fill(&sel_path, Fill::from(fill_color));
            frame.stroke(&sel_path, Stroke::default().with_color(stroke_color).with_width(1.5));
        }

        if self.is_area_selecting && let Some((start_x, start_y)) = self.area_select_start && let Some((current_x, current_y)) = self.area_select_current {
            let local_start_x = start_x - bounds.x;
            let local_start_y = start_y - bounds.y;
            let local_current_x = current_x - bounds.x;
            let local_current_y = current_y - bounds.y;

            let sel_x = local_start_x.min(local_current_x);
            let sel_y = local_start_y.min(local_current_y);
            let sel_w = (local_current_x - local_start_x).abs();
            let sel_h = (local_current_y - local_start_y).abs();

            let sel_path = Path::rectangle(Point::new(sel_x, sel_y), Size::new(sel_w, sel_h));
            frame.fill(&sel_path, Fill::from(Color::from_rgba(0.3, 0.5, 0.9, 0.15)));
            frame.stroke(&sel_path, Stroke::default().with_color(Color::from_rgb(0.4, 0.65, 1.0)).with_width(1.5));
        }

        vec![base_geometry, frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Input>) {
        let bin_w_units = self.output.bin_width as f32;
        let bin_h_units = self.output.total_height;

        if bin_w_units <= 0.0 || bin_h_units <= 0.0 {
            return (canvas::event::Status::Ignored, None);
        }

        let fit_x = bounds.width / bin_w_units;
        let fit_y = bounds.height / bin_h_units;
        let base_scale = fit_x.min(fit_y);
        let scale = base_scale * self.zoom;

        let draw_w = bin_w_units * scale;
        let draw_h = bin_h_units * scale;

        let origin_x = (bounds.width - draw_w) / 2.0 + self.pan_x;
        let origin_y = (bounds.height - draw_h) / 2.0 + self.pan_y;
        let total = self.output.placements.len();
        let count = self.visible_count.min(total);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if let Some(position) = cursor.position() {
                    if let Some(region_idx) = self.find_region_at_point(position.x, position.y, &bounds, scale, origin_x, origin_y, bin_h_units) {
                        let local_x = position.x - bounds.x;
                        let local_y = position.y - bounds.y;
                        (canvas::event::Status::Captured, Some(Input::ShowRegionContextMenu(region_idx, local_x, local_y)))
                    } else if self.hovered_rect.is_some() {
                        // Clicking on a rectangle - toggle selection
                        (canvas::event::Status::Captured, Some(Input::RightClickCanvas(self.hovered_rect)))
                    } else if self.settings.area_select_enabled {
                        // Only start area selection if click is INSIDE the bin rectangle
                        let local_x = position.x - bounds.x;
                        let local_y = position.y - bounds.y;
                        let bin_rect = iced::Rectangle {
                            x: origin_x,
                            y: origin_y,
                            width: bin_w_units * scale,
                            height: bin_h_units * scale,
                        };
                        if bin_rect.contains(Point::new(local_x, local_y)) {
                            (canvas::event::Status::Captured, Some(Input::AreaSelectStart(position.x, position.y)))
                        } else {
                            (canvas::event::Status::Ignored, None)
                        }
                    } else {
                        (canvas::event::Status::Ignored, None)
                    }
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) => {
                if self.is_area_selecting && let Some((start_x, start_y)) = self.area_select_start && let Some((end_x, end_y)) = self.area_select_current {
                    let local_start_x = start_x - bounds.x;
                    let local_start_y = start_y - bounds.y;
                    let local_end_x = end_x - bounds.x;
                    let local_end_y = end_y - bounds.y;

                    // Convert to bin coordinates
                    let bin_x1 = (local_start_x - origin_x) / scale;
                    let bin_y1 = bin_h_units - (local_start_y - origin_y) / scale;
                    let bin_x2 = (local_end_x - origin_x) / scale;
                    let bin_y2 = bin_h_units - (local_end_y - origin_y) / scale;

                    // Get min/max corners
                    let raw_x = bin_x1.min(bin_x2);
                    let raw_y = bin_y1.min(bin_y2);
                    let raw_x2 = bin_x1.max(bin_x2);
                    let raw_y2 = bin_y1.max(bin_y2);

                    // Clamp to bin boundaries
                    let clamped_x = raw_x.max(0.0);
                    let clamped_y = raw_y.max(0.0);
                    let clamped_x2 = raw_x2.min(bin_w_units);
                    let clamped_y2 = raw_y2.min(bin_h_units);

                    // Calculate final width/height after clamping
                    let bin_x = clamped_x;
                    let bin_y = clamped_y;
                    let bin_w = (clamped_x2 - clamped_x).max(0.0);
                    let bin_h = (clamped_y2 - clamped_y).max(0.0);

                    (canvas::event::Status::Captured, Some(Input::AreaSelectEnd(Vec::new(), bin_x, bin_y, bin_w, bin_h)))
                } else if self.is_area_selecting {
                    (canvas::event::Status::Captured, Some(Input::AreaSelectEnd(Vec::new(), 0.0, 0.0, 0.0, 0.0)))
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if let Some(position) = cursor.position() {
                    let canvas_rect = iced::Rectangle {
                        x: bounds.x,
                        y: bounds.y,
                        width: bounds.width,
                        height: bounds.height,
                    };

                    if canvas_rect.contains(position) {
                        let dy = match delta {
                            mouse::ScrollDelta::Lines { y, .. } => y,
                            mouse::ScrollDelta::Pixels { y, .. } => y / 100.0,
                        };

                        let factor = if dy > 0.0 { 1.1 } else { 0.9 };
                        return (canvas::event::Status::Captured, Some(Input::ZoomChanged(factor)));
                    }
                }
                (canvas::event::Status::Ignored, None)
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position() {
                    let canvas_rect = iced::Rectangle {
                        x: bounds.x,
                        y: bounds.y,
                        width: bounds.width,
                        height: bounds.height,
                    };

                    if !canvas_rect.contains(position) {
                        return (canvas::event::Status::Ignored, None);
                    }

                    let draw_w = self.output.bin_width as f32 * (base_scale * self.zoom);
                    let draw_h = self.output.total_height * (base_scale * self.zoom);
                    let bin_origin_x = bounds.x + (bounds.width - draw_w) / 2.0 + self.pan_x;
                    let bin_origin_y = bounds.y + (bounds.height - draw_h) / 2.0 + self.pan_y;

                    let bin_rect = iced::Rectangle {
                        x: bin_origin_x,
                        y: bin_origin_y,
                        width: draw_w,
                        height: draw_h,
                    };

                    if !bin_rect.contains(position) {
                        (canvas::event::Status::Captured, Some(Input::PanStart(position.x, position.y)))
                    } else {
                if !self.animating && let Some(rect_idx) = self.find_rectangle_at_point(position.x, position.y, &bounds, scale, origin_x, origin_y, bin_w_units, bin_h_units) {
                        return (canvas::event::Status::Captured, Some(Input::RectangleDragStart(rect_idx, position.x, position.y)));
                    }
                        
                        (canvas::event::Status::Ignored, None)
                    }
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.is_panning {
                    (canvas::event::Status::Captured, Some(Input::PanEnd))
                } else if let Some(dragged_idx) = self.dragged_rect {
                    if dragged_idx < self.output.placements.len() {
                        let p = &self.output.placements[dragged_idx];
                        let w = p.width as f32 * scale;
                        let h = p.height as f32 * scale;
                        let x_px = origin_x + p.x.into_inner() * scale + self.dragged_rect_offset_x;
                        let y_px = origin_y + (bin_h_units - (p.y.into_inner() + p.height as f32)) * scale + self.dragged_rect_offset_y;

                        let bin_rect = iced::Rectangle {
                            x: origin_x,
                            y: origin_y,
                            width: bin_w_units * scale,
                            height: bin_h_units * scale,
                        };

                        let is_inside = is_inside(&bin_rect, x_px, y_px, w, h); 

                        let mut intersects = false;
                        let dragged_bin_x = (x_px - origin_x) / scale;
                        let dragged_bin_y = bin_h_units - (y_px - origin_y) / scale - (p.height as f32);
                        let dragged_x1 = dragged_bin_x + p.width as f32;
                        let dragged_y1 = dragged_bin_y + p.height as f32;

                        let candidates = if let Some(grid) = self.hit_grid {
                            self.candidates_from_grid(grid, dragged_bin_x, dragged_bin_y, dragged_x1, dragged_y1, count)
                        } else {
                            (0..count).collect()
                        };

                        for idx in candidates {
                            if idx == dragged_idx {
                                continue;
                            }

                            let other = &self.output.placements[idx];
                            let other_width = other.width as f32 * scale;
                            let other_height = other.height as f32 * scale;
                            let other_x = origin_x + other.x.into_inner() * scale;
                            let other_y = origin_y + (bin_h_units - (other.y.into_inner() + other.height as f32)) * scale;

                            intersects = !(x_px + w <= other_x ||
                                         x_px >= other_x + other_width ||
                                         y_px + h <= other_y ||
                                         y_px >= other_y + other_height);

                            if intersects {
                                break;
                            }
                        }

                        let new_x = p.x + (self.dragged_rect_offset_x / scale);
                        let new_y = p.y - (self.dragged_rect_offset_y / scale);

                        (canvas::event::Status::Captured, Some(Input::RectangleDragEnd(is_inside, intersects, *OrderedFloat(new_x), *OrderedFloat(new_y))))
                    } else {
                        (canvas::event::Status::Ignored, None)
                    }
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if self.is_area_selecting {
                    (canvas::event::Status::Captured, Some(Input::AreaSelectMove(position.x, position.y)))
                } else if self.is_panning {
                    (canvas::event::Status::Captured, Some(Input::PanMove(position.x, position.y)))
                } else if self.dragged_rect.is_some() {
                    (canvas::event::Status::Captured, Some(Input::RectangleDragMove(position.x, position.y, scale)))
                } else {
                    let now = Instant::now();
                    if let Some(last) = state.last_hover_time && now.duration_since(last) < Duration::from_millis(12) {
                        return (canvas::event::Status::Ignored, None);
                    }
                    state.last_hover_time = Some(now);
                    let hovered = self.find_rectangle_at_point(position.x, position.y, &bounds, scale, origin_x, origin_y, bin_w_units, bin_h_units);
                    if hovered == self.hovered_rect {
                        (canvas::event::Status::Ignored, None)
                    } else {
                        (canvas::event::Status::Captured, Some(Input::RectangleHovered(hovered)))
                    }
                }
            }
            _ => (canvas::event::Status::Ignored, None)
        }
    }
}

fn color_from_dimensions(x: i32, y: i32) -> Color {
    let mut h = 14695981039346656037u64;
    for v in [x as u32, y as u32] {
        h ^= v as u64;
        h = h.wrapping_mul(1099511628211);
    }

    let hue = ((h & 0xFFFF) as f32 / 65535.0) * 360.0;

    let saturation = 0.55 + ((h >> 16) & 0xFF) as f32 / 255.0 * 0.25; // 0.55-0.80
    let lightness = 0.55 + ((h >> 24) & 0xFF) as f32 / 255.0 * 0.20;  // 0.55-0.75

    let c = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let x_val = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = lightness - c / 2.0;

    let (r, g, b) = if hue < 60.0 {
        (c, x_val, 0.0)
    } else if hue < 120.0 {
        (x_val, c, 0.0)
    } else if hue < 180.0 {
        (0.0, c, x_val)
    } else if hue < 240.0 {
        (0.0, x_val, c)
    } else if hue < 300.0 {
        (x_val, 0.0, c)
    } else {
        (c, 0.0, x_val)
    };

    Color::from_rgb(r + m, g + m, b + m)
}

fn region_color(is_inherited: bool) -> (Color, Color) {
    if is_inherited {
        let stroke = Color::from_rgb(0.3, 0.5, 0.9);
        let fill = Color::from_rgba(0.3, 0.5, 0.9, 0.15);
        (fill, stroke)
    } else {
        let stroke = Color::from_rgb(0.95, 0.6, 0.2);
        let fill = Color::from_rgba(0.95, 0.6, 0.2, 0.15);
        (fill, stroke)
    }
}

fn is_inside(bin_rect: &iced::Rectangle, x:f32, y:f32, w:f32, h:f32) -> bool {
    bin_rect.contains(Point {x, y})
        && bin_rect.contains(Point {x: x + w, y})
        && bin_rect.contains(Point {x, y: y + h})
        && bin_rect.contains(Point {x: x + w, y: y + h})
}
