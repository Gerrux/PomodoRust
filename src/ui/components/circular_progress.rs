//! Circular progress indicator with gradient

use egui::{vec2, Color32, Pos2, Rect, Stroke, Ui};
use std::f32::consts::{PI, TAU};

use crate::ui::theme::Theme;

/// A circular progress ring with gradient and animations
pub struct CircularProgress {
    /// Progress value (0.0 to 1.0)
    progress: f32,
    /// Radius of the progress ring
    radius: f32,
    /// Thickness of the ring
    thickness: f32,
    /// Start color of gradient
    start_color: Color32,
    /// End color of gradient
    end_color: Color32,
    /// Background ring color
    bg_color: Color32,
    /// Pulse intensity (0.0 to 1.0)
    pulse: f32,
}

impl CircularProgress {
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            radius: 100.0,
            thickness: 8.0,
            start_color: Color32::from_rgb(59, 130, 246),  // blue-500
            end_color: Color32::from_rgb(139, 92, 246),    // violet-500
            bg_color: Color32::from_rgb(39, 39, 42),       // zinc-800
            pulse: 0.0,
        }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn with_colors(mut self, start: Color32, end: Color32) -> Self {
        self.start_color = start;
        self.end_color = end;
        self
    }

    pub fn with_bg_color(mut self, color: Color32) -> Self {
        self.bg_color = color;
        self
    }

    pub fn with_pulse(mut self, pulse: f32) -> Self {
        self.pulse = pulse.clamp(0.0, 1.0);
        self
    }

    pub fn show(&self, ui: &mut Ui, center_content: impl FnOnce(&mut Ui)) {
        let size = vec2(self.radius * 2.0 + self.thickness, self.radius * 2.0 + self.thickness);
        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());

        let center = rect.center();
        let outer_radius = self.radius;
        let inner_radius = self.radius - self.thickness;

        // Draw background ring
        self.draw_ring(ui, center, outer_radius, inner_radius, 1.0, self.bg_color, self.bg_color);

        // Draw progress ring with gradient
        if self.progress > 0.0 {
            self.draw_progress_ring(ui, center, outer_radius, inner_radius);
        }

        // Pulse glow effect
        if self.pulse > 0.0 {
            let glow_alpha = (self.pulse * 80.0) as u8;
            let glow_color = Theme::with_alpha(self.start_color, glow_alpha);
            let glow_radius = outer_radius + 4.0 + self.pulse * 8.0;

            ui.painter().circle_stroke(
                center,
                glow_radius,
                Stroke::new(2.0, glow_color),
            );
        }

        // Center content area
        let content_rect = Rect::from_center_size(center, vec2(inner_radius * 1.6, inner_radius * 1.6));
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content_rect), |ui| {
            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                center_content(ui);
            });
        });
    }

    fn draw_ring(
        &self,
        ui: &mut Ui,
        center: Pos2,
        outer_r: f32,
        inner_r: f32,
        progress: f32,
        start_color: Color32,
        end_color: Color32,
    ) {
        if progress <= 0.0 {
            return;
        }

        // More segments for smoother circle (anti-aliasing effect)
        let segments = ((outer_r * 2.0) as usize).clamp(72, 180);
        let filled_segments = ((segments as f32 * progress) as usize).max(1);

        let start_angle = -PI / 2.0;
        let angle_per_segment = TAU / segments as f32;

        // Batch into single mesh
        let mut mesh = egui::Mesh::default();
        mesh.vertices.reserve(filled_segments * 4);
        mesh.indices.reserve(filled_segments * 6);

        for i in 0..filled_segments {
            let t = i as f32 / segments as f32;
            let angle1 = start_angle + angle_per_segment * i as f32;
            let angle2 = start_angle + angle_per_segment * (i + 1) as f32;

            let color = Theme::lerp_color(start_color, end_color, t);

            let (sin1, cos1) = angle1.sin_cos();
            let (sin2, cos2) = angle2.sin_cos();

            let outer1 = Pos2::new(center.x + outer_r * cos1, center.y + outer_r * sin1);
            let outer2 = Pos2::new(center.x + outer_r * cos2, center.y + outer_r * sin2);
            let inner1 = Pos2::new(center.x + inner_r * cos1, center.y + inner_r * sin1);
            let inner2 = Pos2::new(center.x + inner_r * cos2, center.y + inner_r * sin2);

            let idx_base = mesh.vertices.len() as u32;
            mesh.vertices.extend_from_slice(&[
                egui::epaint::Vertex { pos: outer1, uv: egui::epaint::WHITE_UV, color },
                egui::epaint::Vertex { pos: outer2, uv: egui::epaint::WHITE_UV, color },
                egui::epaint::Vertex { pos: inner2, uv: egui::epaint::WHITE_UV, color },
                egui::epaint::Vertex { pos: inner1, uv: egui::epaint::WHITE_UV, color },
            ]);
            mesh.indices.extend_from_slice(&[
                idx_base, idx_base + 1, idx_base + 2,
                idx_base, idx_base + 2, idx_base + 3,
            ]);
        }

        ui.painter().add(egui::Shape::mesh(mesh));
    }

    fn draw_progress_ring(&self, ui: &mut Ui, center: Pos2, outer_r: f32, inner_r: f32) {
        // More segments for smoother circle (anti-aliasing effect)
        let segments = ((outer_r * 2.0) as usize).clamp(72, 180);
        let filled_segments = ((segments as f32 * self.progress) as usize).max(1);

        if filled_segments == 0 {
            return;
        }

        let start_angle = -PI / 2.0;
        let angle_per_segment = TAU / segments as f32;

        // Batch all segments into a single mesh for performance
        let mut mesh = egui::Mesh::default();
        mesh.vertices.reserve(filled_segments * 4);
        mesh.indices.reserve(filled_segments * 6);

        for i in 0..filled_segments {
            let t = i as f32 / filled_segments.max(1) as f32;
            let angle1 = start_angle + angle_per_segment * i as f32;
            let angle2 = start_angle + angle_per_segment * (i + 1) as f32;

            let color = Theme::lerp_color(self.start_color, self.end_color, t);

            // Precompute trig for both angles
            let (sin1, cos1) = angle1.sin_cos();
            let (sin2, cos2) = angle2.sin_cos();

            let outer1 = Pos2::new(center.x + outer_r * cos1, center.y + outer_r * sin1);
            let outer2 = Pos2::new(center.x + outer_r * cos2, center.y + outer_r * sin2);
            let inner1 = Pos2::new(center.x + inner_r * cos1, center.y + inner_r * sin1);
            let inner2 = Pos2::new(center.x + inner_r * cos2, center.y + inner_r * sin2);

            let idx_base = mesh.vertices.len() as u32;

            mesh.vertices.extend_from_slice(&[
                egui::epaint::Vertex { pos: outer1, uv: egui::epaint::WHITE_UV, color },
                egui::epaint::Vertex { pos: outer2, uv: egui::epaint::WHITE_UV, color },
                egui::epaint::Vertex { pos: inner2, uv: egui::epaint::WHITE_UV, color },
                egui::epaint::Vertex { pos: inner1, uv: egui::epaint::WHITE_UV, color },
            ]);

            mesh.indices.extend_from_slice(&[
                idx_base,
                idx_base + 1,
                idx_base + 2,
                idx_base,
                idx_base + 2,
                idx_base + 3,
            ]);
        }

        ui.painter().add(egui::Shape::mesh(mesh));

        // Draw end cap (rounded)
        if self.progress > 0.0 && self.progress < 1.0 {
            let end_angle = start_angle + TAU * self.progress;
            let mid_r = (outer_r + inner_r) / 2.0;
            let (sin_e, cos_e) = end_angle.sin_cos();
            let cap_center = Pos2::new(center.x + mid_r * cos_e, center.y + mid_r * sin_e);
            ui.painter().circle_filled(cap_center, self.thickness / 2.0, self.end_color);
        }

        // Draw start cap
        if self.progress > 0.0 {
            let mid_r = (outer_r + inner_r) / 2.0;
            let (sin_s, cos_s) = start_angle.sin_cos();
            let cap_center = Pos2::new(center.x + mid_r * cos_s, center.y + mid_r * sin_s);
            ui.painter().circle_filled(cap_center, self.thickness / 2.0, self.start_color);
        }
    }
}
