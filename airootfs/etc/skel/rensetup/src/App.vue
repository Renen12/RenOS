<script setup>
import { invoke } from "@tauri-apps/api/core";
</script>
<script>
async function remove_startup_desktopentry() {
    await invoke("remove_startup_desktopentry");
}
remove_startup_desktopentry();
// Application data
let spotify = {
    name: "Spotify",
    id: "com.spotify.Client",
    type: "flatpak",
};
let steam = {
    id: "steam",
    name: "Steam",
    type: "native",
};
let vscode = {
    id: "code",
    name: "Visual Studio Code(OSS)",
    type: "native",
};
let stk = {
    id: "supertuxkart",
    name: "Super Tux Kart",
    type: "native",
};
async function check_internet() {
    while ((await invoke("has_internet")) != true) {
        alert(
            "An internet connection is required to install additional software. Please connect to the internet and try again!",
        );
    }
}
// Install additional software
document.getElementById("additional").innerHTML =
    "Please wait while non-essential improvements are being applied to RenOS...";
document.getElementById("continue").disabled = true;
async function install_other() {
    await check_internet().then(async () => {
        await invoke("install_other").then(async () => {
            await invoke("gdm_logo_fix");
            document.getElementById("additional").innerHTML =
                "Select the additional software you want below:";
            document.getElementById("continue").disabled = false;
        });
    });
}
install_other();
let software = [stk, vscode, steam, spotify];
software.forEach((program) => {
    document.body.innerHTML =
        document.body.innerHTML +
        `<label for=${program.id}>${program.name}</label>`;
    document.body.innerHTML =
        document.body.innerHTML + `<input id="${program.id}" type="checkbox">`;
    document.body.innerHTML = document.body.innerHTML + `<br>`;
});
document.body.innerHTML =
    document.body.innerHTML +
    `<button type="button" id="continue">Install</button>`;
document.getElementById("continue").onclick = () => {
    document.querySelectorAll("label").forEach(async (element) => {
        if (document.getElementById(element.getAttribute("for")).checked) {
            software.forEach(async (v) => {
                if (v.name == element.innerHTML) {
                    if (v.type == "native") {
                        console.log("native");
                        await invoke("install_native_package", {
                            package: v.id,
                        }).then(async () => {});
                    } else if (v.type == "flatpak") {
                        console.log("flatpak");
                        await invoke("install_flatpak_package", {
                            package: v.id,
                        }).then(async () => {});
                    }
                }
            });
        }
    });
};
</script>
