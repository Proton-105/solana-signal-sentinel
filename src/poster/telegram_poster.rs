use anyhow::Result;
use reqwest::Client;
use serde::Serialize;
use tracing::{error, info};

#[derive(Serialize)]
struct SendMessage {
    chat_id:    String,
    text:       String,
    parse_mode: String,
}

pub struct TelegramPoster {
    client:     Client,
    bot_token:  String,
    channel_id: String,
}

impl TelegramPoster {
    pub fn new(bot_token: String, channel_id: String) -> Self {
        Self {
            client: Client::new(),
            bot_token,
            channel_id,
        }
    }

    pub async fn send(&self, text: String) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        let body = SendMessage {
            chat_id:    self.channel_id.clone(),
            text,
            parse_mode: "HTML".to_string(),
        };

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Telegram message sent successfully");
        } else {
            let status = response.status();
            let text   = response.text().await.unwrap_or_default();
            error!("Telegram API error: {} — {}", status, text);
        }

        Ok(())
    }
}
