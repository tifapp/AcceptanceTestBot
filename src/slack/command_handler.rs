use std::{future::Future, sync::Arc};

use serde::Deserialize;
use anyhow::Error;
use tokio::spawn;
use super::{command::RoswaalSlackCommand, message::{SlackMessage, SlackSendMessage}, pending_view::PendingView, ui_lib::{blocks::SlackBlocks, empty_view::EmptySlackView, slack_view::{render_slack_view, SlackView}}};

/// A request from slack.
#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
pub struct RoswaalSlackRequest {
    channel_id: String,
    text: String,
    command: RoswaalSlackCommand,
    response_url: String
}

/// A trait for handling slack commands.
pub trait RoswaalSlackHandler: Sized + 'static {
    /// Handles the specified command and command text, and returns a `SlackView` with the contents
    /// of the response to the command.
    fn handle_command(
        self,
        command: &RoswaalSlackCommand,
        command_text: &str
    ) -> impl Future<Output = Result<impl SlackView + Send, Error>> + Send;

    /// Handles a `RoswaalSlackRequest` and returns the `SlackBlocks` that form the content of the
    /// response.
    ///
    /// If the command in the request is long running, then the function immediately returns a
    /// message to indicating that the request is being handled. In the meantime, the request is
    /// being handled on a background task, and it the returned message will be sent to slack in
    /// the background via `message_sender` when the request is fully handled.
    async fn handle_request(
        self,
        request: RoswaalSlackRequest,
        message_sender: Arc<(impl SlackSendMessage + Send + Sync + 'static)>
    ) -> Result<SlackBlocks, Error> where Self: Send, Self: Sync {
        if request.command.is_long_running() {
            // NB: A long running command must spin up an unstructered background task since we
            // have to send an ack response to slack within 3 seconds.
            spawn(async move {
                let message = self.handle_command(&request.command, &request.text)
                    .await
                    .map(|view| {
                        SlackMessage::new(&request.channel_id, &view, &request.response_url)
                    })?;
                message_sender.send(&message).await?;
                Ok::<(), Error>(())
            });
            Ok(render_slack_view(&PendingView))
        } else {
            self.handle_command(&request.command, &request.text)
                .await
                .map(|view| render_slack_view(&view))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use tokio::{sync::Mutex, time::sleep};

    use crate::slack::{message::{SlackMessage, SlackSendMessage}, pending_view::PendingView, ui_lib::block_kit_views::SlackDivider};

    use super::*;

    struct TestSlackMessager {
        messages: Arc<Mutex<Vec<SlackMessage>>>
    }

    impl TestSlackMessager {
        fn new() -> Self {
            Self { messages: Arc::new(Mutex::new(vec![])) }
        }
    }

    impl SlackSendMessage for TestSlackMessager {
        async fn send(&self, message: &SlackMessage) -> Result<(), Error> {
            let mut messages = self.messages.lock().await;
            (*messages).push(message.clone());
            Ok(())
        }
    }

    const TEST_VIEW: SlackDivider = SlackDivider;

    struct TestCommandHandler;

    impl RoswaalSlackHandler for TestCommandHandler {
        async fn handle_command(self, _: &RoswaalSlackCommand, _: &str) -> Result<impl SlackView, Error> {
            Ok(TEST_VIEW)
        }
    }

    impl RoswaalSlackRequest {
        fn for_testing(command: RoswaalSlackCommand) -> Self {
            Self {
                channel_id: "bob".to_string(),
                text: "abc, 12.080282, 120.298722".to_string(),
                command,
                response_url: "https://api.slack.com/chat.postMessage".to_string()
            }
        }
    }

    #[tokio::test]
    async fn non_long_living_command_does_uses_view_as_direct_response() {
        let messenger = Arc::new(TestSlackMessager::new());
        let blocks = TestCommandHandler.handle_request(
            RoswaalSlackRequest::for_testing(RoswaalSlackCommand::ViewLocations),
            messenger.clone()
        ).await.unwrap();
        assert_eq!(blocks, render_slack_view(&TEST_VIEW));
        assert!(messenger.messages.lock().await.is_empty())
    }

    #[tokio::test]
    async fn long_running_command_sends_a_deffered_message() {
        let messenger = Arc::new(TestSlackMessager::new());
        let request = RoswaalSlackRequest::for_testing(RoswaalSlackCommand::AddTests);
        let expected_message = SlackMessage::new(
            &request.channel_id,
            &TEST_VIEW,
            &request.response_url
        );
        let blocks = TestCommandHandler.handle_request(
            request,
            messenger.clone()
        ).await.unwrap();
        assert_eq!(blocks, render_slack_view(&PendingView));
        wait().await;
        let messages = messenger.messages.lock().await;
        let messages = (*messages).clone();
        assert_eq!(messages, vec![expected_message])
    }

    async fn wait() {
        sleep(Duration::from_millis(10)).await
    }
}
