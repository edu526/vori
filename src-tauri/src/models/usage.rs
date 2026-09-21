use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// How often, and how recently, something was opened from Vori.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    pub count: u32,
    /// Unix seconds of the last time it was opened.
    pub last: f64,
}

/// Keyed by the opened path (the same key `recents` uses).
pub type UsageMap = HashMap<String, Usage>;

/// Record one more open of `path` at time `at` (unix seconds).
pub fn record(usage: &mut UsageMap, path: &str, at: f64) {
    let entry = usage.entry(path.to_string()).or_default();
    entry.count = entry.count.saturating_add(1);
    entry.last = entry.last.max(at);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_counts_opens_and_keeps_the_latest_time() {
        let mut map = UsageMap::new();
        record(&mut map, "/p", 100.0);
        record(&mut map, "/p", 300.0);
        record(&mut map, "/p", 200.0); // out of order: the latest time is kept
        assert_eq!(map["/p"], Usage { count: 3, last: 300.0 });
    }
}
