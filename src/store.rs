use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub trait RedisKV {
    fn get(&self, key: String) -> Option<String>;
    fn set(&self, key: String, value: String) -> Option<String>;
}

#[derive(Debug, Clone)]
pub struct HashStore {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl RedisKV for HashStore {
    fn get(&self, key: String) -> Option<String> {
        self.data.lock().unwrap().get(&key).cloned()
    }
    fn set(&self, key: String, value: String) -> Option<String> {
        self.data.lock().unwrap().insert(key, value)
    }
}

impl HashStore {
    pub fn new() -> Self {
        HashStore {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
