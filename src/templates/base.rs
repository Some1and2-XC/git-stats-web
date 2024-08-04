use maud::{html, Markup, DOCTYPE};
use super::Css;

pub trait WithBase {
    fn template_base(self) -> Self;
}

/// Base function for all html
impl WithBase for Markup {
    fn template_base(self) -> Self {
        return html! {
            (DOCTYPE)
            html lang="en" data-scheme="dark" {
                head {
                    title { "Git Stats" }
                    meta charset="utf-8" {}
                    link rel="icon" type="image/svg+xml" href="/static/icon.svg" {}
                    (Css("/static/index.css"))
                    (Css("/static/theme.css"))

                    // JQuery
                    script src="https://code.jquery.com/jquery-3.7.1.slim.min.js" integrity="sha256-kmHvs0B+OpCW5GVHUNjv9rOmY0IvSIRcf7zGUDTDQM8=" crossorigin="anonymous" {}

                    // NPM full calendar
                    script src="https://cdn.jsdelivr.net/npm/fullcalendar@6.1.13/index.global.min.js" {}
                    script src="/static/index.js" {}
                    script src="/static/theme.js" {}

                    // Marterialize Compiled and minified CSS
                    link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/materialize/1.0.0/css/materialize.min.css" {}

                    // Marterialize Compiled and minified JavaScript
                    script src="https://cdnjs.cloudflare.com/ajax/libs/materialize/1.0.0/js/materialize.min.js" {}

                    // Marterialize fonts
                    link href="https://fonts.googleapis.com/icon?family=Material+Icons" rel="stylesheet" {}

                    // Font Awesome
                    link rel="stylesheet" href="https://use.fontawesome.com/releases/v5.15.4/css/all.css" integrity="sha384-DyZ88mC6Up2uqS4h/KRgHuoeGwBcD4Ng9SiP4dIRy0EXTlnuz47vAwmeGwVChigm" crossorigin="anonymous" {}

                }
                body {
                    (self)
                }
            }
        };
    }
}
