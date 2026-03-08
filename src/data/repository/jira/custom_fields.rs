use std::collections::BTreeMap;

pub(super) fn extract_custom_fields(issue: &gouqi::Issue) -> Option<String> {
    let mut custom = BTreeMap::new();
    for (key, value) in &issue.fields {
        if key.starts_with("customfield_") {
            custom.insert(key.clone(), value.clone());
        }
    }

    if custom.is_empty() {
        None
    } else {
        serde_json::to_string(&custom).ok()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::extract_custom_fields;

    #[test]
    fn extract_custom_fields_should_only_include_customfield_entries() {
        let issue: gouqi::Issue = serde_json::from_value(json!({
            "self": "https://jira.example/issues/1",
            "key": "TUIJI-1",
            "id": "1",
            "fields": {
                "summary": "ignored",
                "customfield_10002": 8,
                "customfield_10010": { "name": "flag" }
            }
        }))
        .unwrap();

        let custom_fields = extract_custom_fields(&issue).unwrap();

        assert!(custom_fields.contains(r#""customfield_10002":8"#));
        assert!(custom_fields.contains(r#""customfield_10010":{"name":"flag"}"#));
        assert!(!custom_fields.contains("summary"));
    }
}
