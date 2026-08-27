use std::fmt::Display;

use reqwest::{Method, StatusCode, header::ACCEPT};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use crate::data::model::{IssueDraft, TransitionChoice};

pub struct JiraWriteClient {
    http: reqwest::Client,
    base: String,
    username: String,
    token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Permanent(String),
    #[error("{0}")]
    Transient(String),
}

impl JiraWriteClient {
    pub fn new(base_url: &str, username: &str, token: &str) -> color_eyre::Result<Self> {
        let base = base_url.trim_end_matches('/');
        reqwest::Url::parse(base)?;

        Ok(Self {
            http: reqwest::Client::new(),
            base: base.to_string(),
            username: username.to_string(),
            token: token.to_string(),
        })
    }

    pub async fn myself(&self) -> Result<(String, String), WriteError> {
        let response: MyselfResponse =
            self.decode(self.send(Method::GET, "/myself", None).await?)?;
        Ok((response.account_id, response.display_name))
    }

    pub async fn create_issue(&self, draft: &IssueDraft) -> Result<String, WriteError> {
        let mut fields = Map::new();
        fields.insert("project".to_string(), json!({ "key": &draft.project_key }));
        fields.insert(
            "issuetype".to_string(),
            json!({ "name": &draft.issue_type }),
        );
        fields.insert("summary".to_string(), json!(&draft.summary));

        if let Some(description) = draft
            .description
            .as_deref()
            .filter(|description| !description.is_empty())
        {
            fields.insert("description".to_string(), self.adf(description)?);
        }

        let response: CreateIssueResponse = self.decode(
            self.send(Method::POST, "/issue", Some(json!({ "fields": fields })))
                .await?,
        )?;
        Ok(response.key)
    }

    pub async fn edit_fields(&self, key: &str, fields: Value) -> Result<(), WriteError> {
        self.send(Method::PUT, &format!("/issue/{key}"), Some(fields))
            .await?;
        Ok(())
    }

    pub async fn add_comment(&self, key: &str, body: &str) -> Result<(), WriteError> {
        self.send(
            Method::POST,
            &format!("/issue/{key}/comment"),
            Some(json!({ "body": self.adf(body)? })),
        )
        .await?;
        Ok(())
    }

    pub async fn set_assignee(&self, key: &str, account_id: &str) -> Result<(), WriteError> {
        self.send(
            Method::PUT,
            &format!("/issue/{key}/assignee"),
            Some(json!({ "accountId": account_id })),
        )
        .await?;
        Ok(())
    }

    pub async fn list_transitions(&self, key: &str) -> Result<Vec<TransitionChoice>, WriteError> {
        let response: TransitionsResponse = self.decode(
            self.send(Method::GET, &format!("/issue/{key}/transitions"), None)
                .await?,
        )?;

        Ok(response
            .transitions
            .into_iter()
            .map(|transition| TransitionChoice {
                id: transition.id,
                name: transition.name,
                to_status: transition.to.name,
            })
            .collect())
    }

    pub async fn trigger_transition(
        &self,
        key: &str,
        transition_id: &str,
    ) -> Result<(), WriteError> {
        self.send(
            Method::POST,
            &format!("/issue/{key}/transitions"),
            Some(json!({ "transition": { "id": transition_id } })),
        )
        .await?;
        Ok(())
    }

    pub async fn issue_types(&self, project_key: &str) -> Result<Vec<String>, WriteError> {
        let response: CreateMetaResponse = self.decode(
            self.send(
                Method::GET,
                &format!("/issue/createmeta?projectKeys={project_key}&expand=projects.issuetypes"),
                None,
            )
            .await?,
        )?;

        let issue_types = if let Some(project) = response.projects.into_iter().next() {
            Self::creatable_type_names(project.issuetypes)
        } else {
            let response: IssueTypesResponse = self.decode(
                self.send(
                    Method::GET,
                    &format!("/issue/createmeta/{project_key}/issuetypes"),
                    None,
                )
                .await?,
            )?;
            Self::creatable_type_names(response.values)
        };

        if issue_types.is_empty() {
            Err(WriteError::Permanent(format!(
                "Jira returned no creatable issue types for project {project_key}"
            )))
        } else {
            Ok(issue_types)
        }
    }

    async fn send(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, WriteError> {
        let mut request = self
            .http
            .request(method, format!("{}/rest/api/3{path}", self.base))
            .basic_auth(&self.username, Some(&self.token))
            .header(ACCEPT, "application/json");
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(Self::transient)?;
        let status = response.status();
        let body = response.text().await.map_err(Self::transient)?;

        if status.is_success() {
            if body.trim().is_empty() {
                Ok(Value::Null)
            } else {
                serde_json::from_str(&body).map_err(Self::transient)
            }
        } else {
            let message = Self::error_message(status, &body);
            match status.as_u16() {
                409 | 412 => Err(WriteError::Conflict(message)),
                408 | 429 | 500..=599 => Err(WriteError::Transient(message)),
                400 | 401 | 403 | 404 | 405 | 422 => Err(WriteError::Permanent(message)),
                _ => Err(WriteError::Permanent(message)),
            }
        }
    }

    fn adf(&self, text: &str) -> Result<Value, WriteError> {
        serde_json::to_value(gouqi::AdfDocument::from_text(text)).map_err(Self::permanent)
    }

    fn decode<T: DeserializeOwned>(&self, value: Value) -> Result<T, WriteError> {
        serde_json::from_value(value).map_err(Self::transient)
    }

    fn creatable_type_names(issue_types: Vec<IssueTypeResponse>) -> Vec<String> {
        issue_types
            .into_iter()
            .filter(|issue_type| !issue_type.subtask)
            .map(|issue_type| issue_type.name)
            .collect()
    }

    fn error_message(status: StatusCode, body: &str) -> String {
        let mut messages = Vec::new();
        if let Ok(value) = serde_json::from_str::<Value>(body) {
            if let Some(error_messages) = value.get("errorMessages").and_then(Value::as_array) {
                messages.extend(error_messages.iter().map(Self::message_value));
            }
            if let Some(errors) = value.get("errors").and_then(Value::as_object) {
                messages.extend(errors.values().map(Self::message_value));
            }
        }

        if !messages.is_empty() {
            messages.join("; ")
        } else if body.trim().is_empty() {
            status.to_string()
        } else {
            body.chars().take(500).collect()
        }
    }

    fn message_value(value: &Value) -> String {
        value
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| value.to_string())
    }

    fn permanent(error: impl Display) -> WriteError {
        WriteError::Permanent(error.to_string())
    }

    fn transient(error: impl Display) -> WriteError {
        WriteError::Transient(error.to_string())
    }
}

#[derive(Deserialize)]
struct MyselfResponse {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Deserialize)]
struct CreateIssueResponse {
    key: String,
}

#[derive(Deserialize)]
struct TransitionsResponse {
    transitions: Vec<TransitionResponse>,
}

#[derive(Deserialize)]
struct TransitionResponse {
    id: String,
    name: String,
    to: TransitionStatusResponse,
}

#[derive(Deserialize)]
struct TransitionStatusResponse {
    name: String,
}

#[derive(Deserialize)]
struct CreateMetaResponse {
    #[serde(default)]
    projects: Vec<CreateMetaProjectResponse>,
}

#[derive(Deserialize)]
struct CreateMetaProjectResponse {
    #[serde(default)]
    issuetypes: Vec<IssueTypeResponse>,
}

#[derive(Deserialize)]
struct IssueTypesResponse {
    #[serde(default)]
    values: Vec<IssueTypeResponse>,
}

#[derive(Deserialize)]
struct IssueTypeResponse {
    name: String,
    #[serde(default)]
    subtask: bool,
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::{JiraWriteClient, WriteError};

    #[tokio::test]
    async fn add_comment_posts_an_adf_object_to_the_v3_endpoint() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/api/3/issue/ABC-1/comment"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({})))
            .expect(1)
            .mount(&server)
            .await;
        let client = JiraWriteClient::new(&server.uri(), "user", "token").unwrap();

        client
            .add_comment("ABC-1", "line one\nline two")
            .await
            .unwrap();

        let requests = server.received_requests().await.unwrap();
        let body: Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert!(requests[0].url.path().contains("/rest/api/3/"));
        assert!(body["body"].is_object());
    }

    #[tokio::test]
    async fn send_maps_409_to_conflict() {
        let error = error_from_response(409, None).await;

        assert!(matches!(error, WriteError::Conflict(_)));
    }

    #[tokio::test]
    async fn send_maps_400_to_permanent_with_jira_error_text() {
        let error = error_from_response(
            400,
            Some(json!({ "errors": { "summary": "Summary is required" } })),
        )
        .await;

        assert!(matches!(
            error,
            WriteError::Permanent(message) if message.contains("Summary is required")
        ));
    }

    #[tokio::test]
    async fn send_maps_503_to_transient() {
        let error = error_from_response(503, None).await;

        assert!(matches!(error, WriteError::Transient(_)));
    }

    #[tokio::test]
    async fn list_transitions_maps_destination_status_name() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/ABC-1/transitions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "transitions": [{
                    "id": "31",
                    "name": "Resolve",
                    "to": { "name": "Done" }
                }]
            })))
            .expect(1)
            .mount(&server)
            .await;
        let client = JiraWriteClient::new(&server.uri(), "user", "token").unwrap();

        let transitions = client.list_transitions("ABC-1").await.unwrap();

        assert_eq!(transitions[0].to_status, "Done");
    }

    #[tokio::test]
    async fn issue_types_falls_back_when_projects_are_empty() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/createmeta"))
            .and(query_param("projectKeys", "ABC"))
            .and(query_param("expand", "projects.issuetypes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "projects": [] })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/createmeta/ABC/issuetypes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "values": [
                    { "name": "Story", "subtask": false },
                    { "name": "Sub-task", "subtask": true }
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;
        let client = JiraWriteClient::new(&server.uri(), "user", "token").unwrap();

        let issue_types = client.issue_types("ABC").await.unwrap();

        assert_eq!(issue_types, vec!["Story"]);
    }

    async fn error_from_response(status: u16, body: Option<Value>) -> WriteError {
        let server = MockServer::start().await;
        let mut response = ResponseTemplate::new(status);
        if let Some(body) = body {
            response = response.set_body_json(body);
        }
        Mock::given(method("GET"))
            .and(path("/rest/api/3/myself"))
            .respond_with(response)
            .expect(1)
            .mount(&server)
            .await;
        let client = JiraWriteClient::new(&server.uri(), "user", "token").unwrap();

        client.myself().await.unwrap_err()
    }
}
