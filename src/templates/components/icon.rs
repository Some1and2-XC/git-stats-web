use maud::{html, Markup};

pub fn icon() -> Markup {
    return html! {
        a.icon href="/" style="height: 5rem; width: 5rem; padding-right: 1rem;" {}
    };
}
