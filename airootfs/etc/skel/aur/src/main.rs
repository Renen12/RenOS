use std::{env, process::exit};

use pkgm::{install, search};
mod pkgm;
use tokio;
fn help_message(code: i32) {
    eprintln!("-H - Print this message \n -S [PACKAGE] - install the specified package \n -R [PACKAGE] - remove the specified package \n --noconfirm - skip confirmation \n -Ss [PACKAGE]- search for a package");
    exit(code);
}
#[allow(unused_must_use)]
#[tokio::main]
async fn main() {
    let mut noconfirm = false;
    match std::fs::create_dir("/tmp/aur") {
        Ok(_) => (),
        Err(_) => (),
    };
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    for arg in &args {
        if arg == "--noconfirm" {
            noconfirm = true;
        }
    }
    if args.len() < 1 {
        eprintln!("Not enough arguments were provided.");
        eprintln!("{}:", env::args().collect::<Vec<String>>()[0]);
        help_message(1);
    }
    if args[0] == "-S" {
        if args.len() < 2 {
            eprintln!("No package was supplied to -S!");
            exit(1);
        }
        install(
            &args[1],
            "/tmp/aur/".to_string() + &args[1].as_str(),
            noconfirm,
        );
    } else if args[0] == "-H" {
        help_message(0);
    } else if args[0] == "-R" {
        if args.len() < 2 {
            eprintln!("No package was supplied to -R!");
            exit(1);
        }
        // let pacman remove the package
        pkgm::remove(&args[1]);
        exit(0);
    } else if args[0] == "-Ss" {
        if args.len() < 2 {
            eprintln!("No package was supplied to -Ss!");
            exit(1);
        }
        search(&args[1]).await;
    } else {
        eprintln!("Invalid argument was passed!");
        exit(1);
    }
}
