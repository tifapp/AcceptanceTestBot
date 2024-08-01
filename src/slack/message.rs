use std::{env, future::Future};

use reqwest::Client;
use serde::Serialize;
use anyhow::{Result, Error};

use crate::utils::env::RoswaalEnvironement;

use super::ui_lib::{block_kit_views::SlackSection, blocks::SlackBlocks, if_view::If, slack_view::{render_slack_view, SlackView}};

/// A slack message.
///
/// A slack message is created from a `SlackView` and a string channel identifier.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
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
            blocks: render_slack_view(&MessageView { base: view }),
            response_url: response_url.to_string()
        }
    }
}

struct MessageView<'v, Base: SlackView> {
    base: &'v Base
}

impl <'v, Base: SlackView> SlackView for MessageView<'v, Base> {
    fn slack_body(&self) -> impl SlackView {
        If::is_true(
            RoswaalEnvironement::current() == RoswaalEnvironement::Dev,
            || SlackSection::from_markdown("_This message was sent for development purposeeeeeeeeeees. Please ignoooooooore._")
        )
        .flat_chain_block_ref(self.base)
    }
}

/// A trait for sending a slack message.
pub trait SlackSendMessage {
    fn send(&self, message: &SlackMessage) -> impl Future<Output = Result<()>> + Send;
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
