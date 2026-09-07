use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

static STORAGE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn get_storage() -> &'static Mutex<HashMap<String, String>> {
    STORAGE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn add(key: &str, value: &str) -> Result<Option<String>, String> {
    let mut storage = match get_storage().lock() {
        Ok(s) => s,
        Err(_) => return Err("Internal storage error".to_string()),
    };

    Ok(storage.insert(key.to_string(), value.to_string()))
}

pub fn get(key: &str) -> Result<Option<String>, String> {
    let storage = match get_storage().lock() {
        Ok(s) => s,
        Err(_) => return Err("Internal storage error".to_string()),
    };

    Ok(storage.get(key).cloned())
}
