use maud::{html, Markup, PreEscaped};

pub fn example_section() -> Markup {
    return html!{

        div.carousel data-indicators="true" style=r#"
            height: 100vh;
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
            aspect-ratio: 8.5 / 11;
        "#{
            img
                src=(url)
                style=r#"
                    background-color: red;
                    color: black;

                    width: 100%;
                    height: 100%;

                    object-fit: cover;

                    border-radius: 15px;

                "# {}
        }
    };
}
