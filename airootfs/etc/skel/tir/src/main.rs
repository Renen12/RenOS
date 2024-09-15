use concatenator::cat;
use std::process::exit;
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    process::Command,
};
fn install_graphics(typeofgpu: String) {
    if typeofgpu == "NVIDIA" {
        Command::new("arch-chroot")
            .args(["/mnt", "pacman", "-S", "nvidia-open"])
            .status()
            .expect("Failed to install nvidia graphics drivers");
    } else if typeofgpu == "INTEL" {
        Command::new("arch-chroot")
            .args(["/mnt", "pacman", "-S", "mesa"])
            .status()
            .expect("Failed to install intel graphics drivers");
    } else if typeofgpu == "AMD" {
        Command::new("arch-chroot")
            .args(["/mnt", "pacman", "-S", "mesa"])
            .status()
            .expect("Failed to install amd graphics drivers");
    } else if typeofgpu == "NONE" {
        println!("Why do you not have a graphics card?");
    } else {
        println!("{} is not a valid option.", typeofgpu);
        install_graphics(typeofgpu);
    }
}
fn select_locale(view: bool) -> String {
    if view == true {
        println!(
            "{}",
            fs::read_to_string("/etc/locale.gen")
                .expect("Failed to get the contents of /etc/locale.gen")
                .replace("#", "")
        );
        println!("What locale do you want to use?");
        let mut answer = String::new();
        io::stdin().read_line(&mut answer).unwrap();
        let answer = answer.replace("\n", "");
        return answer;
    } else {
        println!("What locale do you want to use?");
        let mut answer = String::new();
        io::stdin().read_line(&mut answer).unwrap();
        let answer = answer.replace("\n", "");
        return answer;
    }
}
fn do_all_the_things(skippartitioning: bool) {
    if !skippartitioning {
        println!("What drive do you want to install to? e.g /dev/sda");
        let mut disk = String::new();
        io::stdin()
            .read_line(&mut disk)
            .expect("Invalid input parsed.");
        let disk = disk.replace("\n", "");
        partition_drive(&disk);
    }
    println!("Please enter the partition used for the operating system:");
    let mut syspart = String::new();
    io::stdin()
        .read_line(&mut syspart)
        .expect("Failed to parse the syspartition from stdin.");
    let syspart = syspart.replace("\n", "");
    format_sys(&syspart);
    println!("Please enter the partition used for the swap:");
    let mut swappart = String::new();
    io::stdin()
        .read_line(&mut swappart)
        .expect("Failed to get swap partition name from stdin:");
    let swappart = swappart.replace("\n", "");

    format_swap(&swappart);
    println!("Please enter the partition used for the bootloader(efi partition):");
    let mut efipart = String::new();
    io::stdin()
        .read_line(&mut efipart)
        .expect("Failed getting efi partition name from stdin.");
    let efipart = efipart.replace("\n", "");
    format_efi(&efipart);
    install_system(&syspart, &efipart, &swappart).unwrap();
}
fn partition_drive(drive: &String) {
    Command::new("/sbin/cfdisk")
        .arg(drive)
        .status()
        .expect("Partitioning disks failed.");
}
fn format_sys(syspart: &String) {
    Command::new("mkfs.ext4")
        .arg(syspart)
        .status()
        .expect("Failed to format system partition.");
}
fn format_swap(swappart: &String) {
    Command::new("mkswap")
        .arg(swappart)
        .status()
        .expect("Failed to format swap partition.");
}
fn format_efi(efipart: &String) {
    Command::new("mkfs.fat")
        .args(["-F", "32", &efipart])
        .status()
        .expect("Failed to format efi partition.");
}
fn install_system(rootpart: &String, efipart: &String, swappart: &String) -> io::Result<()> {
    Command::new("mount")
        .args([rootpart, "/mnt"])
        .status()
        .expect("Failed to mount root partition on /mnt.");
    Command::new("mount")
        .args(["--mkdir", efipart, "/mnt/boot"])
        .status()
        .expect("Failed mounting efi partition.");
    Command::new("swapon")
        .arg(swappart)
        .status()
        .expect("Failed to swapon swap partition.");
    println!("Installing base system.");
    Command::new("pacstrap")
        .args([
            "-K",
            "/mnt",
            "base",
            "linux",
            "linux-firmware",
            "amd-ucode",
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
            "steam",
            "gnome-backgrounds",
        ])
        .status()
        .expect("Failed to install base system:");
    println!("Generating fstab!");
    let mut file =
        fs::File::create("/tmp/fstab.sh").expect("Failed to create temporary fstab script!");
    file.write("genfstab -U /mnt >> /mnt/etc/fstab".as_bytes())
        .expect("Failed to write to temporary fstab script!");
    Command::new("bash")
        .arg("/tmp/fstab.sh")
        .status()
        .expect("Executing temporary fstab script failed!");
    println!("What timezone do you want to use? (e.g Europe/Stockholm, Time zones are in /usr/share/zoneinfo)");
    let mut timezone = String::new();
    io::stdin().read_line(&mut timezone).unwrap();
    let timezone = timezone.replace("\n", "").replace(" ", "");
    Command::new("arch-chroot")
        .args([
            "/mnt",
            "ln",
            "-sf",
            format!("/usr/share/zoneinfo/{}", timezone).as_str(),
            "/etc/localtime",
        ])
        .status()
        .expect("Failed to set timezone.");
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/mnt/etc/locale.gen")
        .expect("Failed to open locale.gen file:");
    println!("Do you want to view the available locales? [Y/n]");
    let mut answer = String::new();
    io::stdin().read_line(&mut answer).unwrap();
    let answer = answer.replace("\n", "").replace(" ", "").to_lowercase();
    if answer == "Y" {
        let locale = select_locale(true);
        let locale = locale.as_str();
        if let Err(e) = writeln!(file, "{}", locale) {
            eprintln!("Couldn't write to locale file: {}", e);
        }
    } else if answer != "N" {
        let locale = select_locale(true);
        let locale = locale.as_str();
        if let Err(e) = writeln!(file, "{}", locale) {
            eprintln!("Couldn't write to locale file: {}", e);
        }
    } else {
        let locale = select_locale(false);
        let locale = locale.as_str();
        if let Err(e) = writeln!(file, "{}", locale) {
            eprintln!("Couldn't write to locale file: {}", e);
        }
    }
    println!("What language locale do you want to use? (same as the other locale but omitting the UTF-8 at the end (e.g en_GB.UTF-8 instead of en_GB.UTF-8 UTF-8)");
    let mut langlocale = String::new();
    io::stdin().read_line(&mut langlocale).unwrap();
    let langlocale = langlocale.replace("\n", "");
    fs::write("/mnt/etc/locale.conf", format!("LANG={}", langlocale))
        .expect("Failed to write to locale language configuration file:");
    println!("Press enter or any other key AND enter to view the available keymaps...");
    Command::new("localectl")
        .args(["list-keymaps"])
        .status()
        .expect("Failed to run localectl to see available keymaps");
    println!("What keyboard layout do you want to use for the console? (This won't apply to the installed desktop enviroment)");
    let mut keymap = String::new();
    io::stdin().read_line(&mut keymap).unwrap();
    let keymap = keymap.replace("\n", "");
    fs::write("/mnt/etc/vconsole.conf", format!("KEYMAP={}", keymap))
        .expect("Failed to write to console keyboard configuration file:");
    println!("What hostname do you want to use?");
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .expect("Failed reading from stdin:");
    let answer = answer.replace("\n", "").replace(" ", "").to_lowercase();
    fs::write("/mnt/etc/hostname", answer).expect("Failed writing to hostname file:");
    println!("Setting root password!");
    Command::new("arch-chroot")
        .args(["/mnt", "passwd"])
        .status()
        .expect("Failed to set root password:");
    println!("What username do you want to use for the installation?");
    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .expect("Failed to read from stdin:");
    let mut name = name.replace("\n", "").replace(" ", "").to_lowercase();
    if name == String::new() {
        name = String::from("renen");
    }
    Command::new("arch-chroot")
        .args([
            "/mnt",
            "useradd",
            "-m",
            "-d",
            cat("/home/", &name).as_str(),
            "-G",
            "wheel",
            &name,
        ])
        .status()
        .expect("Failed to create user.");
    println!("Changing password for {}", &name);
    Command::new("arch-chroot")
        .args(["/mnt", "passwd", &name])
        .status()
        .expect("Failed to change password for user.");
    fs::copy("/etc/os-release", "/mnt/etc/os-release")
        .expect("Failed to copy os release information");

    println!("Installing grub and efibootmgr.");
    Command::new("arch-chroot")
        .args(["/mnt", "pacman", "-S", "--noconfirm", "grub", "efibootmgr"])
        .status()
        .expect("Failed to install grub and efibootmgr package:");
    Command::new("arch-chroot")
        .args([
            "/mnt",
            "grub-install",
            "--target=x86_64-efi",
            "--efi-directory=/boot",
            "--bootloader-id=RenOS",
        ])
        .status()
        .expect("Failed to install bootloader:");
    fs::write(
        "/mnt/etc/default/grub",
        fs::read_to_string("/mnt/etc/default/grub")
            .expect("Failed to read grub config file")
            .replace("Arch", "RenOS"),
    )
    .expect("Failed setting grub branding");
    Command::new("arch-chroot")
        .args(["/mnt", "grub-mkconfig", "-o", "/boot/grub/grub.cfg"])
        .status()
        .expect("Failed to generate grub config:");
    println!(
        "Current grub configuration file: {}",
        fs::read_to_string("/mnt/boot/grub/grub.cfg").expect("No grub configuration file found:")
    );
    println!("Enabling gdm.");
    Command::new("arch-chroot")
        .args(["/mnt", "systemctl", "enable", "gdm"])
        .status()
        .expect("Failed to enable gdm.");
    println!("Enabling NetworkManager.");
    Command::new("arch-chroot")
        .args(["/mnt", "systemctl", "enable", "NetworkManager"])
        .status()
        .expect("Failed to enable NetworkManager.");
    fs::write("/mnt/etc/doas.conf", "permit persist :wheel \n")
        .expect("Failed to write to doas.conf!");
    println!("Installing yay!");
    Command::new("arch-chroot")
        .args([
            "/mnt",
            "pacman",
            "-S",
            "--noconfirm",
            "--needed",
            "base-devel",
            "git",
        ])
        .status()
        .expect("Failed to install git and base-devel");
    Command::new("arch-chroot")
        .args([
            "/mnt",
            "sh",
            "-c",
            format!("echo \"{} ALL=(ALL:ALL) ALL\" >> /etc/sudoers", &name).as_str(),
        ])
        .status()
        .expect("Failed to add user to the sudoers file!");
    Command::new("arch-chroot")
        .args([
            "-u",
            &name,
            "/mnt",
            "sh",
            "-c",
            format!(
                "export HOME=/home/{} && cd /home/{} && doas chown {} . && git clone https://aur.archlinux.org/yay.git && cd yay && makepkg -si", &name, &name, &name
            ).as_str(),
        ])
        .status()
        .expect("Failed to install yay");
    fs::remove_dir_all(format!("/home/{}/yay", &name))
        .expect("Failed to delete the yay build directory");
    println!("Do you have the Asus USB-AC58 WiFi adapter? [y/N]");
    let mut answer = String::new();
    io::stdin().read_line(&mut answer).unwrap();
    let answer = answer.replace("\n", "").replace(" ", "").to_uppercase();
    if answer == "Y" {
        Command::new("arch-chroot")
            .args(["/mnt", "pacman", "-S", "dkms"])
            .status()
            .expect("Failed to install dkms");
        Command::new("arch-chroot")
            .args([
                "-u",
                &name,
                "/mnt",
                "yay",
                "-S",
                "rtl88x2bu-cilynx-dkms-git",
            ])
            .status()
            .expect("Failed to install the rtl88x2bu drivers");
    }
    println!("What graphics card do you have? (Nvidia, Intel, Amd, None)");
    let mut gpu = String::new();
    io::stdin().read_line(&mut gpu).unwrap();
    let gpu = gpu.replace("\n", "").replace(" ", "").to_uppercase();
    install_graphics(gpu);
    println!("System installed. You may now reboot.");
    exit(0);
}
fn main() {
    println!("Tir testing");
    let mut answer = String::new();
    println!("Do you want to partition your disks? (Y/n)");
    io::stdin()
        .read_line(&mut answer)
        .expect("Invalid input parsed.");
    let answer = answer.replace("\n", "");
    if answer.to_lowercase() == "y" {
        do_all_the_things(false)
    } else {
        do_all_the_things(true)
    };
}
