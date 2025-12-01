use screenshots::Screen;
use image::{DynamicImage};
use tesseract::Tesseract;
use tauri::command;

#[command]
pub fn screen_read_text() -> Result<String, String> {
    // 1. Grab the first available screen
    let screen = Screen::all()
        .map_err(|e| format!("Failed to list screens: {:?}", e))?
        .into_iter()
        .next()
        .ok_or("No screen detected".to_string())?;

    // 2. Capture the screen
    let shot = screen.capture()
        .map_err(|e| format!("Screenshot failed: {:?}", e))?;

    // 3. Save screenshot temporarily (workaround)
    let temp_path = std::env::temp_dir().join("screenshot.png");
    shot.save(&temp_path)
        .map_err(|e| format!("Failed to save screenshot: {:?}", e))?;

    // 4. Load the PNG into memory using image crate
    let dyn_img = image::open(&temp_path)
        .map_err(|e| format!("Failed to open saved screenshot: {:?}", e))?;

    // Convert to RGBA8 for Tesseract
    let rgba_img = dyn_img.to_rgba8();

    // Encode to PNG in-memory using write_to
    let mut png_bytes: Vec<u8> = Vec::new();
    let cursor = std::io::Cursor::new(&mut png_bytes);
    DynamicImage::ImageRgba8(rgba_img)
        .write_to(cursor, image::ImageFormat::Png)
        .map_err(|e| format!("PNG encode failed: {:?}", e))?;

    // 5. Feed PNG bytes to Tesseract OCR
    let mut tess = Tesseract::new(None, Some("eng"))
        .map_err(|e| format!("Tesseract init failed: {:?}", e))?
        .set_image_from_mem(&png_bytes)
        .map_err(|e| format!("Tesseract set image failed: {:?}", e))?;

    let text = tess.get_text()
        .map_err(|e| format!("OCR failed: {:?}", e))?;

    Ok(text.trim().to_string())
}


#[command]
pub fn get_active_window_title() -> String {
    // later: use windows APIs, x11, or macOS APIs
    "Active window title (placeholder)".into()
}
