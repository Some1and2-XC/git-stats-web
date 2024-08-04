function toggle_theme() {
    var new_theme = "light";

    if (document.documentElement.getAttribute("data-scheme") === "light") {
        new_theme = "dark";
    }

    set_theme(new_theme);
    window.sessionStorage.setItem("data-scheme", new_theme);
}

function set_theme(new_theme) {
    document.documentElement.setAttribute("data-scheme", new_theme);
}

let saved_theme = window.sessionStorage.getItem("data-scheme");

if (saved_theme !== null) {
    set_theme(saved_theme);
}
