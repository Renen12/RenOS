use std::{process::Command, thread};

use tauri::{AppHandle, Manager};
#[tauri::command]
async fn install_native_package(package: String) {
    let thread = thread::spawn(move || {
        Command::new("pkexec")
            .args(["pacman", "--noconfirm", "-S", package.as_str()])
            .status()
            .expect("Installing software failed");
    });
    thread.join().unwrap();
}
#[tauri::command]
async fn install_flatpak_package(package: String) {
    let thread = thread::spawn(move || {
        Command::new("pkexec")
            .args([
                "flatpak",
                "install",
                "--noninteractive",
                "-y",
                package.as_str(),
            ])
            .status()
            .expect("Installing software failed");
    });
    thread.join().unwrap();
}
#[tauri::command]
async fn exit(app: AppHandle) {
    app.get_webview_window("main").unwrap().close().unwrap();
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            install_native_package,
            install_flatpak_package,
            exit
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
