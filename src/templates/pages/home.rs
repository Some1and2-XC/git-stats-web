use maud::{html, Markup};

use super::super::{
    WithBase,
    header,
    home_carousel,
};

pub async fn home() -> Markup {
    return html! {
        (header())
        (home_carousel())


        div.row {
            div.col.s12.m4 {
                div.card.black style="border: 1px solid white;" {
                    div.card-image.waves-effect.waves-block.waves-light {
                        img.activator src="https://images.pexels.com/photos/8872400/pexels-photo-8872400.jpeg" style="height: 250px; object-fit: cover;" {}
                    }
                    div.card-content {
                        span.card-title.activator {
                            "Automate Invoices"
                            i.material-icons.right {
                                "more_vert"
                            }

                        }
                        p {"Automate generating invoices directory from your commit history."}
                    }
                    div.card-action.blue-text {
                        a href="/repo/github.com/some1and2-xc/git-stats" {
                            "Example Page"
                        }
                        a href="#" {
                            "SignUp"
                        }
                    }
                    div.card-reveal.black {
                        span.card-title {
                            "Automate Invoices"
                            i.material-icons.right {
                                "close"
                            }
                        }
                        p {
                            "From periodically sending emails of progress to creating customized reports, we can help optimize your invoicing workflow."
                        }
                    }
                }
            }
        }

        p {
            "Try "
            a href="/repo/github.com/some1and2-xc/git-stats" {
                "example page?"
            }
        }
    }.template_base();
}
