use tray_icon::menu::{Menu, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub const MENU_CAPTURE_ID: &str = "capture";
pub const MENU_QUIT_ID: &str = "quit";

pub fn create_tray_menu() -> Option<TrayIcon> {
    let icon_bytes = include_bytes!("../../assets/trayicon.png");
    let img = match image::load_from_memory(icon_bytes) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("アイコン画像の読み込みに失敗しました: {}", e);
            return None;
        }
    };
    let rgba = img.into_rgba8();
    let (w, h) = rgba.dimensions();
    let icon = match Icon::from_rgba(rgba.into_raw(), w, h) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("トレイアイコンの作成に失敗しました: {}", e);
            return None;
        }
    };

    let menu = Menu::new();
    let capture_item = MenuItem::with_id(MENU_CAPTURE_ID, "QRキャプチャ (Ctrl+Shift+Q)", true, None);
    let quit_item = MenuItem::with_id(MENU_QUIT_ID, "終了", true, None);

    if let Err(e) = menu.append_items(&[&capture_item, &quit_item]) {
        eprintln!("トレイメニューの構築に失敗しました: {}", e);
    }

    match TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Screen QR Reader")
        .with_icon(icon)
        .build()
    {
        Ok(tray) => Some(tray),
        Err(e) => {
            eprintln!("TrayIconBuilderの作成に失敗しました: {}", e);
            None
        }
    }
}
