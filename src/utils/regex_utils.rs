use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX_CACHE: Lazy<Mutex<HashMap<String, Regex>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub trait CachedRegex {
    fn get(pattern: String) -> anyhow::Result<Regex>;
}

impl CachedRegex for Regex {
    fn get(pattern: String) -> anyhow::Result<Regex> {
        if let Ok(mut cache) = REGEX_CACHE.lock() {
            if let Some(compiled) = cache.get(&pattern) {
                return Ok(compiled.clone());
            }
            let compiled = Regex::new(&pattern.clone())
                .map_err(|_| anyhow::anyhow!(format!("Failed to compile regex {pattern}")))?;
            cache.insert(pattern.clone(), compiled.clone());

            return Ok(compiled)
        }

        let compiled = Regex::new(&pattern.clone())
            .map_err(|_| anyhow::anyhow!(format!("Failed to compile regex {pattern}")))?;
        Ok(compiled)
    }
}
