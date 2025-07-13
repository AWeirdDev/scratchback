use std::sync::Arc;

use serde::{ Deserialize, Serialize };

use futures_util::{ stream::{ SplitSink, SplitStream }, SinkExt, StreamExt };
use tokio::{ net::TcpStream, sync::Mutex };
use tokio_tungstenite::{ tungstenite::Message, MaybeTlsStream, WebSocketStream };

const ENDPOINT: &'static str = "wss://clouddata.scratch.mit.edu/";
const CLOUD: &'static str = "☁ ";

type Tx = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type Rx = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("WebSocket error: {0:?}")] WebSocket(tokio_tungstenite::tungstenite::Error),
    #[error("Failed to serialize: {0:#?}")] Serializing(serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum NextError {
    #[error("WebSocket error: {0:?}")] WebSocket(tokio_tungstenite::tungstenite::Error),
    #[error("Failed to convert to text")] ToText,
    #[error("Failed to deserialize: {0:#?}")] Deserializing(serde_json::Error),
}

#[derive(Clone)]
pub struct Cloud {
    tx: Arc<Mutex<Tx>>,
    rx: Arc<Mutex<Rx>>,
    user: String,
}

impl Cloud {
    pub async fn connect(username: String) -> Result<Self, Box<dyn core::error::Error>> {
        let (stream, _) = tokio_tungstenite::connect_async(ENDPOINT).await?;
        let (tx, rx) = stream.split();
        Ok(Self {
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            user: username,
        })
    }

    /// Send a model to the server.
    pub async fn send<S: Serialize>(&self, model: &S) -> Result<(), SendError> {
        let mut sink = self.tx.lock().await;
        let binding = serde_json::to_string(model);
        let Ok(serialized) = binding else {
            return Err(SendError::Serializing(binding.unwrap_err()));
        };

        let res = sink.send(Message::text(serialized)).await;
        if let Err(err) = res {
            return Err(SendError::WebSocket(err));
        }
        Ok(())
    }

    /// Next item.
    /// This is from the cloud server, therefore the arm `CloudMethod::Handshake` can be marked `unreachable!()`.
    pub async fn next(&self) -> Option<Result<CloudMethod, NextError>> {
        let mut stream = self.rx.lock().await;
        let Some(res) = stream.next().await else {
            return None;
        };
        let Ok(data) = res else {
            return Some(Err(NextError::WebSocket(res.unwrap_err())));
        };

        let Ok(s) = data.to_text() else {
            return Some(Err(NextError::ToText));
        };

        let binding = serde_json::from_str::<CloudMethod>(s);
        let Ok(method) = binding else {
            return Some(Err(NextError::Deserializing(binding.unwrap_err())));
        };

        Some(Ok(method))
    }

    pub fn project(&self, id: String) -> CloudProject {
        CloudProject { id, cloud: self.clone() }
    }
}

pub struct CloudProject {
    id: String,
    cloud: Cloud,
}

impl CloudProject {
    /// Connect to a project on the cloud.
    pub async fn connect(
        username: String,
        project_id: String
    ) -> Result<Self, Box<dyn core::error::Error>> {
        let cloud = Cloud::connect(username).await?;
        Ok(cloud.project(project_id))
    }

    /// Handshake with the server.
    pub async fn handshake(&self) -> Result<(), SendError> {
        self.cloud.send(
            &ijson::ijson!({
            "method": "handshake",
            "user": self.cloud.user.as_str(),
            "proejct_id": self.id.as_str(),
        })
        ).await
    }

    /// Set a cloud variable.
    pub async fn set(&self, var: &str, value: &str) -> Result<(), SendError> {
        self.cloud.send(
            &ijson::ijson!({
                "method": "set",
                "name": format!("{}{}", CLOUD, var.trim_start_matches(CLOUD)),
                "project_id": &self.id,
                "value": value
            })
        ).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "method")]
pub enum CloudMethod {
    Handshake {
        user: String,
        project_id: String,
    },
    Set {
        name: String,
        user: String,
        project_id: String,
        value: String,
    },
}
