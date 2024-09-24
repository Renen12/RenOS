use reqwest;
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
    let rmcmd = match Command::new("doas").args(["rm", "-rv", &outdir]).status() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
    };
    if rmcmd.code().expect("Failed to retrieve doas status code") == 1 {
        eprintln!("Failed to authenticate");
        exit(1);
    }
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
    let packagerequest = reqwest::get(
        "https://aur.archlinux.org/rpc/v5/search/".to_owned() + &packagename + "?by=name",
    )
    .await
    .expect("Failed to contact the aur api");
    let _packagerequest = packagerequest.text().await.unwrap();
    todo!("Search not implemented");
}
