mod app;
mod masked_field;
mod state;
mod ui;
mod windows_ops;

#[cfg(windows)]
fn main() -> std::io::Result<()> {
    app::run_app()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Unsupported platform: this app runs on Windows only.");
}
