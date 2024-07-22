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
            html lang="en" {
                head {
                    title {
                        "Git Stats"
                    }
                    meta charset="utf-8" {}
                    link rel="icon" type="image/svg+xml" href="/static/icon.svg" {}
                    (Css("/static/index.css"))

                    // NPM full calendar
                    script src="https://cdn.jsdelivr.net/npm/fullcalendar@6.1.13/index.global.min.js" {}
                    script src="/static/index.js" {}

                    // Compiled and minified CSS
                    link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/materialize/1.0.0/css/materialize.min.css" {}

                    // Compiled and minified JavaScript
                    script src="https://cdnjs.cloudflare.com/ajax/libs/materialize/1.0.0/js/materialize.min.js" {}


                }
                body {
                    (self)
                }
            }
        };
    }
}
