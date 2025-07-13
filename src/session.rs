use reqwest::Client;
use serde::{ Deserialize, Serialize };

const LOGIN_ENDPOINT: &'static str = "https://scratch.mit.edu/accounts/login/";

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("reqwest error: {0:#?}")] Reqwest(reqwest::Error),
    #[error("Deserialize error: {0:#?}")] Deserializing(reqwest::Error),
}

#[derive(Debug)]
pub struct Session {
    client: Client,
    id: String,
}

impl Session {
    #[deprecated]
    pub async fn login<K: AsRef<str>>(_username: K, _password: K) -> Result<Self, LoginError> {
        panic!("Cloudflare blocked");
    }

    pub fn from_id(session_id: String) -> Self {
        Self {
            client: Client::new(),
            id: session_id,
        }
    }
}
