use maud::{html, Markup};

pub fn icon() -> Markup {
    return html! {
        a.icon href="/" style="padding-right: 1rem; aspect-ratio: 1/1; height: 5rem; max-height: 5rem;" {}
    };
}
