use std::sync::Arc;

use gouqi::{Board, Credentials, Issue, Project, SearchOptions, Sprint, r#async::Jira};
use serde::Deserialize;
use serde_json::Value;

pub type Issues = Vec<Issue>;

#[derive(Deserialize, Debug, Clone)]
pub struct BoardConfig {
    #[serde(rename = "columnConfig")]
    pub columns: Vec<BoardColumn>,
    pub estimation: Estimation,
}

impl Default for BoardConfig {
    fn default() -> Self {
        BoardConfig {
            columns: vec![BoardColumn::default()],
            estimation: Estimation::default(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub enum Estimation {
    StoryPoints(String),
    DateBased(String),
}

impl Estimation {
    pub fn field_id(&self) -> &str {
        match self {
            Estimation::StoryPoints(field_id) => field_id,
            Estimation::DateBased(field_id) => field_id,
        }
    }

    pub fn extract_value(&self, issue: &Issue) -> Option<f64> {
        match self {
            Estimation::StoryPoints(field_id) | Estimation::DateBased(field_id) => {
                if let Some(value) = issue.fields.get(field_id)
                    && let Some(num) = value.as_f64()
                {
                    return Some(num);
                }
                None
            }
        }
    }
}

impl Default for Estimation {
    fn default() -> Self {
        Estimation::StoryPoints("customfield_10002".to_string())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct BoardColumn {
    pub name: String,
    pub statuses: Vec<ColumnStatusRef>,
}

impl Default for BoardColumn {
    fn default() -> Self {
        BoardColumn {
            name: "TODO".to_string(),
            statuses: Vec::new(),
        }
    }
}

impl BoardColumn {
    fn post_process(&mut self) {
        self.name = self.name.to_uppercase();
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ColumnStatusRef {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
}

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
        let jql = format!("sprint = {}", sprint_id);
        let opts = SearchOptions::builder().all_fields().build();
        let res = self.client.search().list(&jql, &opts).await?;
        Ok(res.issues)
    }

    pub async fn get_board_config(&self, board_id: u64) -> gouqi::Result<BoardConfig> {
        let resp: Value = self
            .client
            .get("agile", &format!("/board/{}/configuration", board_id))
            .await?;
        let columns_cfg = resp.get("columnConfig");
        if columns_cfg.is_none() {
            return gouqi::Result::Err(gouqi::Error::ConfigError {
                message: "No columnConfig found".to_string(),
            });
        }
        let columns_cfg: Vec<BoardColumn> = serde_json::from_value(
            columns_cfg
                .unwrap()
                .get("columns")
                .ok_or(gouqi::Error::ConfigError {
                    message: "No columns found".to_string(),
                })?
                .clone(),
        )?;
        let estim: Estimation = {
            let est_cfg = resp.get("estimation");
            if est_cfg.is_none() {
                return gouqi::Result::Err(gouqi::Error::ConfigError {
                    message: "No estimation config found".to_string(),
                });
            }
            match est_cfg
                .unwrap()
                .get("field")
                .ok_or(gouqi::Error::ConfigError {
                    message: "No estimation field found".to_string(),
                })?
                .get("displayName")
                .ok_or(gouqi::Error::ConfigError {
                    message: "No estimation displayName found".to_string(),
                })?
                .as_str()
                .ok_or(gouqi::Error::ConfigError {
                    message: "Estimation displayName is not a string".to_string(),
                })? {
                "Story Points" => Estimation::StoryPoints(
                    est_cfg
                        .unwrap()
                        .get("field")
                        .unwrap()
                        .get("fieldId")
                        .ok_or(gouqi::Error::ConfigError {
                            message: "No fieldId found for Story Points".to_string(),
                        })?
                        .as_str()
                        .ok_or(gouqi::Error::ConfigError {
                            message: "fieldId for Story Points is not a string".to_string(),
                        })?
                        .to_string(),
                ),
                "Days" => Estimation::DateBased(
                    est_cfg
                        .unwrap()
                        .get("field")
                        .unwrap()
                        .get("fieldId")
                        .ok_or(gouqi::Error::ConfigError {
                            message: "No fieldId found for Days".to_string(),
                        })?
                        .as_str()
                        .ok_or(gouqi::Error::ConfigError {
                            message: "fieldId for Days is not a string".to_string(),
                        })?
                        .to_string(),
                ),
                _ => {
                    return gouqi::Result::Err(gouqi::Error::ConfigError {
                        message: "Unknown estimation type".to_string(),
                    });
                }
            }
        };
        Ok(BoardConfig {
            columns: columns_cfg
                .into_iter()
                .map(|mut col| {
                    col.post_process();
                    col
                })
                .collect(),
            estimation: estim,
        })
    }
}
