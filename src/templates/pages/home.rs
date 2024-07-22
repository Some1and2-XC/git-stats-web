use maud::{html, Markup};
use super::super::{
    WithBase,
    header,
    example_section,
};

pub async fn home() -> Markup {
    return html! {
        (header())
        (example_section())
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
