//! Lucide-style vector icons for egui
//!
//! Stroke-based icons with consistent 1.5px stroke at 24px base size.
//! All icons are drawn programmatically using egui's painter API.

use egui::{vec2, Color32, Pos2, Rect, Stroke, Ui};

/// Icon identifier enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    // Playback controls
    Play,
    Pause,
    Stop,
    SkipForward,
    SkipBack,
    RotateCcw,

    // Navigation
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    ChevronDown,
    ArrowLeft,

    // UI controls
    X,
    Plus,
    Minus,
    Check,

    // Window controls
    Minimize,
    Maximize,
    Restore,

    // App-specific
    Settings,
    LayoutDashboard,
    Timer,
    Clock,
    Bell,
    BellOff,
    Volume2,
    VolumeX,

    // Status
    Coffee,
    Target,
    Flame,
    Calendar,
    BarChart3,
    TrendingUp,

    // General
    Sun,
    Moon,
    Palette,
    Zap,
    Download,
    Pin,
    PinOff,
}

/// Draw an icon at the specified rectangle
pub fn draw_icon(ui: &mut Ui, icon: Icon, rect: Rect, color: Color32) {
    let painter = ui.painter();
    let center = rect.center();
    let size = rect.width().min(rect.height());

    // Scale stroke width based on icon size (1.5px at 24px)
    let stroke_width = (size / 24.0 * 1.5).clamp(1.0, 3.0);
    let stroke = Stroke::new(stroke_width, color);

    // Helper to scale coordinates from 24x24 base to actual size
    let scale = |x: f32, y: f32| -> Pos2 {
        Pos2::new(
            center.x + (x - 12.0) * (size / 24.0),
            center.y + (y - 12.0) * (size / 24.0),
        )
    };

    match icon {
        Icon::Play => {
            // Triangle pointing right
            let points = [
                scale(8.0, 5.0),
                scale(19.0, 12.0),
                scale(8.0, 19.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                points.to_vec(),
                color,
                Stroke::NONE,
            ));
        }

        Icon::Pause => {
            // Two vertical bars
            painter.line_segment([scale(8.0, 6.0), scale(8.0, 18.0)], Stroke::new(stroke_width + 1.5, color));
            painter.line_segment([scale(16.0, 6.0), scale(16.0, 18.0)], Stroke::new(stroke_width + 1.5, color));
        }

        Icon::Stop => {
            // Square
            let r = size * 0.3;
            let stop_rect = Rect::from_center_size(center, vec2(r * 2.0, r * 2.0));
            painter.rect_filled(stop_rect, size * 0.08, color);
        }

        Icon::SkipForward => {
            // Two triangles + line
            let points1 = [
                scale(5.0, 6.0),
                scale(12.0, 12.0),
                scale(5.0, 18.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                points1.to_vec(),
                color,
                Stroke::NONE,
            ));
            let points2 = [
                scale(12.0, 6.0),
                scale(19.0, 12.0),
                scale(12.0, 18.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                points2.to_vec(),
                color,
                Stroke::NONE,
            ));
        }

        Icon::SkipBack => {
            // Two triangles pointing left
            let points1 = [
                scale(19.0, 6.0),
                scale(12.0, 12.0),
                scale(19.0, 18.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                points1.to_vec(),
                color,
                Stroke::NONE,
            ));
            let points2 = [
                scale(12.0, 6.0),
                scale(5.0, 12.0),
                scale(12.0, 18.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                points2.to_vec(),
                color,
                Stroke::NONE,
            ));
        }

        Icon::RotateCcw => {
            // Circular arrow (reset)
            use std::f32::consts::PI;
            let r = size * 0.35;
            let segments = 20;
            let start_angle = -PI * 0.1;
            let end_angle = PI * 1.4;

            // Arc
            let mut points: Vec<Pos2> = Vec::new();
            for i in 0..=segments {
                let t = i as f32 / segments as f32;
                let angle = start_angle + t * (end_angle - start_angle);
                points.push(Pos2::new(
                    center.x + angle.cos() * r,
                    center.y + angle.sin() * r,
                ));
            }
            painter.add(egui::Shape::line(points, stroke));

            // Arrow head at start
            let arrow_angle = start_angle;
            let arrow_pos = Pos2::new(
                center.x + arrow_angle.cos() * r,
                center.y + arrow_angle.sin() * r,
            );
            let arrow_size = size * 0.15;
            painter.line_segment(
                [
                    arrow_pos,
                    Pos2::new(arrow_pos.x - arrow_size, arrow_pos.y - arrow_size * 0.3),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    arrow_pos,
                    Pos2::new(arrow_pos.x + arrow_size * 0.3, arrow_pos.y - arrow_size),
                ],
                stroke,
            );
        }

        Icon::ChevronLeft => {
            painter.line_segment([scale(15.0, 6.0), scale(9.0, 12.0)], stroke);
            painter.line_segment([scale(9.0, 12.0), scale(15.0, 18.0)], stroke);
        }

        Icon::ChevronRight => {
            painter.line_segment([scale(9.0, 6.0), scale(15.0, 12.0)], stroke);
            painter.line_segment([scale(15.0, 12.0), scale(9.0, 18.0)], stroke);
        }

        Icon::ChevronUp => {
            painter.line_segment([scale(6.0, 15.0), scale(12.0, 9.0)], stroke);
            painter.line_segment([scale(12.0, 9.0), scale(18.0, 15.0)], stroke);
        }

        Icon::ChevronDown => {
            painter.line_segment([scale(6.0, 9.0), scale(12.0, 15.0)], stroke);
            painter.line_segment([scale(12.0, 15.0), scale(18.0, 9.0)], stroke);
        }

        Icon::ArrowLeft => {
            painter.line_segment([scale(19.0, 12.0), scale(5.0, 12.0)], stroke);
            painter.line_segment([scale(5.0, 12.0), scale(11.0, 6.0)], stroke);
            painter.line_segment([scale(5.0, 12.0), scale(11.0, 18.0)], stroke);
        }

        Icon::X => {
            painter.line_segment([scale(6.0, 6.0), scale(18.0, 18.0)], stroke);
            painter.line_segment([scale(18.0, 6.0), scale(6.0, 18.0)], stroke);
        }

        Icon::Plus => {
            painter.line_segment([scale(12.0, 5.0), scale(12.0, 19.0)], stroke);
            painter.line_segment([scale(5.0, 12.0), scale(19.0, 12.0)], stroke);
        }

        Icon::Minus => {
            painter.line_segment([scale(5.0, 12.0), scale(19.0, 12.0)], stroke);
        }

        Icon::Check => {
            painter.line_segment([scale(5.0, 12.0), scale(10.0, 17.0)], stroke);
            painter.line_segment([scale(10.0, 17.0), scale(19.0, 7.0)], stroke);
        }

        Icon::Minimize => {
            painter.line_segment([scale(6.0, 12.0), scale(18.0, 12.0)], stroke);
        }

        Icon::Maximize => {
            let r = size * 0.3;
            let max_rect = Rect::from_center_size(center, vec2(r * 2.0, r * 2.0));
            painter.rect_stroke(max_rect, 0.0, stroke);
        }

        Icon::Restore => {
            // Two overlapping squares
            let r = size * 0.22;
            let offset = size * 0.1;

            // Back square
            let back_rect = Rect::from_center_size(
                center + vec2(offset, -offset),
                vec2(r * 2.0, r * 2.0),
            );
            painter.rect_stroke(back_rect, 0.0, stroke);

            // Front square (with filled background to cover back)
            let front_rect = Rect::from_center_size(
                center + vec2(-offset * 0.5, offset * 0.5),
                vec2(r * 2.0, r * 2.0),
            );
            painter.rect_stroke(front_rect, 0.0, stroke);
        }

        Icon::Settings => {
            // Gear/cog icon
            use std::f32::consts::PI;
            let outer_r = size * 0.4;
            let inner_r = size * 0.2;
            let teeth = 6;

            // Draw gear teeth
            for i in 0..teeth {
                let angle = (i as f32 / teeth as f32) * PI * 2.0 - PI / 2.0;
                let tooth_width = PI / teeth as f32 * 0.6;

                let p1 = Pos2::new(
                    center.x + (angle - tooth_width).cos() * inner_r * 1.3,
                    center.y + (angle - tooth_width).sin() * inner_r * 1.3,
                );
                let p2 = Pos2::new(
                    center.x + (angle - tooth_width * 0.5).cos() * outer_r,
                    center.y + (angle - tooth_width * 0.5).sin() * outer_r,
                );
                let p3 = Pos2::new(
                    center.x + (angle + tooth_width * 0.5).cos() * outer_r,
                    center.y + (angle + tooth_width * 0.5).sin() * outer_r,
                );
                let p4 = Pos2::new(
                    center.x + (angle + tooth_width).cos() * inner_r * 1.3,
                    center.y + (angle + tooth_width).sin() * inner_r * 1.3,
                );

                painter.line_segment([p1, p2], stroke);
                painter.line_segment([p2, p3], stroke);
                painter.line_segment([p3, p4], stroke);
            }

            // Inner circle
            painter.circle_stroke(center, inner_r, stroke);

            // Outer connection circle
            painter.circle_stroke(center, inner_r * 1.3, stroke);
        }

        Icon::LayoutDashboard => {
            // Dashboard grid layout
            let padding = size * 0.15;
            let gap = size * 0.08;

            let rect = Rect::from_center_size(center, vec2(size - padding * 2.0, size - padding * 2.0));

            // Top-left (larger)
            let tl = Rect::from_min_max(
                rect.left_top(),
                Pos2::new(rect.center().x - gap / 2.0, rect.center().y - gap / 2.0),
            );
            painter.rect_stroke(tl, size * 0.04, stroke);

            // Top-right
            let tr = Rect::from_min_max(
                Pos2::new(rect.center().x + gap / 2.0, rect.top()),
                Pos2::new(rect.right(), rect.center().y - gap / 2.0),
            );
            painter.rect_stroke(tr, size * 0.04, stroke);

            // Bottom-left
            let bl = Rect::from_min_max(
                Pos2::new(rect.left(), rect.center().y + gap / 2.0),
                Pos2::new(rect.center().x - gap / 2.0, rect.bottom()),
            );
            painter.rect_stroke(bl, size * 0.04, stroke);

            // Bottom-right
            let br = Rect::from_min_max(
                Pos2::new(rect.center().x + gap / 2.0, rect.center().y + gap / 2.0),
                rect.right_bottom(),
            );
            painter.rect_stroke(br, size * 0.04, stroke);
        }

        Icon::Timer => {
            // Timer/stopwatch
            let r = size * 0.38;
            painter.circle_stroke(center, r, stroke);

            // Top button
            painter.line_segment([scale(12.0, 3.0), scale(12.0, 6.0)], stroke);

            // Hands
            painter.line_segment([center, scale(12.0, 8.0)], stroke);
            painter.line_segment([center, scale(16.0, 12.0)], stroke);
        }

        Icon::Clock => {
            let r = size * 0.4;
            painter.circle_stroke(center, r, stroke);

            // Hour hand
            painter.line_segment([center, scale(12.0, 8.0)], stroke);
            // Minute hand
            painter.line_segment([center, scale(16.0, 12.0)], stroke);
        }

        Icon::Bell => {
            // Bell shape
            use std::f32::consts::PI;

            // Bell body (arc)
            let bell_r = size * 0.3;
            let segments = 12;
            let mut points: Vec<Pos2> = Vec::new();

            // Left side curve down
            points.push(scale(8.0, 10.0));

            // Bottom curve
            for i in 0..=segments {
                let t = i as f32 / segments as f32;
                let angle = PI + t * PI;
                points.push(Pos2::new(
                    center.x + angle.cos() * bell_r,
                    center.y + 2.0 * (size / 24.0) + angle.sin().abs() * bell_r * 0.5,
                ));
            }

            points.push(scale(16.0, 10.0));

            painter.add(egui::Shape::line(points, stroke));

            // Top connection
            painter.line_segment([scale(8.0, 10.0), scale(10.0, 6.0)], stroke);
            painter.line_segment([scale(16.0, 10.0), scale(14.0, 6.0)], stroke);
            painter.line_segment([scale(10.0, 6.0), scale(14.0, 6.0)], stroke);

            // Clapper
            painter.line_segment([scale(12.0, 17.0), scale(12.0, 19.0)], stroke);

            // Top knob
            painter.circle_stroke(scale(12.0, 5.0), size * 0.05, stroke);
        }

        Icon::BellOff => {
            // Bell shape (duplicated to avoid borrow issues)
            painter.line_segment([scale(6.0, 10.0), scale(6.0, 14.0)], stroke);
            painter.line_segment([scale(6.0, 14.0), scale(8.0, 18.0)], stroke);
            painter.line_segment([scale(8.0, 18.0), scale(16.0, 18.0)], stroke);
            painter.line_segment([scale(16.0, 18.0), scale(18.0, 14.0)], stroke);
            painter.line_segment([scale(18.0, 14.0), scale(18.0, 10.0)], stroke);
            painter.line_segment([scale(18.0, 10.0), scale(6.0, 10.0)], stroke);
            painter.line_segment([scale(8.0, 10.0), scale(10.0, 6.0)], stroke);
            painter.line_segment([scale(16.0, 10.0), scale(14.0, 6.0)], stroke);
            painter.line_segment([scale(10.0, 6.0), scale(14.0, 6.0)], stroke);
            painter.line_segment([scale(12.0, 17.0), scale(12.0, 19.0)], stroke);
            painter.circle_stroke(scale(12.0, 5.0), size * 0.05, stroke);

            // Diagonal slash
            let slash_stroke = Stroke::new(stroke_width * 1.5, color);
            painter.line_segment([scale(4.0, 4.0), scale(20.0, 20.0)], slash_stroke);
        }

        Icon::Volume2 => {
            // Speaker with waves
            // Speaker body
            painter.line_segment([scale(6.0, 9.0), scale(9.0, 9.0)], stroke);
            painter.line_segment([scale(9.0, 9.0), scale(13.0, 5.0)], stroke);
            painter.line_segment([scale(13.0, 5.0), scale(13.0, 19.0)], stroke);
            painter.line_segment([scale(13.0, 19.0), scale(9.0, 15.0)], stroke);
            painter.line_segment([scale(9.0, 15.0), scale(6.0, 15.0)], stroke);
            painter.line_segment([scale(6.0, 15.0), scale(6.0, 9.0)], stroke);

            // Sound waves (arcs)
            use std::f32::consts::PI;
            for (_i, r) in [(0, size * 0.15), (1, size * 0.25)].iter() {
                let wave_center = scale(13.0, 12.0);
                let segments = 8;
                let mut points: Vec<Pos2> = Vec::new();

                for j in 0..=segments {
                    let t = j as f32 / segments as f32;
                    let angle = -PI / 3.0 + t * (PI * 2.0 / 3.0);
                    points.push(Pos2::new(
                        wave_center.x + angle.cos() * r,
                        wave_center.y + angle.sin() * r,
                    ));
                }

                painter.add(egui::Shape::line(points, stroke));
            }
        }

        Icon::VolumeX => {
            // Speaker with X
            // Speaker body
            painter.line_segment([scale(3.0, 9.0), scale(6.0, 9.0)], stroke);
            painter.line_segment([scale(6.0, 9.0), scale(10.0, 5.0)], stroke);
            painter.line_segment([scale(10.0, 5.0), scale(10.0, 19.0)], stroke);
            painter.line_segment([scale(10.0, 19.0), scale(6.0, 15.0)], stroke);
            painter.line_segment([scale(6.0, 15.0), scale(3.0, 15.0)], stroke);
            painter.line_segment([scale(3.0, 15.0), scale(3.0, 9.0)], stroke);

            // X
            painter.line_segment([scale(14.0, 9.0), scale(20.0, 15.0)], stroke);
            painter.line_segment([scale(20.0, 9.0), scale(14.0, 15.0)], stroke);
        }

        Icon::Coffee => {
            // Coffee cup
            // Cup body
            painter.line_segment([scale(6.0, 7.0), scale(6.0, 17.0)], stroke);
            painter.line_segment([scale(6.0, 17.0), scale(14.0, 17.0)], stroke);
            painter.line_segment([scale(14.0, 17.0), scale(14.0, 7.0)], stroke);

            // Cup top
            painter.line_segment([scale(5.0, 7.0), scale(15.0, 7.0)], stroke);

            // Handle
            use std::f32::consts::PI;
            let handle_center = scale(14.0, 11.0);
            let handle_r = size * 0.12;
            let segments = 8;
            let mut points: Vec<Pos2> = Vec::new();

            for i in 0..=segments {
                let t = i as f32 / segments as f32;
                let angle = -PI / 2.0 + t * PI;
                points.push(Pos2::new(
                    handle_center.x + angle.cos() * handle_r,
                    handle_center.y + angle.sin() * handle_r,
                ));
            }
            painter.add(egui::Shape::line(points, stroke));

            // Steam
            painter.line_segment([scale(8.0, 4.0), scale(8.5, 2.0)], stroke);
            painter.line_segment([scale(10.0, 4.0), scale(10.5, 2.0)], stroke);
            painter.line_segment([scale(12.0, 4.0), scale(12.5, 2.0)], stroke);
        }

        Icon::Target => {
            // Target/focus
            let r1 = size * 0.4;
            let r2 = size * 0.25;
            let r3 = size * 0.1;

            painter.circle_stroke(center, r1, stroke);
            painter.circle_stroke(center, r2, stroke);
            painter.circle_filled(center, r3, color);
        }

        Icon::Flame => {
            // Fire/flame
            // Outer flame shape
            let points = [
                scale(12.0, 3.0),
                scale(15.0, 8.0),
                scale(17.0, 12.0),
                scale(16.0, 16.0),
                scale(14.0, 19.0),
                scale(12.0, 20.0),
                scale(10.0, 19.0),
                scale(8.0, 16.0),
                scale(7.0, 12.0),
                scale(9.0, 8.0),
            ];

            painter.add(egui::Shape::line(points.to_vec(), stroke));
            painter.line_segment([points[9], points[0]], stroke);

            // Inner flame
            painter.line_segment([scale(12.0, 12.0), scale(13.0, 15.0)], stroke);
            painter.line_segment([scale(13.0, 15.0), scale(12.0, 18.0)], stroke);
            painter.line_segment([scale(12.0, 18.0), scale(11.0, 15.0)], stroke);
            painter.line_segment([scale(11.0, 15.0), scale(12.0, 12.0)], stroke);
        }

        Icon::Calendar => {
            // Calendar
            let r = size * 0.35;
            let cal_rect = Rect::from_center_size(center + vec2(0.0, size * 0.03), vec2(r * 2.0, r * 2.0));

            painter.rect_stroke(cal_rect, size * 0.06, stroke);

            // Top line (header)
            painter.line_segment(
                [
                    Pos2::new(cal_rect.left(), cal_rect.top() + r * 0.5),
                    Pos2::new(cal_rect.right(), cal_rect.top() + r * 0.5),
                ],
                stroke,
            );

            // Hooks
            painter.line_segment([scale(9.0, 4.0), scale(9.0, 8.0)], stroke);
            painter.line_segment([scale(15.0, 4.0), scale(15.0, 8.0)], stroke);
        }

        Icon::BarChart3 => {
            // Bar chart
            painter.line_segment([scale(6.0, 19.0), scale(6.0, 13.0)], Stroke::new(stroke_width + 1.0, color));
            painter.line_segment([scale(10.0, 19.0), scale(10.0, 9.0)], Stroke::new(stroke_width + 1.0, color));
            painter.line_segment([scale(14.0, 19.0), scale(14.0, 5.0)], Stroke::new(stroke_width + 1.0, color));
            painter.line_segment([scale(18.0, 19.0), scale(18.0, 11.0)], Stroke::new(stroke_width + 1.0, color));
        }

        Icon::TrendingUp => {
            // Trending up arrow
            painter.line_segment([scale(4.0, 17.0), scale(10.0, 11.0)], stroke);
            painter.line_segment([scale(10.0, 11.0), scale(14.0, 15.0)], stroke);
            painter.line_segment([scale(14.0, 15.0), scale(20.0, 7.0)], stroke);

            // Arrow head
            painter.line_segment([scale(20.0, 7.0), scale(15.0, 7.0)], stroke);
            painter.line_segment([scale(20.0, 7.0), scale(20.0, 12.0)], stroke);
        }

        Icon::Sun => {
            // Sun
            let r = size * 0.18;
            painter.circle_stroke(center, r, stroke);

            // Rays
            let ray_inner = size * 0.25;
            let ray_outer = size * 0.38;

            for i in 0..8 {
                let angle = (i as f32 / 8.0) * std::f32::consts::PI * 2.0;
                let inner = Pos2::new(
                    center.x + angle.cos() * ray_inner,
                    center.y + angle.sin() * ray_inner,
                );
                let outer = Pos2::new(
                    center.x + angle.cos() * ray_outer,
                    center.y + angle.sin() * ray_outer,
                );
                painter.line_segment([inner, outer], stroke);
            }
        }

        Icon::Moon => {
            // Crescent moon
            use std::f32::consts::PI;

            let r = size * 0.35;
            let inner_r = size * 0.25;
            let offset = size * 0.15;

            // Outer arc
            let segments = 20;
            let mut points: Vec<Pos2> = Vec::new();

            for i in 0..=segments {
                let t = i as f32 / segments as f32;
                let angle = -PI * 0.3 + t * PI * 1.6;
                points.push(Pos2::new(
                    center.x + angle.cos() * r,
                    center.y + angle.sin() * r,
                ));
            }

            // Inner arc (reverse direction to create crescent)
            for i in (0..=segments).rev() {
                let t = i as f32 / segments as f32;
                let angle = -PI * 0.1 + t * PI * 1.2;
                points.push(Pos2::new(
                    center.x + offset + angle.cos() * inner_r,
                    center.y + angle.sin() * inner_r,
                ));
            }

            painter.add(egui::Shape::line(points, stroke));
        }

        Icon::Palette => {
            // Color palette
            let r = size * 0.38;
            painter.circle_stroke(center, r, stroke);

            // Color dots
            let dot_r = size * 0.06;
            let dot_positions = [
                (-0.5, -0.5),
                (0.3, -0.5),
                (-0.6, 0.1),
                (0.0, 0.3),
            ];

            for (dx, dy) in dot_positions.iter() {
                let pos = Pos2::new(
                    center.x + dx * r,
                    center.y + dy * r,
                );
                painter.circle_filled(pos, dot_r, color);
            }

            // Thumb hole
            let hole_center = Pos2::new(center.x + r * 0.4, center.y + r * 0.3);
            painter.circle_stroke(hole_center, dot_r * 1.5, stroke);
        }

        Icon::Zap => {
            // Lightning bolt
            let points = [
                scale(13.0, 2.0),
                scale(7.0, 12.0),
                scale(11.0, 12.0),
                scale(9.0, 22.0),
                scale(17.0, 10.0),
                scale(13.0, 10.0),
            ];

            painter.add(egui::Shape::line(points.to_vec(), stroke));
            painter.line_segment([points[5], points[0]], stroke);
        }

        Icon::Download => {
            // Download arrow with line
            // Vertical line
            painter.line_segment([scale(12.0, 4.0), scale(12.0, 14.0)], stroke);
            // Arrow head
            painter.line_segment([scale(12.0, 14.0), scale(7.0, 9.0)], stroke);
            painter.line_segment([scale(12.0, 14.0), scale(17.0, 9.0)], stroke);
            // Bottom line (tray)
            painter.line_segment([scale(5.0, 18.0), scale(19.0, 18.0)], stroke);
            // Side lines
            painter.line_segment([scale(5.0, 18.0), scale(5.0, 14.0)], stroke);
            painter.line_segment([scale(19.0, 18.0), scale(19.0, 14.0)], stroke);
        }

        Icon::Pin => {
            // Pin/thumbtack icon (always on top - active)
            // Pin body (diagonal)
            painter.line_segment([scale(15.0, 4.0), scale(9.0, 10.0)], stroke);
            // Pin head circle
            painter.circle_stroke(scale(16.5, 5.5), size * 0.12, stroke);
            // Pin body rectangle
            painter.line_segment([scale(9.0, 10.0), scale(7.0, 12.0)], stroke);
            painter.line_segment([scale(7.0, 12.0), scale(11.0, 16.0)], stroke);
            painter.line_segment([scale(11.0, 16.0), scale(13.0, 14.0)], stroke);
            painter.line_segment([scale(13.0, 14.0), scale(9.0, 10.0)], stroke);
            // Pin needle
            painter.line_segment([scale(9.0, 14.0), scale(5.0, 20.0)], stroke);
        }

        Icon::PinOff => {
            // Pin icon with slash (not pinned)
            // Pin body (diagonal)
            painter.line_segment([scale(15.0, 4.0), scale(9.0, 10.0)], stroke);
            // Pin head circle
            painter.circle_stroke(scale(16.5, 5.5), size * 0.12, stroke);
            // Pin body rectangle
            painter.line_segment([scale(9.0, 10.0), scale(7.0, 12.0)], stroke);
            painter.line_segment([scale(7.0, 12.0), scale(11.0, 16.0)], stroke);
            painter.line_segment([scale(11.0, 16.0), scale(13.0, 14.0)], stroke);
            painter.line_segment([scale(13.0, 14.0), scale(9.0, 10.0)], stroke);
            // Pin needle
            painter.line_segment([scale(9.0, 14.0), scale(5.0, 20.0)], stroke);
            // Diagonal slash
            let slash_stroke = Stroke::new(stroke_width * 1.5, color);
            painter.line_segment([scale(4.0, 4.0), scale(20.0, 20.0)], slash_stroke);
        }
    }
}

/// Draw an icon centered at a position with given size
pub fn draw_icon_at(ui: &mut Ui, icon: Icon, center: Pos2, size: f32, color: Color32) {
    let rect = Rect::from_center_size(center, vec2(size, size));
    draw_icon(ui, icon, rect, color);
}

/// Helper struct for drawing icons with builder pattern
pub struct IconPainter {
    icon: Icon,
    size: f32,
    color: Color32,
}

impl IconPainter {
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            size: 24.0,
            color: Color32::WHITE,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    pub fn paint(self, ui: &mut Ui, center: Pos2) {
        draw_icon_at(ui, self.icon, center, self.size, self.color);
    }

    pub fn paint_rect(self, ui: &mut Ui, rect: Rect) {
        draw_icon(ui, self.icon, rect, self.color);
    }
}
