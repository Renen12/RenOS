use concatenator::cat;
use std::process::exit;
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    process::Command,
};
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
    install_system(&syspart, &efipart, &swappart);
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
fn install_system(rootpart: &String, efipart: &String, swappart: &String) {
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
            "usermod",
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
    Command::new("arch-chroot")
        .args([
            "/mnt",
            "ln",
            "-sf",
            "/usr/share/zoneinfo/Europe/Stockholm",
            "/etc/localtime",
        ])
        .status()
        .expect("Failed to set timezone.");
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/mnt/etc/locale.gen")
        .expect("Failed to open locale.gen file:");

    if let Err(e) = writeln!(file, "en_GB.UTF-8 UTF-8") {
        eprintln!("Couldn't write to locale file: {}", e);
    }
    fs::write("/mnt/etc/locale.conf", "LANG=en_GB.UTF-8")
        .expect("Failed to write to locale language configuration file:");
    fs::write("/mnt/etc/vconsole.conf", "KEYMAP=sv-latin1")
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
        .args(["/mnt", "useradd", &name])
        .status()
        .expect("Failed to create user.");
    println!("Changing password for {}", &name);
    Command::new("arch-chroot")
        .args(["/mnt", "passwd", &name])
        .status()
        .expect("Failed to change password for user.");
    println!("Installing grub and efibootmgr.");
    Command::new("arch-chroot")
        .args(["/mnt", "pacman", "-S", "grub", "efibootmgr"])
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
    println!("Creating home directory.");
    Command::new("mkdir")
        .arg("/mnt/home/".to_owned() + &name)
        .status()
        .expect("Failed to create user home directory!");
    let newname = &name.clone().to_string();
    Command::new("arch-chroot")
        .args([
            "/mnt",
            "chown",
            &cat(cat(name, ":".to_string()), newname.to_string()),
            &cat("/home/", &newname),
        ])
        .status()
        .expect("Failed to set proper home directory permissions:");
    Command::new("arch-chroot")
        .args(["/mnt", "usermod", "-a", "-G", "wheel", &newname])
        .status()
        .expect("Failed adding user to the wheel group!");
    fs::write(
        "/etc/doas.conf",
        "permit setenv {PATH=/usr/local/bin:/usr/local/sbin:/usr/bin:/usr/sbin} :wheel",
    )
    .expect("Failed to write to doas.conf!");
    Command::new("chmod")
        .args(["777", "/mnt/etc/doas.conf"])
        .status()
        .expect("Failed to change permissions of doas.conf!");
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
