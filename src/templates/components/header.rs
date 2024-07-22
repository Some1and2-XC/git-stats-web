use maud::{html, Markup};

use super::super::icon;

pub fn header() -> Markup {
    return html! {
        div #header hidden style=r#"

            display: flex;
            position: fixed;

            justify-content: center;
            align-items: center;

            z-index: 5;

            width: 100%;

            color: var(--primary-color);
            background-color: black;

            padding: 5px;
            border-bottom: 1px solid var(--primary-color);
            /*
            box-shadow: 2px 5px 15px -5px var(--primary-color);
            */
        "#{
            a href="/" style=r#"
                display: flex;
                color: white;
            "# {
                (icon())
                h1 {
                    "T-DY"
                }
            }
        }
    };
}
