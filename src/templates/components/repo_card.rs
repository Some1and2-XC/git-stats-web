use maud::{html, Markup, PreEscaped};
use super::super::Css;

pub fn repo_card_list() -> Markup {
    return html! {

        (Css("/static/repo_card.css"))

        .repo_card_list {
            /*
            @for _i in 0..15 {
                (repo_card("https://avatars.githubusercontent.com/u/89313812?v=4", "The Linux Kernel", "Desc", "Linus Torvals", "https://github.com/torvalds"))
            }
            */
        }

        script { (PreEscaped(r#"

            let url = "https://api.github.com/users/some1and2-xc/repos";
            let repo_card_list = document.getElementsByClassName("repo_card_list")[0];
            var out_string = "";

            fetch(url)
                .then(data => data.json())
                .then(arr => {
                    for (i in arr) {
                        out_string += `
                            <a class="repo_card" href="/repo?url=${arr[i].html_url}">
                                <div>
                                    <img src="${arr[i].owner.avatar_url}" alt="Repo Image">
                                </div>
                                <div>
                                    <h1>${arr[i].name}</h1>
                                    <h6>${arr[i].description}</h6>
                                    <span>${arr[i].owner.login}</span>
                                </div>
                            </a>
                        `;
                    }

                    repo_card_list.innerHTML = out_string;
                });
        "#)) }

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
