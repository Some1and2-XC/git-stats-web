use maud::{html, Markup};

use super::super::{
    WithBase,
    header,
    home_carousel,
};

pub async fn home() -> Markup {
    return html! {
        (header())
        (home_carousel())
        h1 {
            "This is the homepage!"
        }
        p {
            "Try "
            a href="/repo/github.com/some1and2-xc/git-stats" {
                "example page?"
            }
        }
    }.template_base();
}
