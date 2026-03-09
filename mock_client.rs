use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let api_key = "test";
    let api_secret = "test";
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    let payload = format!("{}{}{}", api_key, api_secret, timestamp);
    println!("Payload: {}", payload);
}
