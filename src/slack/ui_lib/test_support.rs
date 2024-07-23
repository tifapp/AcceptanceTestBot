use super::slack_view::SlackView;
use super::message::SlackMessage;

#[cfg(test)]
pub fn assert_blocks_json(view: &impl SlackView, json: &str) {
    let message = SlackMessage::for_testing(view);
    let expected_json = format!("{{\"channel\":\"__TEST__\",\"blocks\":{}}}", json);
    let json = serde_json::to_string(&message).unwrap();
    assert_eq!(json, expected_json)
}
