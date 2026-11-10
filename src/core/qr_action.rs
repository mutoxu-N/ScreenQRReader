// QR解析とブラウザ起動/クリップボードコピー

use crate::state::SelectionRect;
use rqrr::PreparedImage;

pub fn process_qr_code(img: &image::RgbaImage, rect: &SelectionRect) {
    let x = rect.left().max(0.0) as u32;
    let y = rect.top().max(0.0) as u32;
    let width = rect.width().max(0.0) as u32;
    let height = rect.height().max(0.0) as u32;

    // 選択範囲が極端に小さい（単なるクリックなど）場合は処理しない
    if width < 5 || height < 5 {
        return;
    }

    // 画像の範囲を超えないように安全にクリッピング
    let crop_x = x.min(img.width());
    let crop_y = y.min(img.height());
    let crop_w = width.min(img.width().saturating_sub(crop_x));
    let crop_h = height.min(img.height().saturating_sub(crop_y));

    if crop_w == 0 || crop_h == 0 {
        return;
    }

    // 画像を指定領域で切り抜き、グレースケールに変換
    let cropped = image::imageops::crop_imm(img, crop_x, crop_y, crop_w, crop_h).to_image();
    let luma = image::DynamicImage::ImageRgba8(cropped).into_luma8();

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
