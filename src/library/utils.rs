use regex::Regex;
use url::Url;
use log::debug;

use git2::{DiffOptions, Repository};

use crate::{
    aliases::AnnotatedCalendarValue,
    cli::CliArgs,
    prediction::{PredictionAttributes, PredictionStructure},
    calendar::CalendarValue,
    git,
};

/// Gets the path of a url from URI
/// May modify the url to work with git@xyz:your/project domains
/// Rewrites them to https://xyz/your/project
/// ```rust
/// # use git_stats_web::utils::get_path;
/// let url = get_path("git@xyz:your/project");
/// assert_eq!(url.to_string(), "https://xyz/your/project".to_string());
/// ```
/// If not a git path, this method just parses out the path.
/// ```rust
/// # use git_stats_web::utils::get_path;
/// let url = get_path("https://xyz/your/project");
/// assert_eq!(url.to_string(), "https://xyz/your/project".to_string());
/// ```
pub fn get_path(src_url: &str) -> Url {
    let mut tmp_url = src_url.to_string();

    let re = Regex::new(r"git@(?<domain>.+):(?<path>.+)").unwrap();

    if let Some(caps) = re.captures(&tmp_url) {
        tmp_url = format!("https://{}/{}", &caps["domain"], &caps["path"]);
    }

    debug!("Parsing URL: {tmp_url}");
    return Url::parse(&tmp_url).unwrap();
}

/// Function for getting commit data and returning json
pub fn calculate_data(args: &CliArgs, repo: &Repository) -> Vec<CalendarValue> {

    let mut commit_arr: Vec<AnnotatedCalendarValue> = Vec::new();

    let mut head = git::get_head_commit(repo);

    let session_time = args.time_allowed;

    let mut prediction = PredictionStructure::new();

    // Adds data the commit_arr
    // let max_commit_depth = 25;
    // for _i in 0..max_commit_depth {
    loop {

        let mut diff_opts = DiffOptions::new();

        let Ok(parent) = head.parent(0) else {
            break;
        };

        let diff = repo.diff_tree_to_tree(
            Some(&parent.tree().unwrap()),
            Some(&head.tree().unwrap()),
            Some(
                diff_opts
                    .force_text(true)
                )
            )
            .unwrap()
            .stats()
            .unwrap();

        let timestamp = head.time().seconds();
        let prev_timestamp = parent.time().seconds();
        let delta_t = timestamp - prev_timestamp;

        let commit_data: AnnotatedCalendarValue = (
            CalendarValue {
                title: head.message().unwrap_or("MESSAGE_NOT_FOUND").trim().to_string(),
                delta_t,
                start: prev_timestamp,
                end: timestamp,
                projected: false,
            },
            vec![
                (PredictionAttributes::FilesChanged, diff.files_changed() as i32),
                (PredictionAttributes::LinesAdded, diff.insertions() as i32),
                (PredictionAttributes::LinesRemoved, diff.deletions() as i32),
            ]
        );

        commit_arr.push(commit_data);

        if delta_t < session_time {
            prediction.insert_item(PredictionAttributes::FilesChanged, diff.files_changed() as i32, delta_t);
            prediction.insert_item(PredictionAttributes::LinesAdded, diff.insertions() as i32, delta_t);
            prediction.insert_item(PredictionAttributes::LinesRemoved, diff.deletions() as i32, delta_t);
        }

        head = parent;
    }

    let output_arr = commit_arr
        .split_inclusive(|v| session_time <= v.0.delta_t)
        .collect::<Vec<&[AnnotatedCalendarValue]>>()
        .iter_mut()
        .map(|v| {
            let mut items = v.to_vec();
            let item = items.last_mut().unwrap();

            // Makes prediction for last item
            let prediction = prediction.predict(&item.1);

            // Updates item with projections
            item.0.delta_t = prediction;
            item.0.start = item.0.end - prediction;
            item.0.projected = true;

            items
        })
        .collect::<Vec<Vec<AnnotatedCalendarValue>>>()
        ;

    // let mut calendar_items = CalendarValueArr::new();
    let mut calendar_items = Vec::new();

    // Converts the list of list of `CommitData`s into a single array of `CalendarValues`s
    for item_lst in output_arr {
        for value in item_lst {
            calendar_items.push(value.0);
        }
    }

    return calendar_items;
}

