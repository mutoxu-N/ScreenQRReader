// QR解析とブラウザ起動/クリップボードコピー

use crate::state::SelectionRect;
use rqrr::PreparedImage;
use std::collections::HashSet;

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

    let mut decoded_contents = Vec::new();

    // 1. まず高速な rqrr で試行
    let mut prepared = PreparedImage::prepare(luma.clone());
    let grids = prepared.detect_grids();

    for grid in grids {
        if let Ok((_, content)) = grid.decode() {
            decoded_contents.push(content);
        }
    }

    // 2. rqrr でデコードできなかった場合（角丸QRやデザインQR等）、rxing (ZXing) でフォールバック
    if decoded_contents.is_empty() {
        let (w, h) = luma.dimensions();
        let mut hints = rxing::DecodeHints::default();
        hints.PossibleFormats = Some(HashSet::from([rxing::BarcodeFormat::QR_CODE]));
        hints.TryHarder = Some(true);

        if let Ok(results) = rxing::helpers::detect_multiple_in_luma_with_hints(
            luma.as_raw().to_vec(),
            w,
            h,
            &mut hints,
        ) {
            for res in results {
                decoded_contents.push(res.getText().to_string());
            }
        } else if let Ok(result) = rxing::helpers::detect_in_luma_slice_with_hints(
            luma.as_raw(),
            w,
            h,
            Some(rxing::BarcodeFormat::QR_CODE),
            &mut hints,
        ) {
            decoded_contents.push(result.getText().to_string());
        }
    }

    if decoded_contents.is_empty() {
        println!("ℹ️ 指定された領域からQRコードが検知されませんでした");
        return;
    }

    let mut texts_to_copy = Vec::new();
    for content in decoded_contents {
        println!("✅ QRコードを検知しました: {}", content);
        if content.starts_with("http://") || content.starts_with("https://") {
            if let Err(e) = open::that(&content) {
                eprintln!("ブラウザの起動に失敗しました: {}", e);
            }
        } else {
            texts_to_copy.push(content);
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
