use std::{
    env,
    fs::read_dir,
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
    let thread = thread::spawn(|| {
        let user = env::var("USER").unwrap();
        Command::new("sh")
            .args(["-c", "rustup default stable"])
            .status()
            .expect("Failed to install rust");
        Command::new("sh").args([ "-c", format!("cd /home/{}/.local/renos && git clone https://aur.archlinux.org/paru && cd paru && makepkg -s --noconfirm", &user).as_str()]).status().expect("Failed to install the arch linux user repository helper");
        for file in read_dir(format!("/home/{}/.local/renos/paru", &user)).unwrap() {
            let file = file.unwrap();
            let name = file.file_name();
            if name.clone().into_string().unwrap().contains(".pkg.tar.zst") {
                Command::new("pkexec")
                    .args([
                        "pacman",
                        "-U",
                        format!(
                            "/home/{}/.local/renos/paru/{}",
                            &user,
                            name.into_string().unwrap()
                        )
                        .as_str(),
                        "--noconfirm",
                    ])
                    .status()
                    .unwrap();
            }
        }
        Command::new("paru")
            .args([
                "-S",
                "--sudo",
                "pkexec",
                "zed-preview-bin",
                "gnome-shell-extension-clipboard-indicator",
                "gnome-shell-extension-blur-my-shell",
                "gnome-shell-extension-appindicator-git",
                "--noconfirm",
            ])
            .status()
            .unwrap();
    });
    thread.join().unwrap();
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
#[tauri::command]
async fn gdm_logo_fix() {
    Command::new("pkexec")
        .args([
            "--user",
            "gdm",
            "dbus-launch",
            "gsettings",
            "set",
            "org.gnome.login-screen",
            "logo",
            "'/usr/share/pixmaps/RenOS.svg'",
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
            install_other,
            gdm_logo_fix
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
