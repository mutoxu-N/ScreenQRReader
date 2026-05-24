use dioxus::{
    desktop::{
        trayicon::{
            menu::{Menu, MenuItem},
            TrayIconBuilder,
        },
        use_muda_event_handler,
    },
    prelude::provide_context,
};

pub fn create_tray_menu() {
    // タスクトレイアイコン
    let icon = {
        let icon_bytes = include_bytes!("../../assets/favicon.ico");
        let img =
            image::load_from_memory(icon_bytes).expect("アイコン画像の読み込みに失敗しました");
        let rgba = img.into_rgba8();
        let (w, h) = rgba.dimensions();
        let i = dioxus::desktop::trayicon::Icon::from_rgba(rgba.into_raw(), w, h)
            .expect("アイコンの作成に失敗しました");
        i
    };

    // トレイメニュー
    let menu = Menu::new();

    // メニューアイテム
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);
    let _ = menu.append_items(&[&quit_item]).unwrap(); // アイテム登録

    // トレイアイコンの作成
    let builder = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_icon(icon);

    // Dioxusにトレイを登録
    provide_context(builder.build().expect("TrayIconBuilderの失敗"));

    // トレイメニューの右クリック処理
    {
        use_muda_event_handler(move |event| {
            println!("{:?}", event.id);
            match event.id.0.as_str() {
                // 終了
                "quit" => {
                    println!("quit>> {:?}", event.id);
                    std::process::exit(0);
                }
                _ => {
                    println!("quit>> {:?}", event.id);
                }
            }
        });
    }
}
