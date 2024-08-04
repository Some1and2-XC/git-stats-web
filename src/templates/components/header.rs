use maud::{html, Markup, PreEscaped};

use super::super::icon;

pub fn header() -> Markup {
    return html! {
        div #header
            // hidden
            style=r#"
            display: flex;
            position: fixed;

            justify-content: space-around;
            align-items: center;

            z-index: 5;

            width: 100%;

            color: var(--primary-color);
            background-color: var(--background-color);

            padding: 5px;
            border-bottom: 1px solid var(--primary-color);
            /*
            box-shadow: 2px 5px 15px -5px var(--primary-color);
            */
        "#{
            a href="/" style=r#"
                display: flex;
                color: var(--primary-color);
            "# {
                (icon())
                h1 {
                    "T-DY"
                }
            }

            div style="display: flex; gap: 15px;" {
                a href="/login" { p {
                    "Login"
                } }

                a href="/sign-up" { p {
                    "Sign-Up"
                } }
            }
        }
    };
}

pub fn header_spacer() -> Markup {
    return html! {
        div style=(format!("height: {}px", get_header_spacer_size())) {}
    };
}

/// Returns component that makes the header hide when scrolled at the top of the page.
pub fn header_hidden_on_top() -> Markup {
    return html! {

        script {(PreEscaped("
            window.onscroll = (e) => {
                let element = document.getElementById('header');

                if (window.scrollY > 20) {
                    element.removeAttribute('hidden');
                } else {
                    element.setAttribute('hidden', '');
                }
            }

            document.getElementById('header').setAttribute('hidden', '');
        "))}

    }
}

/// Function for getting the size of the header in pixeles for all device sizes
pub fn get_header_spacer_size() -> usize {
    return 86;
}
