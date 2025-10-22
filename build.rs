fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("ui/psc_logo.ico"); // path to your .ico
        res.compile().expect("Failed to compile icon resource");
    }
}