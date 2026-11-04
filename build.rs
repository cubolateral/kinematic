fn main() {
    println!("cargo:rerun-if-changed=assets/brand/logo.ico");

    #[cfg(windows)]
    winresource::WindowsResource::new()
        .set_icon("assets/brand/logo.ico")
        .compile()
        .expect("Application icon must compile.");
}
