// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use passharbor_lib::prelude::*;

fn main() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        // Wayland just doesn't boot unless i do this.
        unsafe {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
    passharbor_lib::run();
    Ok(())
}
