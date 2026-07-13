use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use crate::event::Event::App;
use serde::{Deserialize, Serialize};

const COMMAND_SOCK: &str = "/tmp/seep-bridge.sock";
const PUSH_SOCK: &str = "/tmp/seep-push.sock";

#[derive(Deserialize, Debug, Clone)]
pub struct Contact {
    pub jid: String,
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
struct IncomingMessage {
    from: String,
    text: String,
    timestamp: String,
}

#[derive(Serialize)]
struct Command {
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

pub async fn get_contacts() -> color_eyre::Result<Vec<Contact>> {
    let mut stream = UnixStream::connect(COMMAND_SOCK).await?;

    let cmd = serde_json::to_string(&Command {
        action: "get_contacts".to_string(),
        to: None,
        text: None,
    })?;
    stream.write_all(cmd.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    Ok(serde_json::from_str(&line)?)
}

pub async fn send_message(jid: &str, text: &str) -> color_eyre::Result<()> {
    let mut stream = UnixStream::connect(COMMAND_SOCK).await?;

    let cmd = serde_json::to_string(&Command {
        action: "send".to_string(),
        to: Some(jid.to_string()),
        text: Some(text.to_string()),
    })?;
    stream.write_all(cmd.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    Ok(())
}

pub async fn subscribe_messages(sender: tokio::sync::mpsc::UnboundedSender<crate::event::Event>) {
    tokio::spawn(async move {
        let stream = match UnixStream::connect(PUSH_SOCK).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Push socket error: {e}");
                return;
            }
        };

        let mut reader = BufReader::new(stream); // no &mut — stream is moved directly
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) | Err(_) => break, // 0 bytes = connection closed cleanly
                Ok(_) => {
                    if let Ok(msg) = serde_json::from_str::<IncomingMessage>(&line) {
                        let _ = sender.send(App(
                            crate::event::AppEvent::MessageReceived {
                                from: msg.from,
                                text: msg.text,
                                timestamp: msg.timestamp,
                            },
                        ));
                    }
                }
            }
        }
    });
}
