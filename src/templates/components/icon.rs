use maud::{html, Markup};

pub fn icon() -> Markup {
    return html! {
        object type="image/svg+xml" data="/static/icon.svg" style="height: 5rem; padding-right: 1rem;" {}
    };
}
