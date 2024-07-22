use maud::{html, Markup};

pub fn example_section() -> Markup {
    return html!{

        div.carousel style=r#"
            height: 100vh;
        "# {
            @for idx in 0..3 {
                (dummy_data(&format!("{idx}")))
            }
        }

        hr {}

        script {r#"
                document.addEventListener('DOMContentLoaded', function() {
                    var elems = document.querySelectorAll('.carousel');
                    var instances = M.Carousel.init(elems, {
                        numVisible: 3,
                    });
                });
            "#
        }
    };
}

fn dummy_data(content: &str) -> Markup {
    return html! {
        div.carousel-item style=r#"
            background-color: red;
            color: black;

            width: 60vw;
            height: 46.363636364vw;
            aspect-ratio: 8.5 / 11;

            border-radius: 15px;

        "# {
            p {
                "Some content n:"
                (content)
            }
        }
    };
}
