use std::sync::Arc;

use gouqi::{Board, Credentials, Issue, Project, SearchOptions, Sprint, r#async::Jira};

mod board_config;

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

    pub async fn get_projects(&self) -> gouqi::Result<Vec<Project>> {
        self.client.projects().list().await
    }

    pub async fn get_boards(&self) -> gouqi::Result<Vec<Board>> {
        let res = self
            .client
            .boards()
            .list(&SearchOptions::builder().build())
            .await?;
        Ok(res.values)
    }

    pub async fn get_board(&self, board_id: u64) -> gouqi::Result<Board> {
        self.client.boards().get(board_id).await
    }

    pub async fn get_project_boards(&self, project_key: &str) -> gouqi::Result<Vec<Board>> {
        let res = self
            .client
            .boards()
            .list(
                &SearchOptions::builder()
                    .project_key_or_id(project_key)
                    .build(),
            )
            .await?;
        Ok(res.values)
    }

    pub async fn get_board_sprints(&self, board_id: u64) -> gouqi::Result<Vec<Sprint>> {
        let board = self.client.boards().get(board_id).await?;
        let res = self
            .client
            .sprints()
            .list(&board, &SearchOptions::builder().build())
            .await?;
        Ok(res.values)
    }

    pub async fn get_current_sprint(&self, board_id: u64) -> gouqi::Result<Sprint> {
        let opts = SearchOptions::builder().state("active").build();
        let board = self.client.boards().get(board_id).await?;
        let mut page = self.client.sprints().list(&board, &opts).await?;
        page.values.pop().ok_or(gouqi::Error::NotFound)
    }

    pub async fn get_current_sprint_issues(&self, board_id: u64) -> gouqi::Result<Issues> {
        let sprint = self.get_current_sprint(board_id).await?;
        self.get_sprint_issues(sprint.id).await
    }

    pub async fn get_sprint_issues(&self, sprint_id: u64) -> gouqi::Result<Issues> {
        let jql = format!("sprint = {sprint_id}");
        let opts = SearchOptions::builder().all_fields().build();
        let res = self.client.search().list(&jql, &opts).await?;
        Ok(res.issues)
    }
}
