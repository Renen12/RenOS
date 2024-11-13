use std::{
    env,
    io::{self, Write},
    process::Command,
    thread,
};
use tauri::{AppHandle, Manager};
#[tauri::command]
fn backend_msg(msg: String) {
    io::stdout().write((msg + "\n").as_bytes()).unwrap();
}
#[tauri::command]
fn has_internet() -> bool {
    let cmd = Command::new("ping")
        .args(["-c", "1", "8.8.8.8"])
        .status()
        .unwrap();
    if cmd.code().unwrap() >= 1 {
        return false;
    } else {
        return true;
    }
}
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
#[tauri::command]
async fn install_other() {
    let user = env::var("USER").unwrap();
    Command::new("arch-chroot").args(["/mnt", "sh", "-c",  format!("export HOME=/home/{} && export XDG_CONFIG_HOME=/home/{}/.config && export XDG_CACHE_HOME=/home/{}/.cache && rustup default stable", &user, &user, &user).as_str()]).status().expect("Failed to install rust");
    Command::new("arch-chroot").args(["-u", &user, "/mnt", "sh", "-c", format!("export HOME=/home/{} && export XDG_CONFIG_HOME=/home/{}/.config && export XDG_CACHE_HOME=/home/{}/.cache && cd /home/{}/.local/renos && git clone https://aur.archlinux.org/paru && cd paru && makepkg -s --noconfirm", &user, &user, &user, &user).as_str()]).status().expect("Failed to install the arch linux user repository helper");
    Command::new("arch-chroot")
        .args([
            "/mnt",
            "sh",
            "-c",
            format!(
                "cd /home/{}/.local/renos/paru && pacman -U *.pkg.tar.zst --noconfirm",
                &user
            )
            .as_str(),
        ])
        .status()
        .expect("Failed to install the aur helper");
    // Install additional AUR packages
    Command::new("arch-chroot").args(["/mnt", "-u", &user, "sh", "-c", format!("export HOME=/home/{} && export XDG_CONFIG_HOME=/home/{}/.config && export XDG_CACHE_HOME=/home/{}/.cache && paru -S zed-preview-bin gnome-shell-extension-clipboard-indicator gnome-shell-extension-blur-my-shell --noconfirm", &user, &user, &user).as_str()]).status().expect("Failed to install additional software");
}
#[tauri::command]
fn remove_startup_desktopentry() {
    #[allow(deprecated)]
    Command::new("pkexec")
        .args([
            "rm",
            (env::home_dir()
                .unwrap()
                .as_path()
                .to_str()
                .unwrap()
                .to_string()
                + "/.config/autostart/setup-renos.desktop")
                .as_str(),
        ])
        .status()
        .unwrap();
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            install_native_package,
            install_flatpak_package,
            exit,
            has_internet,
            backend_msg,
            remove_startup_desktopentry,
            install_other
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}