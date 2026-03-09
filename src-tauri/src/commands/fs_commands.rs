use std::process::Command;

#[tauri::command]
pub fn open_folder(path: String) {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path) // Windows explorer handles paths with spaces automatically in args
            .spawn()
            .ok();
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .ok();
    }
}