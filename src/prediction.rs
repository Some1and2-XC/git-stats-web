use std::collections::hash_map::HashMap;
use super::Timestamp;


#[derive(Debug, Default, Clone)]
pub struct PredictionAttributes {
    sum: (i32, Timestamp),
    count: i32,
    min: (i32, Timestamp),
    max: (i32, Timestamp),
}

impl PredictionAttributes {
    /// Makes Prediction based on its own values
    pub fn predict(&self, value: i32) -> Timestamp {

        let projection = value as Timestamp * (self.sum.1 / self.sum.0 as Timestamp);

        if projection < self.min.1 { return self.min.1; }
        if projection > self.max.1 { return self.max.1; }

        return projection;
    }
}

#[derive(Debug)]
pub struct PredictionStructure {
    history_map: HashMap<String, PredictionAttributes>,
    keys: Vec<String>,
}

impl PredictionStructure {
    pub fn new() -> Self {
        Self {
            history_map: HashMap::new(),
            keys: vec![
                "files_changed".to_string(),
                "lines_added".to_string(),
                "lines_removed".to_string()
            ],
        }
    }

    /// Function for adding another item to the PredictionStructure
    /// `key` refers to the key of the dictionary as well as what to insert
    /// an example would be "lines_added" or "files_changed"
    /// the value is the value of the attribute
    /// Returns `true` if a new value was added
    /// Returns `false` if a value was modified
    pub fn insert_item(&mut self, key: String, value: i32, time: Timestamp) -> bool {

        assert!(self.keys.contains(&key));

        let mut attributes = match self.history_map.get_mut(&key) {
            Some(v) => v.clone(),
            None => PredictionAttributes::default(),
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
            Some(_) => true,
            None => false,
        };
    }

    /// Makes a prediction based on all the previous values it found
    pub fn predict(&self, files_changed: i32, lines_added: i32, lines_removed: i32) -> Timestamp {

        let mut results = vec![];

        let keys_and_values = [
            ("files_changed".to_string(), files_changed),
            ("lines_added".to_string(), lines_added),
            ("lines_removed".to_string(), lines_removed),
        ];

        for (k, v) in keys_and_values {
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
