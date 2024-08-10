use regex::Regex;
use url::Url;
use log::debug;

use git2::{DiffOptions, Repository};

use crate::{
    cli::CliArgs,
    prediction::{PredictionAttributes, PredictionStructure},
    calendar::CalendarValue,
    git,
};

/// Gets the path of a url from URI
/// May modify the url to work with git@xyz:your/project domains
/// Rewrites them to https://xyz/your/project
pub fn get_path(src_url: &str) -> Url {
    let mut tmp_url = src_url.to_string();

    let re = Regex::new(r"git@(?<domain>.+):(?<path>.+)").unwrap();

    if let Some(caps) = re.captures(&tmp_url) {
        tmp_url = format!("https://{}/{}", &caps["domain"], &caps["path"]);
    }

    debug!("Parsing URL: {tmp_url}");
    Url::parse(&tmp_url).unwrap()
}

/// Function for getting commit data and returning json
pub fn calculate_data(args: &CliArgs, repo: &Repository) -> Vec<CalendarValue> {

    let mut commit_arr: Vec<git::CommitData> = Vec::new();

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

        let commit_data = git::CommitData {
            message: head.message().unwrap_or("MESSAGE_NOT_FOUND").trim().to_string(),
            timestamp,
            prev_timestamp,
            delta_t,
            files_changed: diff.files_changed() as i32,
            lines_added: diff.insertions() as i32,
            lines_removed: diff.deletions() as i32,
            projected: false,
        };

        commit_arr.push(commit_data);

        if delta_t < session_time {
            prediction.insert_item(PredictionAttributes::FilesChanged, diff.files_changed() as i32, delta_t);
            prediction.insert_item(PredictionAttributes::LinesAdded, diff.insertions() as i32, delta_t);
            prediction.insert_item(PredictionAttributes::LinesRemoved, diff.deletions() as i32, delta_t);
        }

        head = parent;
    }

    let output_arr = commit_arr
        .split_inclusive(|v| session_time <= v.delta_t)
        .collect::<Vec<&[git::CommitData]>>()
        .iter_mut()
        .map(|v| {
            let mut items = v.to_vec();
            let item = items.last_mut().unwrap();

            // Makes prediction for last item
            let prediction = prediction.predict(vec![
                (PredictionAttributes::FilesChanged, item.files_changed),
                (PredictionAttributes::LinesAdded, item.lines_added),
                (PredictionAttributes::LinesRemoved, item.lines_removed),
                ]);

            // Updates item with projections
            item.delta_t = prediction;
            item.prev_timestamp = item.timestamp - prediction;
            item.projected = true;

            items
        })
        .collect::<Vec<Vec<git::CommitData>>>()
        ;

    // let mut calendar_items = CalendarValueArr::new();
    let mut calendar_items = Vec::new();

    // Converts the list of list of `CommitData`s into a single array of `CalendarValues`s
    for item_lst in output_arr {
        for value in item_lst {
            calendar_items.push(
                CalendarValue {
                    title: value.message.clone(),
                    delta_t: value.delta_t,
                    start: value.prev_timestamp + 1, // Adding one so the timestamp don't overlap
                    end: value.timestamp,
                    projected: value.projected,
                }
            );
        }
    }

    return calendar_items;
}

