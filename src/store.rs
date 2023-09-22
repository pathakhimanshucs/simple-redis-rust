use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

pub trait RedisKV {
    fn get(&self, key: String) -> Option<String>;
    fn set(&self, key: String, value: RedisValue) -> Option<String>;
}

#[derive(Debug, Clone)]
pub enum RedisValue {
    SimpleValue(String),
    ValueWithExpiry { value: String, expiry_unix_ms: u128 },
}

#[derive(Debug, Clone)]
pub struct HashStore {
    data: Arc<Mutex<HashMap<String, RedisValue>>>,
}

impl RedisKV for HashStore {
    fn get(&self, key: String) -> Option<String> {
        if let Some(v) = self.data.lock().unwrap().get(&key).cloned() {
            match v {
                RedisValue::SimpleValue(str) => Some(str),
                RedisValue::ValueWithExpiry {
                    value,
                    expiry_unix_ms,
                } => {
                    if expiry_unix_ms < get_epoch_ms() {
                        return None;
                    }
                    Some(value)
                }
            }
        } else {
            None
        }
    }
    fn set(&self, key: String, value: RedisValue) -> Option<String> {
        if let Some(v) = self.data.lock().unwrap().insert(key, value) {
            match v {
                RedisValue::SimpleValue(str) => Some(str),
                RedisValue::ValueWithExpiry {
                    value,
                    expiry_unix_ms: _,
                } => Some(value),
            }
        } else {
            None
        }
    }
}

impl HashStore {
    pub fn new() -> Self {
        HashStore {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
