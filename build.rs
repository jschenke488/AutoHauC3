#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.set("ProductName", "AutoHauC3");
    res.set("FileDescription", "AutoHauC3");
    res.compile().unwrap();
}

#[cfg(not(target_os = "windows"))]
fn main() {}