fn main() {
    #[cfg(target_os = "windows")]
    {
        use std::path::Path;
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let ico_path = "assets/icon.ico";
        let png_path = "assets/icon.png";

        // Convert PNG to ICO if needed
        if Path::new(png_path).exists() && !Path::new(ico_path).exists() {
            if let Ok(img) = image::open(png_path) {
                let img = img.into_rgba8();
                let (width, height) = (img.width(), img.height());

                // Create ICO file manually (simple single-image ICO)
                if let Ok(file) = File::create(ico_path) {
                    let mut writer = BufWriter::new(file);

                    // ICO header
                    let _ = writer.write_all(&[0, 0]); // Reserved
                    let _ = writer.write_all(&[1, 0]); // Type: 1 = ICO
                    let _ = writer.write_all(&[1, 0]); // Number of images

                    // Image entry
                    let _ = writer.write_all(&[(width & 0xFF) as u8]); // Width
                    let _ = writer.write_all(&[(height & 0xFF) as u8]); // Height
                    let _ = writer.write_all(&[0]); // Color palette
                    let _ = writer.write_all(&[0]); // Reserved
                    let _ = writer.write_all(&[1, 0]); // Color planes
                    let _ = writer.write_all(&[32, 0]); // Bits per pixel

                    // PNG data
                    let png_data = std::fs::read(png_path).unwrap_or_default();
                    let size = png_data.len() as u32;
                    let _ = writer.write_all(&size.to_le_bytes()); // Size of image data
                    let _ = writer.write_all(&22u32.to_le_bytes()); // Offset to image data

                    // PNG image data
                    let _ = writer.write_all(&png_data);
                    let _ = writer.flush();

                    println!("cargo:warning=Created icon.ico from icon.png");
                }
            }
        }

        let mut res = winresource::WindowsResource::new();

        // Application icon
        if Path::new(ico_path).exists() {
            res.set_icon(ico_path);
        }

        // Version info
        res.set("ProductName", "PomodoRust");
        res.set("FileDescription", "Modern Pomodoro Timer - Focus better with the Pomodoro Technique");
        res.set("LegalCopyright", "Copyright 2025 gerrux. MIT License.");
        res.set("CompanyName", "gerrux");
        res.set("OriginalFilename", "pomodorust.exe");
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("InternalName", "pomodorust");

        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}
