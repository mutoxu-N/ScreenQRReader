// トリミング用のUI (egui)

use crate::core::capture::MonitorCapture;
use crate::state::SelectionRect;
use egui::{Color32, Pos2, Rect, Stroke, StrokeKind};

pub struct OverlayState {
    pub capture: MonitorCapture,
    pub texture: Option<egui::TextureHandle>,
    pub rect: SelectionRect,
}

impl OverlayState {
    pub fn new(capture: MonitorCapture) -> Self {
        Self {
            capture,
            texture: None,
            rect: SelectionRect::default(),
        }
    }
}

pub enum OverlayAction {
    None,
    Complete(SelectionRect),
    Cancel,
}

pub fn render_overlay(
    ui: &mut egui::Ui,
    state: &mut OverlayState,
) -> OverlayAction {
    let ctx = ui.ctx().clone();

    // キャプチャ画像からテクスチャをロード
    if state.texture.is_none() {
        let size = [state.capture.width as usize, state.capture.height as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            size,
            state.capture.image.as_raw(),
        );
        state.texture = Some(ctx.load_texture(
            "screen_capture",
            color_image,
            egui::TextureOptions::LINEAR,
        ));
    }

    let mut action = OverlayAction::None;

    // カーソルを十字にする
    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);

    // ESCキーまたは右クリックでキャンセル
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return OverlayAction::Cancel;
    }
    if ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Secondary)) {
        return OverlayAction::Cancel;
    }

    let full_rect = ui.max_rect();

    // 背景画像を描画
    if let Some(texture) = &state.texture {
        let painter = ui.painter();
        painter.image(
            texture.id(),
            full_rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    // マウス入力処理
    let response = ui.allocate_rect(full_rect, egui::Sense::click_and_drag());

    if response.drag_started_by(egui::PointerButton::Primary) {
        if let Some(pos) = response.interact_pointer_pos() {
            state.rect.start_x = pos.x;
            state.rect.start_y = pos.y;
            state.rect.current_x = pos.x;
            state.rect.current_y = pos.y;
            state.rect.is_dragging = true;
        }
    } else if response.dragged_by(egui::PointerButton::Primary) {
        if let Some(pos) = response.interact_pointer_pos() {
            state.rect.current_x = pos.x;
            state.rect.current_y = pos.y;
        }
    } else if response.drag_stopped_by(egui::PointerButton::Primary) {
        if state.rect.is_dragging {
            state.rect.is_dragging = false;
            action = OverlayAction::Complete(state.rect);
        }
    }

    let painter = ui.painter();
    let overlay_color = Color32::from_black_alpha(140);

    if state.rect.is_dragging || state.rect.width() > 1.0 || state.rect.height() > 1.0 {
        let sel = state.rect.to_egui_rect();

        // 選択範囲の外側4つの領域を暗転
        let top_rect = Rect::from_min_max(
            full_rect.min,
            Pos2::new(full_rect.max.x, sel.min.y.clamp(full_rect.min.y, full_rect.max.y)),
        );
        let bottom_rect = Rect::from_min_max(
            Pos2::new(full_rect.min.x, sel.max.y.clamp(full_rect.min.y, full_rect.max.y)),
            full_rect.max,
        );
        let left_rect = Rect::from_min_max(
            Pos2::new(full_rect.min.x, sel.min.y.clamp(full_rect.min.y, full_rect.max.y)),
            Pos2::new(sel.min.x.clamp(full_rect.min.x, full_rect.max.x), sel.max.y.clamp(full_rect.min.y, full_rect.max.y)),
        );
        let right_rect = Rect::from_min_max(
            Pos2::new(sel.max.x.clamp(full_rect.min.x, full_rect.max.x), sel.min.y.clamp(full_rect.min.y, full_rect.max.y)),
            Pos2::new(full_rect.max.x, sel.max.y.clamp(full_rect.min.y, full_rect.max.y)),
        );

        painter.rect_filled(top_rect, 0.0, overlay_color);
        painter.rect_filled(bottom_rect, 0.0, overlay_color);
        painter.rect_filled(left_rect, 0.0, overlay_color);
        painter.rect_filled(right_rect, 0.0, overlay_color);

        // 選択矩形の枠線を描画
        painter.rect_stroke(sel, 0.0, Stroke::new(1.5, Color32::WHITE), StrokeKind::Outside);
    } else {
        // 選択前は薄く暗転
        painter.rect_filled(full_rect, 0.0, Color32::from_black_alpha(60));
    }

    action
}
