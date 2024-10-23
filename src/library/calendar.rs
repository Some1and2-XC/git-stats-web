use super::aliases::Timestamp;
use serde::{Deserialize, Serialize};

/// A struct used for returning data from the calendar endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CalendarValue {
    /// The title of the commit
    pub title: String,
    /// The amount of time from the start of the commit to the end.
    pub delta_t: Timestamp,
    /// The epoch timestamp for when the commit started.
    pub start: Timestamp,
    /// The epoch timestamp for when the commit finished. This is usually directly from the git
    /// database.
    pub end: Timestamp,
    /// A flag, true if the value was projected, false if not.
    pub projected: bool,
    /// The author of the commit
    pub author: String,
}

