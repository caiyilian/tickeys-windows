fn main() {
    let dll_dir = std::path::Path::new("resource/dll");

    if dll_dir.exists() {
        println!("cargo:rustc-link-search={}", dll_dir.display());
        println!("cargo:rustc-link-lib=OpenAL32");

        let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
        let out = std::path::Path::new("target").join(&profile);
        let dll_src = dll_dir.join("OpenAL32.dll");
        let dst = out.join("OpenAL32.dll");
        if dll_src.exists() && !dst.exists() {
            let _ = std::fs::copy(&dll_src, &dst);
        }
    }

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
