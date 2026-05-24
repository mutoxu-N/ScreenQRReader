// トリミング用のUI

use crate::state::SelectionRect;
use dioxus::desktop::use_window;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct OverlayProps {
    pub image_data: String,
}

#[component]
pub fn Overlay(props: OverlayProps) -> Element {
    let mut rect = use_signal(|| SelectionRect {
        start_x: 0.0,
        start_y: 0.0,
        current_x: 0.0,
        current_y: 0.0,
        is_dragging: false,
    });
    let window = use_window();

    // マウント時に自身をグローバルなウィンドウリストに登録
    use_hook(|| {
        crate::core::capture::register_overlay_window(window.clone());
    });

    // モニターのスクリーンショットを保持
    let image_data = props.image_data.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: crate::MAIN_CSS }
        div {
            class: "fixed inset-0 z-[9999] cursor-crosshair select-none outline-none overflow-hidden",
            tabindex: 0,
            autofocus: true,

            // ESC ならキャンセル
            onkeydown: move |e| {
                if let dioxus::html::input_data::keyboard_types::Key::Escape = e.key() {
                    crate::core::capture::close_all_overlays();
                    return;
                }
                if let dioxus::html::input_data::keyboard_types::Key::Control = e.key() {
                    crate::core::capture::close_all_overlays();
                    return;
                }
            },

            // 右クリック時のコンテキストメニュー表示を防ぎつつキャンセル処理
            oncontextmenu: move |_| {
                crate::core::capture::close_all_overlays();
            },

            // ドラッグ開始
            onmousedown: move |e| {
                // 右クリックならキャンセル
                if e.trigger_button() == Some(dioxus::html::input_data::MouseButton::Secondary) {
                    crate::core::capture::close_all_overlays();
                    return;
                }

                let coords = e.client_coordinates();
                rect.set(SelectionRect {
                    start_x: coords.x,
                    start_y: coords.y,
                    current_x: coords.x,
                    current_y: coords.y,
                    is_dragging: true,
                });
            },

            // ドラッグ中
            onmousemove: move |e| {
                if rect.read().is_dragging {
                    let coords = e.client_coordinates();
                    rect.write().current_x = coords.x;
                    rect.write().current_y = coords.y;
                }
            },

            // ドラッグ終了
            onmouseup: move |_| {
                if rect.read().is_dragging {
                    let mut r = rect.write();
                    r.is_dragging = false;

                    println!("========== 領域が選択されました ==========");
                    println!("左上X: {}, 左上Y: {}", r.left(), r.top());
                    println!("幅: {}, 高さ: {}", r.width(), r.height());
                    println!("========================================");

                    // 全画面と切り抜き領域を渡す
                    crate::core::qr_action::process_qr_code(&image_data, &r);

                    crate::core::capture::close_all_overlays();
                }
            },

            img {
                src: "{props.image_data}",
                class: "absolute inset-0 w-full h-full object-cover",
                draggable: false,
            }


            div {
                class: "absolute pointer-events-none border border-white/50 z-10",
                style: "left: {rect.read().left()}px; top: {rect.read().top()}px; width: {rect.read().width()}px; height: {rect.read().height()}px; box-shadow: 0 0 0 9999px rgba(0,0,0,0.5);"
            }
        }
    }
}
