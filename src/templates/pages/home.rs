use actix_session::Session;
use actix_web::web::Data;
use log::debug;
use maud::{html, Markup};

use crate::auth::SESSION_USER_ID_KEY;

use super::super::{
    WithBase,
    header,
    header_spacer,
    home_carousel,
};

pub async fn home(session: Session) -> Markup {

    return html! {
        /* (header()) */ 
        /* (header_spacer()) */
        /* (home_carousel()) */

        form.search_box action="/repo" {
            .search_bar {
                input name="url" placeholder="Enter Git URL" {
                }
            }
        }

        /*
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
        */

        /*
        article {
            p { b { "T-DY: Transform Your Commit History into Insights" }}

            p { "Unlock the power of your git commit history with T-DY, the innovative app that turns your development journey into a visual and analytical masterpiece. Whether you’re a solo coder, part of a team, or managing multiple projects, T-DY offers indispensable features to streamline your workflow and enhance productivity." }

            p { b { "Key Features:" } }

            ul {
                li { p { "Calendar Visualization: Your git commit history beautifully displayed in a calendar format, showing your coding activity over time." } }
                li { p { "Insight Generation: Gain deep insights into your coding patterns, including languages used, commit frequency, and project timelines." } }
                li { p { "Automated Reports: Automatically generate reports and summaries of your development progress. Effortlessly track milestones and project evolution." } }
                li { p { "Email Notifications: Stay connected and informed with automated email notifications for important milestones, commit summaries, and insights." } }
                li { p { "Collaboration Tools: Enhance teamwork with collaborative features that allow teams to visualize and analyze collective coding efforts." } }
                li { p { "Customizable Settings: Tailor your experience with customizable settings for privacy, notifications, and report formats." } }
                li { p { "Integration Ready: Seamlessly integrates with popular version control platforms like GitHub, GitLab, and Bitbucket." } }
            }

            p { b { "Why T-DY?" } }

            p { "T-DY empowers developers and teams to understand their coding habits, improve productivity, and celebrate accomplishments. Whether you’re a freelancer, a startup, or an enterprise, T-DY is your go-to tool for turning code commits into actionable insights." }

            p { b { "Get Started Today" } }

            p { "Join thousands of developers worldwide who rely on T-DY to streamline their workflow and gain valuable insights from their git repositories. Start visualizing your coding journey like never before." }
        }
        */

    }.template_base();
}
