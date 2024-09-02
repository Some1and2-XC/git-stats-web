use std::error::Error;

use actix_web::{web, HttpRequest};
use git_stats_web::aliases::Timestamp;
use maud::{html, Markup, PreEscaped};
use url::Url;
use serde::Deserialize;
use chrono::NaiveDate;

use super::super::{
    WithBase,
    icon,
    super::errors::AppError,
};

#[derive(Debug, Deserialize)]
pub struct RepoUrl {
    pub url: String,
    pub date_end: Option<NaiveDate>,
    pub date_start: Option<NaiveDate>,
    pub time_allowed: Option<Timestamp>,
}

// pub async fn calendar(path: web::Path<(String, String, String)>) -> Markup {
pub async fn calendar(req: HttpRequest) -> Result<Markup, AppError> {

    let params = match web::Query::<RepoUrl>::from_query((&req).query_string()) {
        Ok(v) => v,
        Err(e) => {
            return Ok(html! {
                h2 {
                    "Error!"
                }
                p {
                    (e.source().unwrap().to_string())
                }
            }.template_base());

            /*
            return Err(AppError {
                cause: Some(format!("Can't parse `RepoUrl` from get request parameter. Parameters: `{}`", req.query_string())),
                message: Some("Can't parse repo from get request parameter".to_string()),
                error_type: StatusCode::BAD_REQUEST,
            });
            */

        },
    };

    let full_url = match Url::parse(&params.0.url) {
        Ok(v) => v,
        Err(e) => {

            let output = Ok(html! {
                p { "Can't parse URL!" }
                a href="/" { "Home?" }
            }.template_base());


            if e != url::ParseError::RelativeUrlWithoutBase {
                return output;
            } else {
                // Tries parsing again but adding https encoding
                match Url::parse(&format!("https://{}", &params.0.url)) {
                    Ok(v) => v,
                    Err(_) => return output,
                }
            }

        },
    };

    let path = (&full_url).path().trim_matches('/');

    return Ok(html! {

        /*
        (header())
        (header_hidden_on_top())
        */

        div style=r#"
            display: flex;
            align-items: center;
            justify-content: left;
            width: 100%;
        "# {
            (icon())
            div style="flex: 3;" {
                h1 {
                    a #calendar-source target="_blank" href=(full_url) {
                        (path)
                    }
                }
            }
        }
        hr {}
        #calendar { }
        code.bottom-message {
            "Loading..."
        }
        script {
            (PreEscaped(format!(r#"
                updateCalendar("/api/repo/?url={full_url}");
                "#)))
        }
    }.template_base());
}
