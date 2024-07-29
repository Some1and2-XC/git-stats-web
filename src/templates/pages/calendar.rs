use actix_web::web;
use maud::{html, Markup, PreEscaped};
use super::super::{
    WithBase,
    header,
    header_hidden_on_top,
};

pub async fn calendar(path: web::Path<(String, String, String)>) -> Markup {

    let (site, username, repo) = path.into_inner();
    let full_url = format!("https://{site}/{username}/{repo}");

    return html! {
        (header())
        (header_hidden_on_top())
        div style=r#"
            display: flex;
            align-items: center;
            justify-content: left;
            width: 100%;
        "# {
            object type="image/svg+xml" data="/static/icon.svg" style="height: 5rem; padding-right: 1rem;" {}
            div {
                h1 {
                    a #calendar-source target="_blank" href=(full_url) {
                        (username)
                        "/"
                        (repo)
                    }
                }
            }
        }
        hr {}
        #calendar {}
        code.bottom-message {
            "This report was automatically generated"
        }
        script {
            (PreEscaped(format!(r#"
                updateCalendar("/api/repo/{site}/{username}/{repo}");
                "#)))
        }
    }.template_base();
}
