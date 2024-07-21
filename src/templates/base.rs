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
                    script src="https://cdn.jsdelivr.net/npm/fullcalendar@6.1.13/index.global.min.js" {}
                    script src="/static/index.js" {}
                }
                body {
                    (self)
                }
            }
        };
    }
}
