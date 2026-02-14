use std::sync::{Mutex};
use once_cell::sync::Lazy;
use reqwest::{header, Client};

static CLIENT: Lazy<Mutex<Client>> = Lazy::new(|| {
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", "Mozilla/5.0".parse().unwrap());

    let client = Client::builder().default_headers(headers);

    Mutex::new(client.build().unwrap())
});

pub fn client() -> Client {
    CLIENT.lock().unwrap().clone()
}
