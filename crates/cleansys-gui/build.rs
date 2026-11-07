//! Windows: embed the app icon into `cleansys-gui.exe`, so Explorer, the taskbar and the
//! installer's shortcuts show it. Other targets need nothing here.

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=assets/cleansys.ico");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/cleansys.ico")
            .compile()?;
    }
    Ok(())
}
