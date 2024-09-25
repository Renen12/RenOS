use aur_rs::{self, Request};
use std::{
    env, fs, io,
    process::{exit, Command},
};
pub fn install(pkgname: &String, outdir: String, noconfirm: bool) {
    if noconfirm == false {
        println!("Installing {}, is this okay? [Y/n]", pkgname);
        let mut answer = String::new();
        io::stdin().read_line(&mut answer).unwrap();
        let answer = answer.replace("\n", "").replace(" ", "").to_uppercase();
        if answer == "N" {
            exit(1);
        }
    }
    match Command::new("doas")
        .args([
            "sh",
            "-c",
            format!("rm -rv {} &> /dev/null", &outdir).as_str(),
        ])
        .status()
    {
        Ok(_) => (),
        Err(_) => (),
    };
    // just move on
    match fs::create_dir(&outdir) {
        Ok(_) => (),
        Err(_) => (),
    };
    env::set_current_dir("/tmp/aur").expect("Invalid output directory");
    let gitcmd = Command::new("git")
        .args([
            "clone",
            ("https://aur.archlinux.org/".to_owned() + pkgname + ".git").as_str(),
            "--quiet",
        ])
        .status()
        .expect("Failed to execute git, do you have it installed?");
    if gitcmd.code().expect("Failed to retrieve git exit code!") == 128 {
        eprintln!("No such package was found!");
        exit(1);
    }
    env::set_current_dir(&outdir).expect("Invalid output directory");
    Command::new("makepkg")
        .arg("-si")
        .status()
        .expect("Failed to make package:");
    exit(0);
}
pub fn remove(package: &String) {
    Command::new("pacman")
        .args(["-R", &package, "--noconfirm"])
        .status()
        .expect("Failed to execute the pacman command");
}
pub async fn search(packagename: &String) {
    let request = Request::default();
    let response = request
        .search_package_by_name(packagename)
        .await
        .expect("Failed to retrieve package information");
    if response.results.len() == 0 {
        eprintln!("No such package was found!");
        exit(1);
    }
    for package in response.results {
        let package = package.name;
        println!("{package}");
    }
}
