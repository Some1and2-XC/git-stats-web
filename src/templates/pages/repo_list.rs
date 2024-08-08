use maud::{html, Markup};
use super::super::{
    WithBase,
    header,
    header_spacer,
    components::repo_card_list,
};

pub async fn repo_list() -> Markup {
    return html!(
        (header())
        (header_spacer())

        (repo_card_list())

    ).template_base();
}
