use actix_web::{http::StatusCode, web, HttpRequest};
use maud::{html, Markup, PreEscaped};
use url::Url;
use serde::Deserialize;
use super::super::{
    WithBase,
    header,
    header_hidden_on_top,
    icon,
    super::errors::AppError,
};

#[derive(Debug, Deserialize)]
pub struct RepoUrl {
    pub url: String,
}

// pub async fn calendar(path: web::Path<(String, String, String)>) -> Markup {
pub async fn calendar(req: HttpRequest) -> Result<Markup, AppError> {

    let params = match web::Query::<RepoUrl>::from_query((&req).query_string()) {
        Ok(v) => v,
        Err(_) => {
            return Err(AppError {
                cause: Some(format!("Can't parse `RepoUrl` from get request parameter. Parameters: `{}`", req.query_string())),
                message: Some("Can't parse repo from get request parameter".to_string()),
                error_type: StatusCode::BAD_REQUEST,
            });
        },
    };

    let full_url = match Url::parse(&params.0.url) {
        Ok(v) => v,
        Err(e) => {
            let message = match e {
                url::ParseError::RelativeUrlWithoutBase => "Can't parse URL from provided string: missing encoding!",
                _ => "Can't parse URL from provided string!",
            };

            return Err(AppError {
                cause: Some(format!("Can't Parse URL from provided string. Parameters: `{}`", req.query_string())),
                message: Some(message.to_string()),
                error_type: StatusCode::BAD_REQUEST,
            });
        },
    };

    let path = (&full_url).path().trim_matches('/');

    return Ok(html! {
        (header())
        (header_hidden_on_top())
        div style=r#"
            display: flex;
            align-items: center;
            justify-content: left;
            width: 100%;
        "# {
            (icon())
            div {
                h1 {
                    a #calendar-source target="_blank" href=(full_url) {
                        (path)
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
                updateCalendar("/api/repo/?url={full_url}");
                "#)))
        }
    }.template_base());
}
