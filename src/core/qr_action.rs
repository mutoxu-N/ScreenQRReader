// QR解析とブラウザ起動/クリップボードコピー

use crate::state::SelectionRect;
use base64::{engine::general_purpose, Engine as _};
use rqrr::PreparedImage;

pub fn process_qr_code(base64_img: &str, rect: &SelectionRect) {
    // Base64 のヘッダーを取り除いてデコード
    let b64_data = base64_img.trim_start_matches("data:image/jpeg;base64,");

    let decoded = match general_purpose::STANDARD.decode(b64_data) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Base64デコードエラー: {}", e);
            return;
        }
    };

    // バイトデータから画像として読み込み
    let mut img = match image::load_from_memory(&decoded) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("画像読み込みエラー: {}", e);
            return;
        }
    };

    // クロップ座標とサイズを計算
    let x = rect.left() as u32;
    let y = rect.top() as u32;
    let width = rect.width() as u32;
    let height = rect.height() as u32;

    // 選択範囲がない（単なるクリックなど）場合は処理しない
    if width == 0 || height == 0 {
        return;
    }

    // 画像の範囲を超えないようにクリッピングを安全に行う
    let crop_x = x.min(img.width());
    let crop_y = y.min(img.height());
    let crop_w = width.min(img.width() - crop_x);
    let crop_h = height.min(img.height() - crop_y);

    // 画像を指定領域で切り抜き、グレースケールに変換
    let cropped = img.crop(crop_x, crop_y, crop_w, crop_h);
    let luma = cropped.into_luma8();

    // QRコードを検知
    let mut prepared = PreparedImage::prepare(luma);
    let grids = prepared.detect_grids();

    if grids.is_empty() {
        println!("ℹ️ 指定された領域からQRコードが検知されませんでした");
    } else {
        let mut texts_to_copy = Vec::new();

        for grid in grids {
            if let Ok((_, content)) = grid.decode() {
                println!("✅ QRコードを検知しました: {}", content);
                if content.starts_with("http://") || content.starts_with("https://") {
                    if let Err(e) = open::that(&content) {
                        eprintln!("ブラウザの起動に失敗しました: {}", e);
                    }
                } else {
                    texts_to_copy.push(content);
                }
            } else {
                eprintln!("⚠️ QRコードは検知できましたが、デコードに失敗しました");
            }
        }

        if !texts_to_copy.is_empty() {
            let combined_text = texts_to_copy.join("\n");
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Err(e) = clipboard.set_text(combined_text) {
                    eprintln!("クリップボードへのコピーに失敗しました: {}", e);
                }
            } else {
                eprintln!("クリップボードの初期化に失敗しました");
            }
        }
    }
}
