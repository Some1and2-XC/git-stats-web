use maud::{html, Markup};
use super::super::Css;

pub fn repo_card_list() -> Markup {
    return html! {

        (Css("/static/repo_card.css"))

        .repo_card_list {
            @for _i in 0..15 {
                (repo_card("https://avatars.githubusercontent.com/u/89313812?v=4", "The Linux Kernel", "Desc", "Linus Torvals", "https://github.com/torvalds"))
            }
        }

    }
}

pub fn repo_card(img: &str, title: &str, description: &str, author: &str, author_url: &str) -> Markup {
    return html! {
        .repo_card {

            div {
                img src=(img) alt="Repo Image" {}
            }

            div {
                h1 { (title) }
                p { (description) }
                a href=(author_url) { (author) }
            }

        }

    };
}
