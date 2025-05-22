use std::path::Path;

fn main() {
    #[cfg(windows)]
    {
        // Set up the Windows resource
        let mut res = winres::WindowsResource::new();

        // Check if we already have an icon file
        let icon_path = Path::new("app_icon.ico");
        if !icon_path.exists() {
            println!("cargo:warning=No app_icon.ico found. Window icon will be used but exe icon will be default.");
        } else {
            res.set_icon("app_icon.ico");
        }

        res.compile().unwrap();
    }
}
