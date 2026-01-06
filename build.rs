fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/app/windowlasso.ico");
        res.compile().expect("Failed to compile Windows resources");
    }
}
