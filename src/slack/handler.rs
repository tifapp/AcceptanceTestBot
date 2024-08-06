use std::{future::Future, sync::Arc};

use super::{
    command::RoswaalSlackCommand,
    error_view::ErrorView,
    message::{SlackMessage, SlackSendMessage},
    pending_view::PendingView,
    ui_lib::{
        blocks::SlackBlocks,
        slack_view::{render_slack_view, SlackView},
    },
};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use tokio::spawn;

/// A request from slack.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct RoswaalSlackRequest {
    channel_id: String,
    text: String,
    command: RoswaalSlackCommand,
    response_url: String,
}

impl RoswaalSlackRequest {
    pub fn new(
        channel_id: String,
        text: String,
        command: RoswaalSlackCommand,
        response_url: String,
    ) -> Self {
        Self {
            channel_id,
            text,
            command,
            response_url,
        }
    }
}

/// A trait for handling slack commands.
pub trait RoswaalSlackHandler: Sized + 'static {
    /// Handles the specified command and command text, and returns a `SlackView` with the contents
    /// of the response to the command.
    fn handle_command(
        &self,
        command: &RoswaalSlackCommand,
        command_text: &str,
    ) -> impl Future<Output = Result<impl SlackView + Send, Error>> + Send;
}

/// Handles a `RoswaalSlackRequest` and returns the `SlackBlocks` that form the content of the
/// response.
///
/// If the command in the request is long running, then the function immediately returns a
/// message to indicating that the request is being handled. In the meantime, the request is
/// being handled on a background task, and it the returned message will be sent to slack in
/// the background via `messenger` when the handling of the request is finished.
pub async fn handle_slack_request(
    handler: Arc<impl RoswaalSlackHandler + Send + Sync>,
    request: RoswaalSlackRequest,
    messenger: Arc<(impl SlackSendMessage + Send + Sync + 'static)>,
) -> SlackBlocks {
    if request.command.is_long_running() {
        // NB: A long running command must spin up an unstructered background task since we
        // have to send an ack response to slack within 3 seconds.
        spawn(async move {
            let view = view_for_request(handler.as_ref(), &request).await;
            let message = SlackMessage::new(&request.channel_id, &view, &request.response_url);
            messenger.send(&message).await
        });
        render_slack_view(&PendingView)
    } else {
        render_slack_view(&view_for_request(handler.as_ref(), &request).await)
    }
}

async fn view_for_request(
    handler: &impl RoswaalSlackHandler,
    request: &RoswaalSlackRequest,
) -> impl SlackView {
    match handler
        .handle_command(&request.command, &request.text)
        .await
    {
        Ok(view) => view.erase_to_any_view(),
        Err(error) => ErrorView::new(error).erase_to_any_view(),
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use serde::Serialize;
    use tokio::{sync::Mutex, time::sleep};

    use crate::{
        slack::{
            message::{SlackMessage, SlackSendMessage},
            pending_view::PendingView,
            ui_lib::{block_kit_views::SlackDivider, empty_view::EmptySlackView},
        },
        utils::test_error::TestError,
    };

    use super::*;

    struct TestSlackMessager {
        messages: Arc<Mutex<Vec<SlackMessage>>>,
    }

    impl TestSlackMessager {
        fn new() -> Self {
            Self {
                messages: Arc::new(Mutex::new(vec![])),
            }
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

    struct SuccessfulHandler;

    impl RoswaalSlackHandler for SuccessfulHandler {
        async fn handle_command(
            &self,
            _: &RoswaalSlackCommand,
            _: &str,
        ) -> Result<impl SlackView, Error> {
            Ok(TEST_VIEW)
        }
    }

    struct FailingHandler;

    impl RoswaalSlackHandler for FailingHandler {
        async fn handle_command(
            &self,
            _: &RoswaalSlackCommand,
            _: &str,
        ) -> Result<impl SlackView, Error> {
            Err::<EmptySlackView, Error>(Error::new(TestError))
        }
    }

    impl RoswaalSlackRequest {
        fn for_testing(command: RoswaalSlackCommand) -> Self {
            Self {
                channel_id: "bob".to_string(),
                text: "abc, 12.080282, 120.298722".to_string(),
                command,
                response_url: "https://api.slack.com/chat.postMessage".to_string(),
            }
        }
    }

    #[tokio::test]
    async fn non_long_running_command_does_uses_view_as_direct_response() {
        let messenger = Arc::new(TestSlackMessager::new());
        let blocks = handle_slack_request(
            Arc::new(SuccessfulHandler),
            RoswaalSlackRequest::for_testing(RoswaalSlackCommand::ViewLocations),
            messenger.clone(),
        )
        .await;
        assert_eq!(blocks, render_slack_view(&TEST_VIEW));
        assert!(messenger.messages.lock().await.is_empty())
    }

    #[tokio::test]
    async fn long_running_command_sends_a_deffered_message() {
        let messenger = Arc::new(TestSlackMessager::new());
        let request = RoswaalSlackRequest::for_testing(RoswaalSlackCommand::AddTests);
        let expected_message =
            SlackMessage::new(&request.channel_id, &TEST_VIEW, &request.response_url);
        let blocks =
            handle_slack_request(Arc::new(SuccessfulHandler), request, messenger.clone()).await;
        assert_eq!(blocks, render_slack_view(&PendingView));
        wait().await;
        let messages = messenger.messages.lock().await;
        let messages = (*messages).clone();
        assert_eq!(messages, vec![expected_message])
    }

    #[tokio::test]
    async fn non_long_running_command_returns_error_view_when_failure_occurs() {
        let messenger = Arc::new(TestSlackMessager::new());
        let blocks = handle_slack_request(
            Arc::new(FailingHandler),
            RoswaalSlackRequest::for_testing(RoswaalSlackCommand::ViewLocations),
            messenger.clone(),
        )
        .await;
        assert_error_blocks(&blocks);
        assert!(messenger.messages.lock().await.is_empty())
    }

    #[tokio::test]
    async fn long_running_command_sends_a_deffered_error_message_when_failure_occurs() {
        let messenger = Arc::new(TestSlackMessager::new());
        let request = RoswaalSlackRequest::for_testing(RoswaalSlackCommand::AddTests);
        let blocks =
            handle_slack_request(Arc::new(FailingHandler), request, messenger.clone()).await;
        assert_eq!(blocks, render_slack_view(&PendingView));
        wait().await;
        let messages = messenger.messages.lock().await;
        assert_error_blocks((*messages).first().unwrap())
    }

    fn assert_error_blocks(blocks: &impl Serialize) {
        let json = serde_json::to_string(&blocks).unwrap();
        assert!(json.contains("An Error Occurred"));
    }

    async fn wait() {
        sleep(Duration::from_millis(10)).await
    }
}
