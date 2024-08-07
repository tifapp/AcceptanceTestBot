use std::sync::Arc;

use anyhow::Error;
use axum::Form;
use axum::{
    extract::Query, http::StatusCode, middleware::from_fn, response::IntoResponse, routing::post,
    serve, Json, Router,
};
#[cfg(test)]
use axum_test::TestServer;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use crate::slack::ui_lib::blocks::SlackBlocks;
use crate::{
    git::branch_name::RoswaalOwnedGitBranchName,
    operations::{
        add_locations::AddLocationsStatus, add_tests::AddTestsStatus,
        close_branch::CloseBranchStatus, load_all_locations::LoadAllLocationsStatus,
        merge_branch::MergeBranchStatus, remove_tests::RemoveTestsStatus,
        save_progress::save_test_progress, search_tests::SearchTestsStatus,
    },
    slack::{
        add_locations_view::AddLocationsView,
        add_tests_view::AddTestsView,
        command::RoswaalSlackCommand,
        handler::{handle_slack_request, RoswaalSlackHandler, RoswaalSlackRequest},
        locations_list_view::LocationsListView,
        message::SlackSendMessage,
        remove_tests_view::RemoveTestsView,
        search_tests_view::SearchTestsView,
        ui_lib::slack_view::SlackView,
    },
    tests_data::progress::RoswaalTestProgressUpload,
    utils::sqlite::RoswaalSqlite,
};

use super::{
    password::check_password_middleware, response_result::ResponseResult,
    server_environment::ServerEnvironment,
};

/// Runs this tool as an http server using the specified `ServerEnvironment`.
pub async fn run_http_server(environment: Arc<ServerEnvironment>) -> anyhow::Result<()> {
    let server = roswaal_server(environment.clone());
    let listener = TcpListener::bind(environment.address()).await?;
    Ok(serve(listener, server).await?)
}

fn roswaal_server(environment: Arc<ServerEnvironment>) -> Router<()> {
    let slack_handler = Arc::new(HTTPSlackHandler {
        environment: environment.clone(),
    });
    let messenger = environment.slack_messenger();
    let password = environment.password();
    let password_protection =
        from_fn(move |req, next| check_password_middleware(req, next, password.clone()));
    let sqlite_close = environment.sqlite();
    let sqlite_progress = environment.sqlite();
    let sqlite_merge = environment.sqlite();
    Router::new()
        .route(
            "/merge",
            post(move |query| post_merge_branch(query, sqlite_merge)),
        )
        .route(
            "/close",
            post(move |query| post_close_branch(query, sqlite_close)),
        )
        .route(
            "/progress",
            post(move |body| post_progess(body, sqlite_progress)),
        )
        .route_layer(password_protection)
        .route(
            "/slack",
            post(move |body| post_slack_request(body, slack_handler, messenger)),
        )
}

#[derive(Debug, Deserialize)]
struct ProgressUpload {
    results: Vec<RoswaalTestProgressUpload>,
}

async fn post_progess(
    Json(upload): Json<ProgressUpload>,
    sqlite: Arc<RoswaalSqlite>,
) -> impl IntoResponse {
    let result = save_test_progress(&upload.results, sqlite.as_ref())
        .await
        .map(|_| StatusCode::NO_CONTENT);
    ResponseResult::new(result)
}

#[derive(Debug, Deserialize)]
struct BranchQueryParameters {
    branch: RoswaalOwnedGitBranchName,
}

async fn post_merge_branch(
    Query(query): Query<BranchQueryParameters>,
    sqlite: Arc<RoswaalSqlite>,
) -> impl IntoResponse {
    let result = MergeBranchStatus::from_merging_branch_with_name(&query.branch, sqlite.as_ref())
        .await
        .map(|s| match s {
            MergeBranchStatus::Merged(_) => StatusCode::NO_CONTENT,
            MergeBranchStatus::UnknownBranchKind(_) => StatusCode::BAD_REQUEST,
        });
    ResponseResult::new(result)
}

async fn post_close_branch(
    Query(query): Query<BranchQueryParameters>,
    sqlite: Arc<RoswaalSqlite>,
) -> impl IntoResponse {
    let result = CloseBranchStatus::from_closing_branch(&query.branch, sqlite.as_ref())
        .await
        .map(|s| match s {
            CloseBranchStatus::Closed(_) => StatusCode::NO_CONTENT,
            CloseBranchStatus::UnknownBranchKind(_) => StatusCode::BAD_REQUEST,
        });
    ResponseResult::new(result)
}

#[derive(Serialize)]
struct SlackResponse {
    blocks: SlackBlocks,
}

async fn post_slack_request(
    Form(request): Form<RoswaalSlackRequest>,
    slack_handler: Arc<HTTPSlackHandler>,
    messenger: Arc<impl SlackSendMessage + Send + Sync + 'static>,
) -> impl IntoResponse {
    Json(SlackResponse {
        blocks: handle_slack_request(slack_handler, request, messenger).await,
    })
}

struct HTTPSlackHandler {
    environment: Arc<ServerEnvironment>,
}

impl RoswaalSlackHandler for HTTPSlackHandler {
    async fn handle_command(
        &self,
        command: &RoswaalSlackCommand,
        command_text: &str,
    ) -> Result<impl SlackView, Error> {
        match command {
            RoswaalSlackCommand::ViewTests => {
                let status = SearchTestsStatus::from_searching_tests(
                    command_text,
                    self.environment.sqlite().as_ref(),
                )
                .await?;
                Ok(SearchTestsView::new(status).erase_to_any_view())
            }
            RoswaalSlackCommand::AddTests => {
                let status = AddTestsStatus::from_adding_tests(
                    command_text,
                    self.environment.sqlite().as_ref(),
                    self.environment.github_pull_request_open(),
                    self.environment.git_repository(),
                )
                .await?;
                Ok(AddTestsView::new(status).erase_to_any_view())
            }
            RoswaalSlackCommand::RemoveTests => {
                let status = RemoveTestsStatus::from_removing_tests(
                    command_text,
                    self.environment.sqlite().as_ref(),
                    self.environment.git_repository(),
                    self.environment.github_pull_request_open(),
                )
                .await?;
                Ok(RemoveTestsView::new(status).erase_to_any_view())
            }
            RoswaalSlackCommand::ViewLocations => {
                let status = LoadAllLocationsStatus::from_stored_locations(
                    self.environment.sqlite().as_ref(),
                )
                .await?;
                Ok(LocationsListView::new(status).erase_to_any_view())
            }
            RoswaalSlackCommand::AddLocations => {
                let status = AddLocationsStatus::from_adding_locations(
                    command_text,
                    self.environment.git_repository(),
                    self.environment.sqlite().as_ref(),
                    self.environment.github_pull_request_open(),
                )
                .await?;
                Ok(AddLocationsView::new(status).erase_to_any_view())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum_test::TestResponse;
    use dotenv::dotenv;
    use serde_json::{json, Value};
    use sqlx::{prelude::FromRow, query_as, Sqlite};
    use tokio::{fs::remove_file, time::sleep};

    use crate::{
        git::{branch_name::RoswaalOwnedGitBranchName, test_support::with_clean_test_repo_access},
        http::password::DEV_RAW_ENDPOINT_PASSWORD,
        with_transaction,
    };

    use super::*;

    const ACCEPTANCE_TEST_CHANNEL_ID: &str = "C06PSMAB7QV";
    const SLACK_RESPONSE_URL: &str = "https://slack.com/api/chat.postMessage";

    #[tokio::test]
    async fn add_and_merge_report_progress_tests() {
        with_clean_test_repo_access(async {
            remove_file("./roswaal-dev.sqlite").await?;
            let app = test_app().await;
            app.add_tests(
                "\
    ```
    New Test: Some Test
    Step A: This is a test
    Requirement A: Make sure that it gets added correctly
    Step B: Merging test
    Requirement B: Merge the test into the tool
    Step C: Progress Update
    Requirement C: Update the progress on the test
    ```
    ",
            )
            .await;
            let branch_name = app.most_recent_branch_name().await?;
            app.merge_test(&branch_name).await;
            let resp = app
                .upload_progress(&json!({
                    "results": [
                        {
                            "testName": "Some Test",
                            "commandFailureOrdinal": 1,
                            "error": {
                                "message": "I died",
                                "stackTrace": "Figure it out..."
                            }
                        }
                    ]
                }))
                .await;
            resp.assert_status(StatusCode::NO_CONTENT);
            Ok(())
        })
        .await
        .unwrap()
    }

    struct TestApp {
        server: TestServer,
        environment: Arc<ServerEnvironment>,
    }

    impl TestApp {
        async fn add_tests(&self, tests_str: &str) {
            let form_data = RoswaalSlackRequest::new(
                ACCEPTANCE_TEST_CHANNEL_ID.to_string(),
                tests_str.to_string(),
                RoswaalSlackCommand::AddTests,
                SLACK_RESPONSE_URL.to_string(),
            );
            self.server.post("/slack").form(&form_data).await;
            // NB: The request will respond immediately with a "pending" message, we'll need to
            // manually wait for the actual work of the request to finish.
            sleep(Duration::from_millis(10_000)).await
        }

        async fn merge_test(&self, branch: &RoswaalOwnedGitBranchName) {
            self.server
                .post("/merge")
                .add_query_param("password", DEV_RAW_ENDPOINT_PASSWORD)
                .add_query_param("branch", branch)
                .await;
        }

        async fn upload_progress(&self, value: &Value) -> TestResponse {
            self.server
                .post("/progress")
                .add_query_param("password", DEV_RAW_ENDPOINT_PASSWORD)
                .json(value)
                .await
        }

        async fn most_recent_branch_name(&self) -> anyhow::Result<RoswaalOwnedGitBranchName> {
            let sqlite = self.environment.sqlite();
            let mut transaction = sqlite.transaction().await?;
            with_transaction!(transaction, async {
                let branch_name = query_as::<Sqlite, SqliteBranchName>(
                    "SELECT unmerged_branch_name FROM Tests ORDER BY creation_date DESC",
                )
                .fetch_one(transaction.connection())
                .await?;
                Ok(branch_name.unmerged_branch_name)
            })
        }
    }

    #[derive(Debug, FromRow)]
    struct SqliteBranchName {
        unmerged_branch_name: RoswaalOwnedGitBranchName,
    }

    async fn test_app() -> TestApp {
        dotenv().unwrap();
        let environment = Arc::new(ServerEnvironment::dev().await.unwrap());
        TestApp {
            server: TestServer::new(roswaal_server(environment.clone())).unwrap(),
            environment,
        }
    }
}
