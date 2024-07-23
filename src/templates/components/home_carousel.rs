use maud::{html, Markup, PreEscaped};

pub fn home_carousel() -> Markup {
    return html!{

        div.carousel style=r#"

            padding-top: 91px;
            height: 100vh; /* Subtracts the height of the navbar */

            background: var(--background-color);

            /* background: linear-gradient(13deg, rgba(20,27,28,1) 0%, rgba(51,153,173,1) 100%); */
        "# {
            @for idx in 0..3 {
                (dummy_data(idx))
            }
        }

        hr {}

        script {(PreEscaped(r#"

                $(document).ready(() => {
                    $('.carousel').carousel({
                        numVisible: 3,
                    });
                });

            "#))}
    };
}

fn dummy_data(content: usize) -> Markup {

    let urls = [
        "https://github.com/Some1and2-XC/git-stats/blob/main/examples/may_12-18_2024.png?raw=true",
        "https://github.com/Some1and2-XC/git-stats/blob/main/examples/server.png?raw=true",
    ];

    let url = urls[content % urls.len()];

    return html! {
        div.carousel-item style=r#"
            width: 60vw;
            height: 46.363636364vw;
            /* aspect-ratio: 8.5 / 11; */
        "#{
            img
                src=(url)
                style=r#"
                    width: 100%;
                    height: 100%;

                    object-fit: cover;

                    border-radius: 15px;

                "# {}
        }
    };
}
