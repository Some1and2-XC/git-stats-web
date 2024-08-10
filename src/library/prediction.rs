use std::collections::hash_map::HashMap;
use super::aliases::Timestamp;

/// A struct representing the attributes used for making a prediction.
#[derive(Debug, Default, Clone)]
pub struct PredictionValues {
    sum: (i32, Timestamp),
    count: i32,
    min: (i32, Timestamp),
    max: (i32, Timestamp),
}

impl PredictionValues {
    /// Makes Prediction based on its own values
    pub fn predict(&self, value: i32) -> Timestamp {

        let projection = value as Timestamp * (self.sum.1 / self.sum.0 as Timestamp);

        if projection < self.min.1 { return self.min.1; }
        if projection > self.max.1 { return self.max.1; }

        return projection;
    }
}

/// Represents some of the values returned from git's diff stats.
#[derive(Eq, Hash, PartialEq, Debug)]
pub enum PredictionAttributes {
    /// Represents the amount of files changed in a commit.
    FilesChanged,
    /// Represents the amount of lines added in a commit.
    LinesAdded,
    /// Represents the amount of lines removed in a commit.
    LinesRemoved,
}

/// The struct holding data to be used in predictions.
#[derive(Debug)]
pub struct PredictionStructure {
    history_map: HashMap<PredictionAttributes, PredictionValues>,
}

impl PredictionStructure {
    /// A method for creating a new `PredictionStructure`.
    /// ```rust
    /// # use git_stats_web::prediction::{PredictionStructure, PredictionAttributes};
    /// # use std::collections::hash_map::HashMap;
    /// let mut ps = PredictionStructure::new();
    /// ps.insert_item(PredictionAttributes::LinesAdded, 5, 1000); // Adds an item
    /// let map = vec![(PredictionAttributes::LinesAdded, 1)]; // a list of attributes to predict by
    /// assert_eq!(ps.predict(map), 200); // the response is 200, (using the ratio from 5:1 => 1000:200)
    /// ```
    pub fn new() -> Self {
        Self {
            history_map: HashMap::new(),
        }
    }

    /// Function for adding another item to the PredictionStructure
    /// `key` refers to the key of the dictionary as well as what to insert
    /// an example would be "lines_added" or "files_changed"
    /// the value is the value of the attribute
    /// Returns `true` if a new value was added
    /// Returns `false` if a value was modified
    /// ```rust
    /// # use git_stats_web::prediction::{PredictionStructure, PredictionAttributes};
    /// let mut ps = PredictionStructure::new();
    /// assert_eq!(ps.insert_item(PredictionAttributes::LinesAdded, 5, 1000), true); // Initially returns true
    /// assert_eq!(ps.insert_item(PredictionAttributes::LinesAdded, 5, 1000), false); // Then returns false (no item is being added)
    /// ```
    pub fn insert_item(&mut self, key: PredictionAttributes, value: i32, time: Timestamp) -> bool {

        let mut attributes = match self.history_map.get_mut(&key) {
            Some(v) => v.clone(),
            None => PredictionValues::default(),
        };

        // Updates sum and count
        attributes.count += 1;
        attributes.sum.0 += value;
        attributes.sum.1 += time;

        // Updates Min
        if value < attributes.min.0 {
            attributes.min.0 = value;
            attributes.min.1 = time;
        }

        // Updates Max
        if value > attributes.max.0 {
            attributes.max.0 = value;
            attributes.max.1 = time;
        }

        return match self.history_map.insert(key, attributes) {
            Some(_) => false,
            None => true,
        };
    }

    /// Makes a prediction based on all the previous values it found
    /// ```rust
    /// # use git_stats_web::prediction::{PredictionStructure, PredictionAttributes};
    /// let mut ps = PredictionStructure::new();
    /// ps.insert_item(PredictionAttributes::LinesAdded, 5, 1000);
    /// assert_eq!(ps.predict(vec![(PredictionAttributes::LinesAdded, 5)]), 1000);
    /// assert_eq!(ps.predict(vec![(PredictionAttributes::LinesAdded, 1)]), 200);
    /// ```
    pub fn predict(&self, values: Vec<(PredictionAttributes, i32)>) -> Timestamp {

        let mut results = vec![];

        for (k, v) in values {
            let pred_value = match self.history_map.get(&k) {
                Some(v) => v,
                None => continue,
            };

            results.push(pred_value.predict(v));
        }

        let response = results.iter().sum::<Timestamp>() / results.len() as Timestamp;
        return response;

    }
}
