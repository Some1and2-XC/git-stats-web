use maud::{html, Markup};
use super::WithBase;

pub async fn calendar() -> Markup {
    return html! {
        div style=r#"
            display: flex;
            align-items: center;
            justify-content: left;
            width: 100%;
        "# {
            object type="image/svg+xml" data="/static/icon.svg" style="height: 5rem; padding-right: 1rem;" {}
            div {
                h1 {
                    a id="title" target="_blank" {
                        "Loading..."
                    }
                }
            }
        }
        hr {}
        div id="calendar" {}
        code class="bottom-message" {
            "This report was automatically generated"
        }
    }.template_base();
}
