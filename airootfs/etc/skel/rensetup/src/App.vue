<script setup>
import { invoke } from "@tauri-apps/api/core";
</script>
<script>
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
