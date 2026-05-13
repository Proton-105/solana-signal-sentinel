use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct DedupCache {
    seen:    HashMap<String, Instant>,
    ttl:     Duration,
}

impl DedupCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            seen: HashMap::new(),
            ttl:  Duration::from_secs(ttl_secs),
        }
    }

    /// Возвращает true если mint уже видели недавно (дубликат)
    pub fn is_duplicate(&mut self, mint: &str) -> bool {
        let now = Instant::now();

        // Чистим устаревшие записи
        self.seen.retain(|_, ts| now.duration_since(*ts) < self.ttl);

        if self.seen.contains_key(mint) {
            return true;
        }

        self.seen.insert(mint.to_string(), now);
        false
    }
}
