use crate::prelude::*;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

pub fn extract_body(payload: &Value) -> Option<String> {
    // ✅ 1. direct body
    if let Some(data) = payload["body"]["data"].as_str() {
        return Some(decode_base64(data));
    }

    // ✅ 2. parts (most important)
    if let Some(parts) = payload["parts"].as_array() {
        for part in parts {
            let mime = part["mimeType"].as_str().unwrap_or("");

            // 🔥 prefer HTML
            if mime == "text/html"
                && let Some(data) = part["body"]["data"].as_str()
            {
                return Some(decode_base64(data));
            }

            // fallback text
            if mime == "text/plain"
                && let Some(data) = part["body"]["data"].as_str()
            {
                return Some(decode_base64(data));
            }

            // 🔁 recursive (VERY IMPORTANT)
            if let Some(nested) = extract_body(part) {
                return Some(nested);
            }
        }
    }

    None
}

pub fn decode_base64(data: &str) -> String {
    let fixed = data.replace("-", "+").replace("_", "/");

    let decoded = STANDARD.decode(fixed).unwrap_or_default();

    String::from_utf8_lossy(&decoded).to_string()
}
