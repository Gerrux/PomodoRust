fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();

        // Application icon (optional)
        if std::path::Path::new("assets/icon.ico").exists() {
            res.set_icon("assets/icon.ico");
        }

        // Version info
        res.set("ProductName", "PomodoRust");
        res.set("FileDescription", "Modern Pomodoro Timer with Vercel-style dark UI");
        res.set("LegalCopyright", "Copyright 2025 gerrux. MIT License.");
        res.set("CompanyName", "gerrux");
        res.set("OriginalFilename", "pomodorust.exe");
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));

        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}
