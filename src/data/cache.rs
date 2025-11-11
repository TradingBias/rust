use polars::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct IndicatorCache {
    data: Mutex<HashMap<String, Series>>,
    capacity: usize,
}

impl IndicatorCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Mutex::new(HashMap::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn get(&self, key: &str) -> Option<Series> {
        let data = self.data.lock().unwrap();
        data.get(key).cloned()
    }

    pub fn set(&self, key: String, value: Series) {
        let mut data = self.data.lock().unwrap();
        if data.len() >= self.capacity {
            // A simple eviction strategy: clear the cache when full.
            data.clear();
        }
        data.insert(key, value);
    }
}
