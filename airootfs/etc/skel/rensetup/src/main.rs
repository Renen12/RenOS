use std::process::Command;
use std::{env, fs};

use gtk::{glib, Application, ApplicationWindow};
use gtk::{prelude::*, Label};

const APP_ID: &str = "net.ren-net.welcome";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);
    fs::remove_file(env::var("HOME").unwrap() + "/.config/autostart/setup-renos.desktop")
        .expect("Can't remove the renos setup desktop file");
    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create a window and set the title
    let contents = gtk::Box::builder().build();
    #[derive(PartialEq)]
    enum SoftwareType {
        Native,
        Flatpak,
    }
    struct Program {
        name: String,
        typeofsoftware: SoftwareType,
        id: Option<String>,
    }
    let software: Vec<Program> = vec![
        Program {
            name: String::from("discord"),
            typeofsoftware: SoftwareType::Flatpak,
            id: Some(String::from("com.discordapp.Discord")),
        },
        Program {
            name: String::from("code"),
            typeofsoftware: SoftwareType::Native,
            id: None,
        },
        Program {
            name: String::from("spotify"),
            typeofsoftware: SoftwareType::Flatpak,
            id: Some(String::from("com.spotify.Client")),
        },
        Program {
            name: String::from("supertuxkart"),
            typeofsoftware: SoftwareType::Native,
            id: None,
        },
    ];
    for program in software {
        let check = gtk::CheckButton::builder().build();
        if &program.name == "code" {
            check.set_label(Some("vscode"));
        } else {
            check.set_label(Some(&program.name));
        }
        check.connect_toggled(move |btn| {
            if btn.is_active() {
                println!("installing {}", &program.name);
                if program.typeofsoftware == SoftwareType::Native {
                    Command::new("pkexec")
                        .args(["pacman", "-S", &program.name, "--noconfirm"])
                        .spawn()
                        .unwrap();
                } else if program.typeofsoftware == SoftwareType::Flatpak {
                    Command::new("sh")
                        .args([
                            "-c",
                            format!("flatpak install -y --user {}", program.id.as_ref().unwrap())
                                .as_str(),
                        ])
                        .spawn()
                        .unwrap();
                }
            } else {
                println!("uninstalling {}", &program.name);
                if program.typeofsoftware == SoftwareType::Native {
                    Command::new("pkexec")
                        .args(["pacman", "-R", &program.name])
                        .spawn()
                        .unwrap();
                } else if program.typeofsoftware == SoftwareType::Flatpak {
                    Command::new("flatpak")
                        .args(["-y", "remove", program.id.as_ref().unwrap().as_str()])
                        .spawn()
                        .unwrap();
                }
            }
        });
        contents.append(&check);
    }
    contents.set_margin_top(100);
    contents.set_margin_bottom(100);
    contents.set_margin_end(100);
    contents.set_margin_start(100);
    let label = Label::builder().build();
    label.set_text("Select the desired extra software below:");
    label.set_margin_bottom(300);
    label.set_margin_start(-450);
    contents.append(&label);
    let window = ApplicationWindow::builder()
        .application(app)
        .title("RenOS first-time setup")
        .width_request(500)
        .height_request(500)
        .build();
    // Present window
    window.set_child(Some(&contents));
    window.present();
    window.connect_close_request(|_| {
        println!("closing");
        std::process::exit(0);
    });
}
