<script setup>
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
document.getElementById("restore").onclick = async () => {
    document.getElementById("yes").remove();
    document.getElementById("no").remove();
    document.getElementById("action").innerHTML =
        "Please select the SYSTEM partition you want to restore:";
    await reset_renos();
};
async function reset_renos() {
    let restorepartitions = {
        syspart: "debugerror",
        efipart: "debugerror",
    };
    document.body.innerHTML =
        document.body.innerHTML +
        `<select name="partitions" id="partitions"> </select> <br> <input type="submit" value="Confirm" id="confirm">`;
    invoke("return_partitions").then((listofpartitions) => {
        listofpartitions.forEach((v) => {
            if (v.replace(" ", "") != "") {
                let e = document.createElement("option");
                e.value = "0";
                e.innerHTML = v;
                document.getElementById("partitions").appendChild(e);
            }
        });
    });
    document.getElementById("confirm").onclick = async () => {
        document.getElementById("action").innerHTML =
            "Please select the BOOT(EFI) partition you want to restore:";
        let partitions = document.getElementById("partitions");
        let selected = partitions.options[partitions.selectedIndex].text;
        console.log(selected);
        restorepartitions["syspart"] = selected;
        document.getElementById("confirm").onclick = async () => {
            let partitions = document.getElementById("partitions");
            let selected = partitions.options[partitions.selectedIndex].text;
            console.log(selected);
            restorepartitions["efipart"] = selected;
            document.getElementById("action").innerHTML =
                "Please wait while RenOS is being reset...";
            await invoke("restore_renos", {
                syspart: restorepartitions["syspart"],
                efipart: restorepartitions["efipart"],
            }).then(async (_) => {
                await invoke("reboot");
            });
        };
    };
}
let partitions_obj = {
    efipart: "",
    syspart: "",
    swappart: "",
};
let user = "";
async function the_rest() {
    document.getElementById("action").innerHTML =
        "Installing additional software...";
    await invoke("monolithic_the_rest", { user: user });
}
function pick_sys_partition() {
    document.getElementById("action").innerHTML =
        "What partition do you want to install the operating system on?";
    document.getElementById("yes").remove();
    document.getElementById("no").remove();
    document.body.innerHTML =
        document.body.innerHTML +
        `<select name="partitions" id="partitions"> </select> <br> <input type="submit" value="Install to" id="install">`;
    invoke("return_partitions").then((listofpartitions) => {
        listofpartitions.forEach((v) => {
            if (v.replace(" ", "") != "") {
                let e = document.createElement("option");
                e.value = "0";
                e.innerHTML = v;
                document.getElementById("partitions").appendChild(e);
            }
        });
    });
    async function proceed() {
        let partitions = document.getElementById("partitions");
        let selected = partitions.options[partitions.selectedIndex].text;
        console.log(selected);
        partitions_obj["syspart"] = selected;
        await invoke("format_syspart", {
            partition: selected,
        });
        format_swap();
    }
    document.getElementById("install").onclick = () => {
        proceed();
    };
}
async function set_root_passwd() {
    document.getElementById("action").innerHTML =
        "What password do you want to use for the administrator account(root)?";
    document.getElementById("confirmbtn").onclick = async () => {
        if (
            document.getElementById("password").value !=
            document.getElementById("passwordconfirm").value
        ) {
            alert("The passwords do not match.");
        } else {
            await invoke("set_passwd", {
                user: "root",
                password: document.getElementById("password").value,
            });
            await the_rest();
        }
    };
}
function set_user_passwd() {
    document.getElementById("action").innerHTML =
        "What password do you want to use for your account?";
    document.getElementById("usernameconfirmbtn").remove();
    document.getElementById("usernameinput").remove();
    document.body.innerHTML =
        document.body.innerHTML +
        `<input type="password" id="password">
        </input>
        <br>
        <input type="password" id="passwordconfirm">
        </input>
        <br>
        <button type="button" id="confirmbtn">Set password
        </button>`;
    document.getElementById("confirmbtn").onclick = () => {
        if (
            document.getElementById("password").value !=
            document.getElementById("passwordconfirm").value
        ) {
            alert("The passwords do not match.");
        } else {
            invoke("set_passwd", {
                user: user,
                password: document.getElementById("password").value,
            });
            set_root_passwd();
        }
    };
}
function set_user_name() {
    document.getElementById("confirmbtn").remove();
    document.getElementById("hostnameinput").remove();

    document.body.innerHTML =
        document.body.innerHTML +
        `<input type="text" id="usernameinput"></input> <button type="button" id="usernameconfirmbtn">Choose</button> `;
    document.getElementById("action").innerHTML =
        "What do you want your username to be?";
    document.getElementById("usernameconfirmbtn").onclick = () => {
        user = document.getElementById("usernameinput").value;
        invoke("create_user", {
            user: document.getElementById("usernameinput").value,
        });
        set_user_passwd();
    };
}
function confirmpartitioningmethod() {
    document.getElementById("action").innerHTML =
        "Do you want to partition your disks?";
    document.getElementById("yes").onclick = () => {
        invoke("run_gparted");
        document.getElementById("restore").remove();
        pick_sys_partition();
    };
    document.getElementById("no").onclick = () => {
        document.getElementById("restore").remove();
        pick_sys_partition();
    };
}
document.addEventListener("contextmenu", (event) => event.preventDefault());
function format_efi() {
    document.getElementById("action").innerHTML =
        "What partition do you want to use for the efi(bootloader)?";
    async function proceed() {
        document.getElementById("action").innerHTML =
            "Installing the base operating system, please wait...";
        let partitions = document.getElementById("partitions");
        let selected = partitions.options[partitions.selectedIndex].text;
        console.log(selected.replace(" ", ""));
        partitions["efipart"] = selected;
        invoke("backend_msg", {
            message: "Before formatting efi rust trigger",
        });
        invoke("format_efipart", {
            partition: selected,
        });
        partitions_obj["efipart"] = selected;
        document.getElementById("install").remove();
        document.getElementById("partitions").remove();
        await install_system();
        return;
    }
    document.getElementById("install").onclick = () => {
        proceed();
    };
}
function format_swap() {
    document.getElementById("action").innerHTML =
        "What partition do you want to use for the swap?";
    function proceed() {
        let partitions = document.getElementById("partitions");
        let selected = partitions.options[partitions.selectedIndex].text;
        console.log(selected);
        invoke("format_swappart", {
            partition: selected,
        });
        partitions_obj["swappart"] = selected;
        format_efi();
    }
    document.getElementById("install").onclick = () => {
        proceed();
    };
}
function everything() {
    confirmpartitioningmethod();
}
function set_hostname() {
    document.getElementById("languages").remove();
    document.querySelectorAll("li").forEach((listitem) => {
        listitem.remove();
    });
    document.querySelectorAll("button").forEach((btn) => {
        btn.remove();
    });
    document.getElementById("action").innerHTML =
        "What name would you like your computer to use when it talks to other computers?";
    document.body.innerHTML =
        document.body.innerHTML +
        `<input type="text" id="hostnameinput"></input>`;
    document.body.innerHTML =
        document.body.innerHTML +
        `<button type="button" id="confirmbtn">Set</button`;
    function proceed() {
        invoke("set_hostname", {
            hostname: document.getElementById("hostnameinput").value,
        });
        set_user_name();
    }
    document.getElementById("confirmbtn").onkeydown = (k) => {
        if (k.key == "Enter") {
            proceed();
        }
    };
    document.getElementById("confirmbtn").onclick = () => {
        proceed();
    };
}
function select_language() {
    document.getElementById("searchinput").remove();
    document.getElementById("search").remove();
    document.getElementById("timezones").id = "languages";
    document.querySelectorAll("button").forEach((btn) => {
        btn.remove();
    });
    document.querySelectorAll("li").forEach((listitem) => {
        listitem.remove();
    });
    document.getElementById("action").innerHTML =
        "What language would you like to use?";
    invoke("get_locales").then((langlist) => {
        langlist.forEach((lang) => {
            if (lang != "") {
                let newlistitem = document.createElement("li");
                newlistitem.innerHTML =
                    newlistitem.innerHTML +
                    `<button type="button"> ${lang} </button>`;
                newlistitem.onclick = () => {
                    invoke("write_to_localegen", { lang });
                    set_hostname();
                };
                document.getElementById("languages").appendChild(newlistitem);
            }
        });
    });
}
async function install_system() {
    console.dir(partitions_obj);
    await invoke("install_system", {
        rootpart: partitions_obj["syspart"],
        swappart: partitions_obj["swappart"],
        efipart: partitions_obj["efipart"],
    });
}
listen("failed", (_) => {
    alert(
        "An error has occured in a component of the operating system installer! The operating system may not correctly install!",
    );
});
listen("languageselection", (_) => {
    document.getElementById("action").innerHTML =
        "What timezone do you want to use?";
    document.body.innerHTML =
        document.body.innerHTML +
        `<input type="text" width=40 id="searchinput"> </input> <button type="button" id="search">Search</button>`;
    document.body.innerHTML =
        document.body.innerHTML + `<ul id="timezones"> </ul>`;
    invoke("get_timezones").then((list) => {
        list.forEach((timezone) => {
            if (timezone != "") {
                let newelement = document.createElement("li");
                newelement.innerHTML = `<button type="button"> ${timezone} </button>`;
                newelement.onclick = () => {
                    invoke("set_timezone", { timezone: timezone });
                    console.log(timezone);
                    select_language();
                };
                document.getElementById("timezones").appendChild(newelement);
            }
        });
        function proceed() {
            let elems = document.querySelectorAll("li");
            elems.forEach((item) => {
                item.remove();
            });
            list.forEach((zone) => {
                if (
                    zone
                        .toLowerCase()
                        .includes(
                            document
                                .getElementById("searchinput")
                                .value.toLowerCase(),
                        )
                ) {
                    invoke("backend_msg", { message: `${zone} matched` });
                    let newlist = document.createElement("li");
                    newlist.id = "timezone-list-component";
                    document.body.append(newlist);
                    let newelement = document.createElement("button");
                    newelement.type = "button";
                    newelement.innerHTML = zone;
                    newlist.appendChild(newelement);
                    newelement.onclick = () => {
                        invoke("set_timezone", { timezone: zone });
                        console.log(zone);
                        select_language();
                    };
                }
            });
        }
        document.getElementById("search").onclick = () => {
            proceed();
        };
        document.getElementById("searchinput").onkeydown = (k) => {
            if (k.key == "Enter") {
                proceed();
            }
        };
    });
});
listen("gputrigger", async () => {
    async function reboot() {
        if (confirm("The RenOS system is now installed! Reboot?")) {
            await invoke("backend_msg", { message: "User:" + user });
            await invoke("reboot");
        } else {
            await invoke("backend_msg", { message: "User:" + user });

            await invoke("exit");
        }
    }
    document.getElementById("password").remove();
    document.getElementById("passwordconfirm").remove();
    document.getElementById("confirmbtn").remove();
    document.getElementById("action").innerHTML =
        "What graphics card do you use?";
    document.body.innerHTML =
        document.body.innerHTML +
        `<button id="intel" type="button">Intel</button> <button id="nvidia" type="button">Nvidia</button> <button id="amd" type="button">AMD</button> <button id="nvidia-old" type="button"> Nvidia(Maxwell to lovelace(older graphics cards)) </button> `;
    function prepareforgpuwaiting() {
        document.getElementById("nvidia").remove();
        document.getElementById("amd").remove();
        document.getElementById("intel").remove();
        document.getElementById("nvidia-old").remove();
        document.getElementById("action").innerHTML =
            "Please wait for the graphics card drivers to be installed...";
    }
    document.getElementById("intel").onclick = async () => {
        prepareforgpuwaiting();
        await invoke("intel_graphics");
        await invoke("set_config_perms", { user: user });
        reboot();
    };
    document.getElementById("nvidia-old").onclick = async () => {
        prepareforgpuwaiting();
        await invoke("nvidia_old");
        await invoke("set_config_perms", { user: user });

        reboot();
    };
    document.getElementById("nvidia").onclick = async () => {
        prepareforgpuwaiting();
        await invoke("nvidia_graphics");
        await invoke("set_config_perms", { user: user });

        reboot();
    };
    document.getElementById("amd").onclick = async () => {
        prepareforgpuwaiting();
        await invoke("amd_graphics");
        await invoke("set_config_perms", { user: user });

        reboot();
    };
});
everything();
</script>
