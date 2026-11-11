use std::time::Duration;

mod components;
mod core;
mod state;

struct ScreenQrReaderApp {
    _tray_icon: Option<tray_icon::TrayIcon>,
    _hotkey_manager: Option<global_hotkey::GlobalHotKeyManager>,
    hotkey: Option<global_hotkey::hotkey::HotKey>,
    captures: Vec<components::overlay::OverlayState>,
    is_capturing: bool,
}

impl ScreenQrReaderApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        core::platform::init_window_animation_hook();
        let tray_icon = core::tray::create_tray_menu();
        let (hotkey_manager, hotkey) = core::hotkey::setup_hotkey();

        Self {
            _tray_icon: tray_icon,
            _hotkey_manager: hotkey_manager,
            hotkey,
            captures: Vec::new(),
            is_capturing: false,
        }
    }

    fn trigger_capture(&mut self) {
        if !core::capture::try_start_capture() {
            return;
        }

        let monitor_captures = core::capture::capture_all_monitors();
        if monitor_captures.is_empty() {
            core::capture::finish_capture();
            return;
        }

        self.captures = monitor_captures
            .into_iter()
            .map(components::overlay::OverlayState::new)
            .collect();
        self.is_capturing = true;
        core::platform::set_capturing_active(true);
    }

    fn cancel_capture(&mut self) {
        self.captures.clear();
        self.is_capturing = false;
        core::platform::set_capturing_active(false);
        core::capture::finish_capture();
    }
}

impl Drop for ScreenQrReaderApp {
    fn drop(&mut self) {
        core::platform::cleanup_window_animation_hook();
    }
}

impl eframe::App for ScreenQrReaderApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // トレイメニューイベントのポーリング
        if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            match event.id().as_ref() {
                core::tray::MENU_QUIT_ID => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }
                core::tray::MENU_CAPTURE_ID => {
                    self.trigger_capture();
                }
                _ => {}
            }
        }

        // ホットキーイベントのポーリング
        if let Ok(event) = global_hotkey::GlobalHotKeyEvent::receiver().try_recv() {
            if let Some(hotkey) = &self.hotkey {
                if event.id == hotkey.id() && event.state == global_hotkey::HotKeyState::Pressed {
                    self.trigger_capture();
                }
            }
        }

        // オーバーレイ描画処理
        if self.is_capturing {
            let mut completed_idx_and_rect = None;
            let mut should_cancel = false;

            for (i, overlay_state) in self.captures.iter_mut().enumerate() {
                let vp_id = egui::ViewportId::from_hash_of(&format!("overlay_{}", i));
                let vp_builder = egui::ViewportBuilder::default()
                    .with_title("Screen QR Reader Overlay")
                    .with_visible(true)
                    .with_decorations(false)
                    .with_resizable(false)
                    .with_has_shadow(false)
                    .with_taskbar(false)
                    .with_transparent(true)
                    .with_always_on_top()
                    .with_position([overlay_state.capture.x as f32, overlay_state.capture.y as f32])
                    .with_inner_size([overlay_state.capture.width as f32, overlay_state.capture.height as f32]);

                ctx.show_viewport_immediate(vp_id, vp_builder, |vp_ui, _class| {
                    let action = components::overlay::render_overlay(vp_ui, overlay_state);
                    match action {
                        components::overlay::OverlayAction::Complete(rect) => {
                            completed_idx_and_rect = Some((i, rect));
                        }
                        components::overlay::OverlayAction::Cancel => {
                            should_cancel = true;
                        }
                        components::overlay::OverlayAction::None => {}
                    }
                });
            }

            core::platform::disable_overlay_window_animations();

            if let Some((idx, rect)) = completed_idx_and_rect {
                if idx < self.captures.len() {
                    let capture = &self.captures[idx].capture;
                    core::qr_action::process_qr_code(&capture.image, &rect);
                }
                self.cancel_capture();
            } else if should_cancel {
                self.cancel_capture();
            }
        }

        // メインウィンドウUI
        ui.vertical_centered(|ui| {
            ui.heading("Screen QR Reader");
            ui.add_space(4.0);
            ui.label("ホットキー: Ctrl + Shift + Q");
            ui.label("タスクトレイにも常駐しています");
            ui.add_space(10.0);

            if ui.button("📷 画面をキャプチャ").clicked() {
                self.trigger_capture();
            }

            ui.add_space(6.0);
            if ui.button("終了").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        // イベント監視のため低頻度で再描画リクエスト
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Screen QR Reader")
            .with_inner_size([280.0, 160.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Screen QR Reader",
        options,
        Box::new(|cc| Ok(Box::new(ScreenQrReaderApp::new(cc)))),
    )
}
