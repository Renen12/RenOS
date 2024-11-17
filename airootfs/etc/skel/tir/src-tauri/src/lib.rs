use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    process::{Command, ExitStatus},
    sync::Mutex,
    thread,
};

use tauri::{AppHandle, Emitter, Manager};
fn write_to_log(status: usize) {
    let mut options = match OpenOptions::new()
        .append(true)
        .create(true)
        .write(true)
        .read(true)
        .open("/home/live/tir.log")
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    options
        .write(format!("\n {}", status.to_string()).as_bytes())
        .unwrap();
}
#[tauri::command]
async fn restore_renos(syspart: String, efipart: String, app: AppHandle) {
    println!("Restoring RenOS");
    let cmd = match Command::new("mount")
        .args([syspart, String::from("/mnt")])
        .status()
    {
        Ok(v) => v,
        Err(_) => {
            emit_err(&app);
            return;
        }
    };
    probe_cmd_err(cmd, &app);
    let cmd = match Command::new("mount")
        .args(["--mkdir", efipart.as_str(), "/mnt/boot"])
        .status()
    {
        Ok(v) => v,
        Err(_) => {
            emit_err(&app);
            return;
        }
    };
    probe_cmd_err(cmd, &app);
    let app_mutex = Mutex::new(app.clone());
    let thread = thread::spawn(move || {
        let cmd = match Command::new("arch-chroot")
            .args(["/mnt", "sh", "-c", "pacman -Qqn | pacman -S - --noconfirm"])
            .status()
        {
            Ok(v) => v,
            Err(_) => {
                emit_err(&app_mutex.lock().unwrap());
                return;
            }
        };
        probe_cmd_err(cmd, &app_mutex.lock().unwrap());
        let cmd = match Command::new("arch-chroot")
            .args([
                "/mnt",
                "sh",
                "-c",
                "grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=RenOS",
            ])
            .status()
        {
            Ok(v) => v,
            Err(_) => {
                emit_err(&app_mutex.lock().unwrap());
                return;
            }
        };
        probe_cmd_err(cmd, &app_mutex.lock().unwrap());
        match fs::copy("/etc/os-release", "/mnt/etc/os-release") {
            Ok(_) => (),
            Err(_) => {
                app.get_webview_window("main")
                    .unwrap()
                    .emit("failed", ())
                    .unwrap();
            }
        };
        match fs::write(
            "/mnt/etc/lsb-release",
            "DISTRIB_ID=\"renos\" \n DISTRIB_RELEASE=\"rolling\" \n DISTRIB_DESCRIPTION=\"RenOS\"",
        ) {
            Ok(_) => (),
            Err(_) => app
                .get_webview_window("main")
                .unwrap()
                .emit("failed", ())
                .unwrap(),
        }
        match fs::copy(
            "/home/live/RenOS.svg",
            "/mnt/usr/share/pixmaps/archlinux-logo-text-dark.svg",
        ) {
            Ok(_) => (),
            Err(_) => emit_err(&app),
        }
    });
    thread.join().unwrap();
    reboot();
}
#[tauri::command]
fn exit(app: AppHandle) {
    app.exit(0);
}
#[tauri::command]
fn reboot() {
    Command::new("reboot").status().unwrap();
}
fn emit_err(app: &AppHandle) {
    app.get_webview_window("main")
        .unwrap()
        .emit("failed", ())
        .unwrap();
}
#[tauri::command]
async fn intel_graphics(app: AppHandle) {
    let app = Mutex::new(app.clone());
    let thread = thread::spawn(move || {
        let cmd = Command::new("arch-chroot")
            .args([
                "/mnt",
                "pacman",
                "-S",
                "mesa",
                "vulkan-intel",
                "vulkan-icd-loader",
                "--noconfirm",
            ])
            .status()
            .expect("Failed to install intel graphics drivers");
        probe_cmd_err(cmd, &app.lock().unwrap());
    });
    thread.join().unwrap();
}
#[tauri::command]
async fn nvidia_graphics(app: AppHandle) {
    let app = Mutex::new(app);
    let thread = thread::spawn(move || {
        let cmd = Command::new("arch-chroot")
            .args([
                "/mnt",
                "pacman",
                "-S",
                "nvidia-open",
                "nvidia-utils",
                "vulkan-icd-loader",
                "--noconfirm",
            ])
            .status()
            .expect("Failed to install Nvidia graphics drivers");
        probe_cmd_err(cmd, &app.lock().unwrap());
    });
    thread.join().unwrap();
}
#[tauri::command]
async fn amd_graphics(app: AppHandle) {
    let app = Mutex::new(app);
    let thread = thread::spawn(move || {
        let cmd = Command::new("arch-chroot")
            .args([
                "/mnt",
                "pacman",
                "-S",
                "mesa",
                "amdvlk",
                "vulkan-icd-loader",
                "--noconfirm",
            ])
            .status()
            .expect("Failed to install AMD graphics drivers");
        probe_cmd_err(cmd, &app.lock().unwrap());
    });
    thread.join().unwrap();
}
fn probe_cmd_err(cmd: ExitStatus, app: &AppHandle) {
    if cmd.code().unwrap() != 0 {
        let window = app.get_webview_window("main").unwrap();
        window.emit("failed", ()).unwrap();
        write_to_log(cmd.code().unwrap() as usize);
    }
}
async fn install_grub(app: AppHandle) {
    // Install the bootloader
    let app_mutex = Mutex::new(app.clone());
    let thread = thread::spawn(move || {
        let cmd = Command::new("arch-chroot")
            .args(["/mnt", "pacman", "-S", "--noconfirm", "grub", "efibootmgr"])
            .status()
            .expect("Failed to install the RenOS bootloader package");
        probe_cmd_err(cmd, &app_mutex.lock().unwrap());
        let cmd = Command::new("arch-chroot")
            .args([
                "/mnt",
                "grub-install",
                "--target=x86_64-efi",
                "--efi-directory=/boot",
                "--bootloader-id=RenOS",
            ])
            .status()
            .expect("Failed to install the GRUB bootloader");
        probe_cmd_err(cmd, &app_mutex.lock().unwrap());
        match fs::write(
            "/mnt/etc/default/grub",
            fs::read_to_string("/mnt/etc/default/grub")
                .expect("Failed to read the GRUB default configuration file")
                .replace("Arch", "RenOS")
                + "\n GRUB_DISABLE_OS_PROBER=false",
        ) {
            Ok(_) => (),
            Err(_) => {
                app_mutex
                    .lock()
                    .unwrap()
                    .get_webview_window("main")
                    .unwrap()
                    .emit("failed", ())
                    .unwrap();
            }
        };
        let cmd = Command::new("arch-chroot")
            .args(["/mnt", "grub-mkconfig", "-o", "/boot/grub/grub.cfg"])
            .status()
            .expect("Failed to generate the GRUB configuration file");
        probe_cmd_err(cmd, &app_mutex.lock().unwrap());
    });
    thread.join().unwrap();
}
#[tauri::command]
async fn monolithic_the_rest(user: String, app: AppHandle) {
    install_grub(app.clone()).await;
    match fs::copy("/etc/os-release", "/mnt/etc/os-release") {
        Ok(_) => (),
        Err(_) => {
            app.get_webview_window("main")
                .unwrap()
                .emit("failed", ())
                .unwrap();
        }
    };
    match fs::create_dir_all(format!("/home/{}/Templates", &user)) {
        Ok(_) => (),
        Err(_) => emit_err(&app),
    }
    match fs::write(format!("/home/{}/Templates/Text File.txt", &user), "") {
        Ok(_) => (),
        Err(_) => emit_err(&app),
    }
    match fs::write(
        "/mnt/etc/lsb-release",
        "DISTRIB_ID=\"renos\" \n DISTRIB_RELEASE=\"rolling\" \n DISTRIB_DESCRIPTION=\"RenOS\"",
    ) {
        Ok(_) => (),
        Err(_) => app
            .get_webview_window("main")
            .unwrap()
            .emit("failed", ())
            .unwrap(),
    }
    match fs::copy("/home/live/Logo.svg", "/mnt/usr/share/pixmaps/renos.svg") {
        Ok(_) => (),
        Err(_) => emit_err(&app),
    }
    let cmd = Command::new("arch-chroot")
        .args(["/mnt", "systemctl", "enable", "gdm"])
        .status()
        .expect("Failed to enable the login manager");
    probe_cmd_err(cmd, &app);
    let cmd = Command::new("arch-chroot")
        .args(["/mnt", "systemctl", "enable", "NetworkManager"])
        .status()
        .expect("Failed to enable the networking software");
    probe_cmd_err(cmd, &app);
    match fs::write("/mnt/etc/doas.conf", "permit persist :wheel \n") {
        Ok(_) => (),
        Err(_) => {
            app.get_webview_window("main")
                .unwrap()
                .emit("failed", ())
                .unwrap();
        }
    };
    match fs::copy("/home/live/RenOS.svg", "/mnt/usr/share/pixmaps/RenOS.svg") {
        Ok(_) => (),
        Err(_) => {
            emit_err(&app);
        }
    }
    match fs::create_dir_all("/mnt/etc/dconf/db/gdm.d") {
        Ok(_) => (),
        Err(_) => emit_err(&app),
    }
    match fs::write(
        "/mnt/etc/dconf/db/gdm.d/02-logo",
        "[org/gnome/login-screen]
    logo='/usr/share/pixmaps/RenOS.svg' \n",
    ) {
        Ok(_) => (),
        Err(_) => emit_err(&app),
    }
    let cmd = Command::new("arch-chroot")
        .args(["/mnt", "dconf", "update"])
        .status()
        .expect("Failed to update the GDM database");
    probe_cmd_err(cmd, &app);
    match fs::copy(
        "/home/live/RenOS.svg",
        "/mnt/usr/share/pixmaps/archlinux-logo-text-dark.svg",
    ) {
        Ok(_) => (),
        Err(_) => emit_err(&app),
    }
    match fs::create_dir_all(format!("/mnt/home/{}/.local/bin/", &user)) {
        Ok(_) => (),
        Err(_) => emit_err(&app),
    };
    match fs::copy(
        "/home/live/.local/bin/rensetup",
        format!("/mnt/home/{}/.local/bin/rensetup", &user),
    ) {
        Ok(_) => (),
        Err(_) => emit_err(&app),
    }
    match fs::create_dir_all(format!("/mnt/home/{}/.config/autostart", &user)) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Autostart err: {e}");
            emit_err(&app);
        }
    }
    match fs::copy(
        "/home/live/.local/share/applications/Setup.desktop",
        format!("/mnt/home/{}/.config/autostart/setup-renos.desktop", &user),
    ) {
        Ok(_) => (),
        Err(_) => emit_err(&app),
    }
    let cmd = Command::new("chmod")
        .args([
            "777",
            format!("/mnt/home/{}/.local/bin/rensetup", &user).as_str(),
        ])
        .status()
        .expect("Failed to set proper setup software permissions");
    probe_cmd_err(cmd, &app);
    match fs::create_dir_all(format!("/mnt/home/{}/.local/renos", &user)) {
        Ok(_) => (),
        Err(_) => emit_err(&app),
    }
    let cmd = match Command::new("arch-chroot")
        .args([
            "/mnt",
            "chown",
            &user,
            format!("/home/{}/.local/", &user).as_str(),
        ])
        .status()
    {
        Ok(v) => v,
        Err(_) => {
            emit_err(&app);
            return;
        }
    };
    probe_cmd_err(cmd, &app);
    let cmd = match Command::new("arch-chroot")
        .args([
            "/mnt",
            "chmod",
            "777",
            format!("/home/{}/.local/", &user).as_str(),
            "-R",
        ])
        .status()
    {
        Ok(v) => v,
        Err(_) => {
            emit_err(&app);
            return;
        }
    };
    probe_cmd_err(cmd, &app);
    let app_mutex = Mutex::new(app.clone());
    thread::spawn(move || {
        unsafe {
            println!("{GLOBAL_LOCALE}");
            match fs::write(
                "/mnt/home/".to_owned() + &user + "/.bashrc",
                format!(
                    "
             [[ $- != *i* ]] && return
             alias ls=\'ls --color=auto\'
             alias grep=\'grep --color=auto\'
             PS1=\'\\u@\\H in \\w; \\t >>> \'
             eval \"$(zoxide init bash)\"
             alias cd=\'z\'",
                ),
            ) {
                Ok(_) => (),
                Err(_) => emit_err(&app_mutex.lock().unwrap()),
            }
        }
        probe_cmd_err(cmd, &app_mutex.lock().unwrap());
    });
    app.get_webview_window("main")
        .unwrap()
        .emit("gputrigger", ())
        .unwrap();
}
#[tauri::command]
fn run_gparted() {
    Command::new("gparted")
        .spawn()
        .expect("Failed to open gparted!");
}
#[tauri::command]
fn set_passwd(user: String, app: AppHandle, password: String) {
    let cmd = Command::new("arch-chroot")
        .args([
            "/mnt",
            "sh",
            "-c",
            format!("echo '{}:{}'|chpasswd", user, password).as_str(),
        ])
        .status()
        .expect("Failed to change the user's password");
    probe_cmd_err(cmd, &app);
}
#[tauri::command]
fn set_config_perms(user: String, app: AppHandle) {
    let cmd = Command::new("arch-chroot")
        .args([
            "/mnt",
            "chown",
            "-R",
            &user,
            &("/home/".to_owned() + user.as_str()),
        ])
        .status()
        .expect("Failed chowning home dir");
    probe_cmd_err(cmd, &app);
    let cmd = Command::new("arch-chroot")
        .args([
            "/mnt",
            "chown",
            "-R",
            &user,
            format!("/home/{}/.config", &user).as_str(),
        ])
        .status()
        .expect("Failed to set proper .config permissions");
    probe_cmd_err(cmd, &app);
    let cmd = match Command::new("arch-chroot")
        .args([
            "/mnt",
            "chmod",
            "770",
            format!("/home/{}/.config", &user).as_str(),
        ])
        .status()
    {
        Ok(v) => v,
        Err(_) => {
            emit_err(&app);
            return;
        }
    };
    probe_cmd_err(cmd, &app);
}
#[tauri::command]
fn create_user(user: String, app: AppHandle) {
    let cmd = Command::new("arch-chroot")
        .args([
            "/mnt",
            "useradd",
            "-m",
            "-d",
            ("/home/".to_owned() + user.as_str()).as_str(),
            "-G",
            "wheel",
            &user,
        ])
        .status()
        .expect("Failed to create a user");
    probe_cmd_err(cmd, &app);
    println!(
        "{}",
        String::from_utf8(
            Command::new("cat")
                .args(["/etc/passwd"])
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap()
                .stdout
        )
        .unwrap()
    );
    // Arch Linux base-devel developers, please make sudo not a dependency of base-devel.
    let mut cmd = Command::new("arch-chroot")
        .args([
            "/mnt",
            "sh",
            "-c",
            "echo   \"%wheel    ALL=(ALL)   ALL >> /etc/sudoers\"",
        ])
        .spawn()
        .unwrap();
    cmd.wait().unwrap();
}
#[tauri::command]
fn return_partitions(app: AppHandle) -> Vec<String> {
    let cmd = match Command::new("sh")
        .args(["-c", "fdisk -l | grep '^/dev' | cut -d' ' -f1"])
        .output()
    {
        Ok(v) => v,
        Err(_) => {
            emit_err(&app);
            return Vec::new();
        }
    };
    let output = String::from_utf8(cmd.stdout).unwrap();
    let mut vec: Vec<String> = Vec::new();
    for partition in output.split("\n") {
        vec.push(partition.to_string());
    }
    println!("{:?}", vec);
    return vec;
}
#[tauri::command]
fn format_syspart(partition: String, app: AppHandle) {
    println!("{partition}");
    let cmd = Command::new("sh")
        .args(["-c", format!("yes | mkfs.ext4 {}", partition).as_str()])
        .status()
        .expect("Failed formatting the system partition");
    probe_cmd_err(cmd, &app);
}
#[tauri::command]
fn format_swappart(partition: String, app: AppHandle) {
    println!("{partition}");
    let cmd = Command::new("sh")
        .args(["-c", format!("yes | mkswap {}", partition).as_str()])
        .status()
        .expect("Failed to format swap partition");
    probe_cmd_err(cmd, &app);
}
#[tauri::command]
fn format_efipart(partition: String, app: AppHandle) {
    println!("{partition}");
    let cmd = Command::new("sh")
        .args(["-c", format!("yes | mkfs.fat -F 32 {}", partition).as_str()])
        .status()
        .expect("Failed to format the efi(bootloader) partition");
    probe_cmd_err(cmd, &app);
}
#[tauri::command]
async fn install_system(rootpart: String, efipart: String, swappart: String, app: AppHandle) {
    let cmd = Command::new("mount")
        .args([rootpart, "/mnt".to_string()])
        .status()
        .expect("Failed to mount the root partition");
    probe_cmd_err(cmd, &app);
    let cmd = Command::new("mount")
        .args(["--mkdir", efipart.as_str(), "/mnt/boot"])
        .status()
        .expect("Failed to mount the efi partition");
    probe_cmd_err(cmd, &app);
    let cmd = Command::new("swapon")
        .arg(swappart)
        .status()
        .expect("Failed to activate the swap device");
    probe_cmd_err(cmd, &app);
    println!("Installing the base operating system, this can take a while!");
    let app_mutex = Mutex::new(app.clone());
    let thread = thread::spawn(move || {
        let mut install_cmd = match Command::new("pacstrap")
            .args([
                "-K",
                "/mnt",
                "base",
                "linux",
                "linux-firmware",
                "nano",
                "networkmanager",
                "gnome-desktop",
                "nautilus",
                "gdm",
                "rustup",
                "gnome-console",
                "vivaldi",
                "gnome-control-center",
                "opendoas",
                "bash",
                "gnome-backgrounds",
                "github-cli",
                "flatpak",
                "totem",
                "gnome-disk-utility",
                "gnome-text-editor",
                "wl-clipboard",
                "dconf-editor",
                "bash-completion",
                "loupe",
                "gnome-calculator",
                "gnome-software",
                "lib32-vulkan-icd-loader",
                "zoxide",
                "vulkan-tools",
                "less",
                "gnome-packagekit",
                "os-prober",
                "gnome-initial-setup",
                "git",
                "base-devel",
                "power-profiles-daemon",
                "gnome-console",
                "nautilus-image-converter",
                "gnome-tweaks",
                "sushi",
                "realtime-privileges",
                "icu",
                "webkit2gtk-4.1",
                "webkit2gtk",
            ])
            .spawn()
        {
            Ok(v) => v,
            Err(_) => {
                emit_err(&app_mutex.lock().unwrap());
                return;
            }
        };
        install_cmd.wait().unwrap();
    });
    thread.join().unwrap();
    let cmd = Command::new("sh")
        .args(["-c", "genfstab -U /mnt >> /mnt/etc/fstab"])
        .status()
        .expect("Failed writing to the fstab");
    probe_cmd_err(cmd, &app);
    app.get_webview_window("main")
        .unwrap()
        .emit("languageselection", ())
        .unwrap();
}
#[tauri::command]
fn get_timezones() -> Vec<String> {
    let cmd = Command::new("sh")
        .args([
            "-c",
            "cd /usr/share/zoneinfo/posix && find * -type f -or -type l | sort",
        ])
        .output()
        .expect("Failed to get timezones");
    let output = String::from_utf8(cmd.stdout.to_vec()).unwrap();
    return output.split("\n").map(|s| s.to_string()).collect();
}
#[tauri::command]
fn set_timezone(timezone: String, app: AppHandle) {
    let cmd = Command::new("arch-chroot")
        .args([
            "/mnt",
            "ln",
            "-sf",
            format!("/usr/share/zoneinfo/{}", timezone).as_str(),
            "/etc/localtime",
        ])
        .status()
        .expect("Failed to set timezone");
    probe_cmd_err(cmd, &app);
}
#[tauri::command]
fn backend_msg(message: String) {
    io::stdout().write((message + "\n").as_bytes()).unwrap();
}
static mut GLOBAL_LOCALE: String = String::new();
#[tauri::command]
fn write_to_localegen(app: AppHandle, lang: String) {
    unsafe {
        GLOBAL_LOCALE = lang.clone();
    }
    println!("Writing locale {lang}");
    let mut localegenfile = match OpenOptions::new()
        .append(true)
        .write(true)
        .open("/mnt/etc/locale.gen")
    {
        Ok(v) => v,
        Err(_) => {
            app.get_webview_window("main")
                .unwrap()
                .emit("failed", ())
                .unwrap();
            return;
        }
    };
    match localegenfile.write(lang.as_bytes()) {
        Ok(_) => (),
        Err(_) => {
            app.get_webview_window("main")
                .unwrap()
                .emit("failed", ())
                .unwrap();
            return;
        }
    };
    match fs::write(
        "/mnt/etc/locale.conf",
        format!(
            "LANG={}",
            match lang.split(" ").collect::<Vec<&str>>().get(0) {
                Some(v) => v,
                None => {
                    emit_err(&app);
                    return;
                }
            }
        ),
    ) {
        Ok(_) => (),
        Err(_) => {
            app.get_webview_window("main")
                .unwrap()
                .emit("failed", ())
                .unwrap();
        }
    };
    let cmd = Command::new("arch-chroot")
        .args(["/mnt", "locale-gen"])
        .status()
        .expect("Failed to generate the locales");
    probe_cmd_err(cmd, &app);
}
#[tauri::command]
fn get_locales(app: AppHandle) -> Vec<String> {
    let locales = match fs::read_to_string("/usr/share/i18n/SUPPORTED") {
        Ok(v) => v,
        Err(_) => {
            app.get_webview_window("main")
                .unwrap()
                .emit("failed", ())
                .unwrap();
            String::new()
        }
    };
    let locales = locales.split("\n").map(|s| s.to_string()).collect();
    locales
}
#[tauri::command]
fn set_hostname(app: AppHandle, hostname: String) {
    match fs::write("/mnt/etc/hostname", hostname) {
        Ok(_) => (),
        Err(_) => {
            app.get_webview_window("main")
                .unwrap()
                .emit("failed", ())
                .unwrap();
        }
    };
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            run_gparted,
            format_syspart,
            format_swappart,
            format_efipart,
            install_system,
            get_timezones,
            set_timezone,
            backend_msg,
            get_locales,
            write_to_localegen,
            set_hostname,
            create_user,
            set_passwd,
            monolithic_the_rest,
            nvidia_graphics,
            amd_graphics,
            intel_graphics,
            reboot,
            exit,
            set_config_perms,
            return_partitions,
            restore_renos
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
