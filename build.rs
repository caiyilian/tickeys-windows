fn main() {
    // Embed application icon if available
    let icon_path = std::path::Path::new("resource/icon.ico");
    if icon_path.exists() {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resource/icon.ico");
        res.set("FileDescription", "Tickeys for Windows");
        res.set("ProductName", "Tickeys");
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.compile().unwrap();
    }
}
