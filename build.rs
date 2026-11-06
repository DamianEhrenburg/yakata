fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").map(|s| s == "windows").unwrap_or(false) {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/favicon.ico");
        res.compile().unwrap();
    }
}
