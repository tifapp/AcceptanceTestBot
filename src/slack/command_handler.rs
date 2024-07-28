use serde::Deserialize;

use super::command::RoswaalSlackCommand;

/// A request from slack.
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct RoswaalSlackRequest {
    token: String,
    channel_id: String,
    text: String,
    command: RoswaalSlackCommand,
    response_url: String
}
