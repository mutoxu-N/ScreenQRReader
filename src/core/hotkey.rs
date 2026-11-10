use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};

pub fn setup_hotkey() -> (Option<GlobalHotKeyManager>, Option<HotKey>) {
    let manager = match GlobalHotKeyManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("GlobalHotKeyManagerの初期化に失敗しました: {}", e);
            return (None, None);
        }
    };

    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyQ);
    if let Err(e) = manager.register(hotkey) {
        eprintln!("ホットキー (Ctrl+Shift+Q) の登録に失敗しました: {}", e);
        return (Some(manager), None);
    }

    (Some(manager), Some(hotkey))
}
