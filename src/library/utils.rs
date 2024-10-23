use std::{collections::BTreeSet, sync::Arc};

use regex::Regex;
use url::Url;
use log::debug;

use git2::{Commit, DiffOptions, Oid, Repository};

use crate::{
    aliases::AnnotatedCalendarValue,
    cli::CliArgs,
    prediction::{PredictionAttributes, PredictionStructure},
    calendar::CalendarValue,
    git,
};

/// Result type for methods which update an external source and can fail.
/// This should generally be used when your function signature looks something like:
/// `fn foo(self) -> UpdateResult<T, T>`
/// with the idea that you want the borrow checker to invalidate your input.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum UpdateResult<T, E> {
    /// The ok variant, meaning your method did what you would like it to.
    Ok(T),
    /// The error variant, meaning your method failed to complete successfully and the result is a
    /// fallback value.
    Err(E),
}

impl<T, E> UpdateResult<T, E> {

    /// Method for checking if the value is `Ok`
    pub const fn is_ok(&self) -> bool {
        return matches!(*self, Self::Ok(_));
    }

    /// Method for checking if the value is `Err`
    pub const fn is_err(&self) -> bool {
        return matches!(*self, Self::Err(_));
    }

    /// Method for casting the `Ok` value to the `Option::Some()`
    pub fn ok(self) -> Option<T> {
        return match self {
            Self::Ok(v) => Some(v),
            Self::Err(_) => None,
        };
    }

    /// Method for casting the `Err` value to the `Option::Some()`
    pub fn err(self) -> Option<E> {
        return match self {
            Self::Ok(_) => None,
            Self::Err(v) => Some(v),
        };
    }

    /// Method for casting an option into `Self`
    pub fn from_option(ok: Option<T>, err: E) -> Self {
        return match ok {
            Some(v) => Self::Ok(v),
            None => Self::Err(err)
        };
    }

}

impl<T> UpdateResult<T, T> {

    /// Method for returning the inner value regardless if the value is `Ok` or `Err`
    pub fn indifferent(self) -> T {
        return match self {
            Self::Ok(v) => v,
            Self::Err(v) => v,
        };
    }

}

impl<T, E> From<Result<T, E>> for UpdateResult<T, E> {
    fn from(res: Result<T, E>) -> Self {
        return match res {
            Ok(v) => UpdateResult::Ok(v),
            Err(e) => UpdateResult::Err(e),
        };
    }
}

impl<T, E> Into<Result<T, E>> for UpdateResult<T, E> {
    fn into(self) -> Result<T, E> {
        return match self {
            Self::Ok(v) => Ok(v),
            Self::Err(e) => Err(e),
        };
    }
}

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

/// Function for getting all the commit data from a repository.
fn recurs_search_trees(args: Arc<CliArgs>, repo: &Repository, commit: Commit, out_vec: &mut Vec<AnnotatedCalendarValue>, searched_commits: &mut BTreeSet<Oid>, out_pred_struct: &mut PredictionStructure) -> () {

    // Tries to insert commit OID but if this isn't possible
    // the just return. (see `BTreeSet::insert()` docs)
    if !searched_commits.insert(commit.id()) {
        return;
    }

    let mut diff_opts = DiffOptions::new();

    let Ok(parent) = commit.parent(0) else {
        return;
    };

    let diff = repo.diff_tree_to_tree(
        Some(&parent.tree().unwrap()),
        Some(&commit.tree().unwrap()),
        Some(
            diff_opts
                .force_text(true)
            )
        )
        .unwrap()
        .stats()
        .unwrap();

    let timestamp = commit.time().seconds();
    let prev_timestamp = parent.time().seconds();
    let delta_t = timestamp - prev_timestamp;

    let commit_data: AnnotatedCalendarValue = (
        CalendarValue {
            title: commit.message().unwrap_or("MESSAGE_NOT_FOUND").trim().to_string(),
            author: commit.author().to_string(),
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

    out_vec.push(commit_data);

    if delta_t < args.time_allowed {
        out_pred_struct.insert_item(PredictionAttributes::FilesChanged, diff.files_changed() as i32, delta_t);
        out_pred_struct.insert_item(PredictionAttributes::LinesAdded, diff.insertions() as i32, delta_t);
        out_pred_struct.insert_item(PredictionAttributes::LinesRemoved, diff.deletions() as i32, delta_t);
    }

    for parent in commit.parents() {
        recurs_search_trees(args.clone(), repo, parent, out_vec, searched_commits, out_pred_struct);
    }

}

/// Function for getting commit data and returning json
pub fn calculate_data(args: Arc<CliArgs>, repo: &Repository) -> Vec<CalendarValue> {

    let mut commit_arr: Vec<AnnotatedCalendarValue> = Vec::new();

    let head = git::get_head_commit(repo);
    let mut prediction = PredictionStructure::new();

    let mut searched_commits: BTreeSet<Oid> = BTreeSet::new();

    // Gets all the data
    recurs_search_trees(args.clone(), repo, head, &mut commit_arr, &mut searched_commits, &mut prediction);

    // Adds data the commit_arr
    // let max_commit_depth = 25;
    // for _i in 0..max_commit_depth {

    let output_arr = commit_arr
        .split_inclusive(|v| (&args).time_allowed <= v.0.delta_t)
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

