use std::{
    env,
    error::Error,
    fmt::{Display, Formatter},
    future::Future,
};

use anyhow::Result;
use reqwest::{header::CONTENT_TYPE, Client};
use serde::{Deserialize, Serialize};

use super::{
    message_view::MessageView,
    ui_lib::{
        blocks::SlackBlocks,
        slack_view::{render_slack_view, SlackView},
    },
};

/// A slack message.
///
/// A slack message is created from a `SlackView` and a string channel identifier.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct SlackMessage {
    #[serde(rename = "channel")]
    channel_id: String,
    blocks: SlackBlocks,
    #[serde(skip)]
    response_url: String,
}

impl SlackMessage {
    pub fn new(channel_id: &str, view: &impl SlackView, response_url: &str) -> Self {
        Self {
            channel_id: channel_id.to_string(),
            blocks: render_slack_view(&MessageView::new(view)),
            response_url: response_url.to_string(),
        }
    }
}

/// A trait for sending a slack message.
pub trait SlackSendMessage {
    fn send(&self, message: &SlackMessage) -> impl Future<Output = Result<()>> + Send;
}

#[derive(Debug, Deserialize)]
struct SlackResponse {
    error: Option<String>,
}

#[derive(Debug)]
struct SlackMessageSendingError {
    message: String,
}

impl Display for SlackMessageSendingError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Slack Message Sending Error: {}", self.message)
    }
}

impl Error for SlackMessageSendingError {}

impl SlackSendMessage for Client {
    async fn send(&self, message: &SlackMessage) -> Result<()> {
        let token = env::var("SLACK_BOT_TOKEN").expect(
            "SLACK_BOT_TOKEN not found in .env, you can get one from the slack app console.",
        );
        let resp = self
            .post(message.response_url.to_string())
            .header(CONTENT_TYPE, "application/json")
            .json(message)
            .bearer_auth(token)
            .send()
            .await?;
        let slack_resp = resp.json::<SlackResponse>().await?;
        match slack_resp.error {
            Some(error) => {
                log::error!("A Slack API error occured {}.", error);
                Err(anyhow::Error::new(SlackMessageSendingError {
                    message: error,
                }))
            }
            None => Ok(()),
        }
    }
}
