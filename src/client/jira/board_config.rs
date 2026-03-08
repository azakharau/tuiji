use serde_json::Value;

use super::JiraClient;
use crate::data::{BoardColumn, BoardConfig, Estimation};

impl JiraClient {
    pub async fn get_board_config(&self, board_id: u64) -> gouqi::Result<BoardConfig> {
        let resp: Value = self
            .client
            .get("agile", &format!("/board/{board_id}/configuration"))
            .await?;

        let columns = parse_columns(&resp)?;
        let estimation = parse_estimation(&resp)?;

        Ok(BoardConfig {
            columns: columns
                .into_iter()
                .map(|mut col| {
                    col.name = col.name.to_uppercase();
                    col
                })
                .collect(),
            estimation,
        })
    }
}

fn parse_columns(resp: &Value) -> gouqi::Result<Vec<BoardColumn>> {
    let columns_cfg = resp
        .get("columnConfig")
        .ok_or_else(|| gouqi::Error::ConfigError {
            message: "No columnConfig found".to_string(),
        })?;
    let columns = columns_cfg
        .get("columns")
        .ok_or_else(|| gouqi::Error::ConfigError {
            message: "No columns found".to_string(),
        })?;
    serde_json::from_value(columns.clone()).map_err(Into::into)
}

fn parse_estimation(resp: &Value) -> gouqi::Result<Estimation> {
    let estimation_cfg = resp
        .get("estimation")
        .ok_or_else(|| gouqi::Error::ConfigError {
            message: "No estimation config found".to_string(),
        })?;
    let field = estimation_cfg
        .get("field")
        .ok_or_else(|| gouqi::Error::ConfigError {
            message: "No estimation field found".to_string(),
        })?;
    let display_name = field
        .get("displayName")
        .and_then(Value::as_str)
        .ok_or_else(|| gouqi::Error::ConfigError {
            message: "Estimation displayName is not a string".to_string(),
        })?;
    let field_id = field
        .get("fieldId")
        .and_then(Value::as_str)
        .ok_or_else(|| gouqi::Error::ConfigError {
            message: "No estimation fieldId found".to_string(),
        })?
        .to_string();

    match display_name {
        "Story Points" => Ok(Estimation::StoryPoints(field_id)),
        "Days" => Ok(Estimation::DateBased(field_id)),
        _ => Err(gouqi::Error::ConfigError {
            message: "Unknown estimation type".to_string(),
        }),
    }
}
