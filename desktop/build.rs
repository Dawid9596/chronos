fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../icons/icon.ico");
    #[cfg(windows)]
    {
        if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
            let mut res = winresource::WindowsResource::new();
            res.set_icon("../icons/icon.ico");

            if std::path::Path::new("C:/msys64/ucrt64/bin/windres.exe").exists() {
                res.set_windres_path("C:/msys64/ucrt64/bin/windres.exe");
            } else if std::path::Path::new("/ucrt64/bin/windres.exe").exists() {
                res.set_windres_path("/ucrt64/bin/windres.exe");
            }

            res.compile()
                .expect("Failed to compile Windows icon resource");
        }
    }
}
