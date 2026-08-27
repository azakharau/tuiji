use crate::data::model::IssueComment;

pub(super) fn extract_comments(issue: &gouqi::Issue, issue_key: &str) -> Vec<IssueComment> {
    let Some(value) = issue.fields.get("comment") else {
        return Vec::new();
    };
    let Some(comments) = value
        .get("comments")
        .and_then(|comments| comments.as_array())
    else {
        return Vec::new();
    };

    let mut out = Vec::with_capacity(comments.len());
    for comment in comments {
        let id = comment
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        if id.is_empty() {
            continue;
        }

        let author = comment
            .get("author")
            .and_then(|author| author.get("displayName"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Unknown")
            .to_string();
        let body = comment.get("body").map_or_else(String::new, |body| {
            body.as_str().map(str::to_string).unwrap_or_else(|| {
                serde_json::from_value::<gouqi::AdfDocument>(body.clone())
                    .map(|document| document.to_plain_text())
                    .unwrap_or_else(|_| serde_json::to_string(body).unwrap_or_default())
            })
        });
        let created_at = comment
            .get("created")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string);
        let updated_at = comment
            .get("updated")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string);

        out.push(IssueComment {
            id,
            issue_key: issue_key.to_string(),
            author,
            body,
            created_at,
            updated_at,
            dirty: false,
            conflict: false,
            remote_snapshot: None,
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::extract_comments;

    #[test]
    fn extract_comments_should_skip_entries_without_id_and_convert_adf_bodies() {
        let issue: gouqi::Issue = serde_json::from_value(json!({
            "self": "https://jira.example/issues/1",
            "key": "TUIJI-1",
            "id": "1",
            "fields": {
                "comment": {
                    "comments": [
                        {
                            "id": "c1",
                            "author": { "displayName": "Alice" },
                            "body": "plain text",
                            "created": "2026-01-01T10:00:00.000+0000",
                            "updated": "2026-01-01T11:00:00.000+0000"
                        },
                        {
                            "id": "",
                            "author": { "displayName": "Skip" },
                            "body": "ignored"
                        },
                        {
                            "id": "c2",
                            "body": {
                                "version": 1,
                                "type": "doc",
                                "content": [
                                    {
                                        "type": "paragraph",
                                        "content": [
                                            { "type": "text", "text": "Cloud body" }
                                        ]
                                    }
                                ]
                            }
                        }
                    ]
                }
            }
        }))
        .unwrap();

        let comments = extract_comments(&issue, "TUIJI-1");

        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].id, "c1");
        assert_eq!(comments[0].author, "Alice");
        assert_eq!(comments[0].body, "plain text");
        assert_eq!(comments[1].id, "c2");
        assert_eq!(comments[1].author, "Unknown");
        assert_eq!(comments[1].body, "Cloud body");
    }
}
