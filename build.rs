fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icons/icon48.ico"); // Ruta al archivo .ico
    res.compile().unwrap();
}