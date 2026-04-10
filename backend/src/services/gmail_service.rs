
use crate::prelude::*;
use std::fs;

fn load_google_secrets() -> serde_json::Value {
    let data = fs::read_to_string("client_secret.json")
        .expect("Failed to read client_secret.json");

    serde_json::from_str(&data).unwrap()
}

pub fn build_gmail_oauth_url() -> String {
    let secrets = load_google_secrets();

    let client_id = secrets["web"]["client_id"]
        .as_str()
        .unwrap();

    let redirect_uri = secrets["web"]["redirect_uris"][0]
        .as_str()
        .unwrap();

    let scope = "https://www.googleapis.com/auth/userinfo.email \
                 https://www.googleapis.com/auth/gmail.send \
                 https://www.googleapis.com/auth/gmail.modify \
                 https://www.googleapis.com/auth/gmail.readonly";

    format!(
        "https://accounts.google.com/o/oauth2/v2/auth\
        ?client_id={}\
        &redirect_uri={}\
        &response_type=code\
        &scope={}\
        &access_type=offline\
        &prompt=consent",
        client_id,
        redirect_uri,
        scope
    )
}