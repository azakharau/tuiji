use std::sync::Arc;

use gouqi::{Board, Credentials, Issue, SearchOptions, Sprint, r#async::Jira};

mod board_config;
pub mod write;

pub type Issues = Vec<Issue>;

#[derive(Clone)]
pub struct JiraClient {
    client: Arc<Jira>,
}

impl JiraClient {
    pub fn new(base_url: &str, username: &str, api_token: &str) -> gouqi::Result<Self> {
        let credentials = Credentials::Basic(username.to_string(), api_token.to_string());
        let client = Arc::new(Jira::new(base_url, credentials)?);
        Ok(JiraClient { client })
    }

    pub async fn get_boards(&self) -> gouqi::Result<Vec<Board>> {
        let res = self
            .client
            .boards()
            .list(&SearchOptions::builder().build())
            .await?;
        Ok(res.values)
    }

    async fn get_board(&self, board_id: u64) -> gouqi::Result<Board> {
        self.client.boards().get(board_id).await
    }

    pub async fn get_current_sprint(&self, board_id: u64) -> gouqi::Result<Sprint> {
        let opts = SearchOptions::builder().state("active").build();
        let board = self.get_board(board_id).await?;
        let mut page = self.client.sprints().list(&board, &opts).await?;
        page.values.pop().ok_or(gouqi::Error::NotFound)
    }

    pub async fn get_sprint_issues(&self, sprint_id: u64) -> gouqi::Result<Issues> {
        self.search_issues(&format!("sprint = {sprint_id}")).await
    }

    pub async fn search_issues(&self, jql: &str) -> gouqi::Result<Issues> {
        let opts = SearchOptions::builder().all_fields().build();
        let res = self.client.search().list(jql, &opts).await?;
        Ok(res.issues)
    }
}
