use std::env;

use reqwest::Client;
use serde::Serialize;
use anyhow::{Result, Error};

use super::ui_lib::{blocks::SlackBlocks, slack_view::{render_slack_view, SlackView}};

/// A slack message.
///
/// A slack message is created from a `SlackView` and a string channel identifier.
#[derive(Debug, Serialize)]
pub struct SlackMessage {
    #[serde(rename = "channel")]
    channel_id: String,
    blocks: SlackBlocks,
    #[serde(skip)]
    response_url: String
}

impl SlackMessage {
    pub fn new(channel_id: &str, view: &impl SlackView, response_url: &str) -> Self {
        Self {
            channel_id: channel_id.to_string(),
            blocks: render_slack_view(view),
            response_url: response_url.to_string()
        }
    }
}

/// A trait for sending a slack message.
pub trait SlackSendMessage {
    async fn send(&self, message: &SlackMessage) -> Result<()>;
}

impl SlackSendMessage for Client {
    async fn send(&self, message: &SlackMessage) -> Result<()> {
        let token = env::var("SLACK_BOT_TOKEN")
            .expect("SLACK_BOT_TOKEN not found in .env, you can get one from the slack app console.");
        let resp = self.post(message.response_url.to_string())
            .json(message)
            .bearer_auth(token)
            .send()
            .await?;
        match resp.error_for_status() {
            Ok(_) => Ok(()),
            Err(error) => {
                log::error!("A Slack API error occured {}.", error);
                Err(Error::new(error))
            }
        }
    }
}
