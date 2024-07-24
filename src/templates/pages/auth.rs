use maud::{html, Markup};
use super::super::{
    WithBase,
    header,
    header_hidden_on_top,
};

fn auth_card(title: &str, alt: Markup, url: &str, content: Markup) -> Markup {

    return html! {

        div style=r#"
            display: flex;

            flex: 1;
            flex-direction: column;
            align-items: center;
            align-content: center;
        "# {
            div style=r#"
                position: absolute;
                top: 40%;
                transform: translateY(-50%);

                border: 1px solid var(--primary-color);
                border-radius: 15px;
                box-shadow: 1px 1px 5px 0px rgba(255, 255, 255, .5)

                background-color: rgba(0, 0, 0, 0.6),

                width: 150px;
                max-width: 95vw;

                padding: 15px;
            "# {
                h1 style="margin: 0" {
                    (title)
                }
                (alt)

                /* Form submission data can be formatted here */
                form method="POST" action=(url) {
                    (content)
                    button.pillow-button
                        href=""
                        style="margin: 25px 0 0 0"
                        type="submit"
                    {
                        "Submit!"
                    }
                }
            }
        }
    };

}

fn make_alt_link(text: &str, url: &str) -> Markup {
    return html! {
        a href=(url) {
            (text)
        }
    };
}

pub async fn login() -> Markup {

    let page_content = html! {
        input name="email" type="text" placeholder="Enter Email" {}
        input name="password" type="password" placeholder="Enter Password" {}
        div style="display: flex; gap: 5px;" {
            label for="remember" { "Remember me" }
            input id="remember" type="checkbox" name="remember" checked {}
        }
    };

    return html! {

        (header())
        (header_hidden_on_top())
        (auth_card("Login", make_alt_link("Signup", "/sign-up"), "/login", page_content))
    }.template_base();
}

pub async fn signup() -> Markup {

    let page_content = html! {
        input name="email" type="text" placeholder="Enter Email" {}
        input name="username" type="text" placeholder="Enter Username" {}
        input name="password" type="password" placeholder="Enter Password" {}
        input name="password2" type="password" placeholder="Re-enter password" {}
        div style="display: flex; gap: 5px;" {
            label for="remember" { "Remember me" }
            input id="remember" type="checkbox" name="remember" checked {}
        }
    };


    return html! {
        (header())
        (header_hidden_on_top())
        (auth_card("SignUp", make_alt_link("Login", "/login"), "/sign-up", page_content))
    }.template_base();
}
