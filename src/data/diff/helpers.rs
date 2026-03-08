use super::snapshot::DiffEntry;

pub(super) fn push_if_diff(
    field: &'static str,
    local: &str,
    remote: &str,
    diffs: &mut Vec<DiffEntry>,
) {
    if local != remote {
        diffs.push(DiffEntry {
            field,
            local: local.to_string(),
            remote: remote.to_string(),
        });
    }
}

pub(super) fn push_if_diff_opt(
    field: &'static str,
    local: Option<&str>,
    remote: Option<&str>,
    diffs: &mut Vec<DiffEntry>,
) {
    push_if_diff(
        field,
        local.unwrap_or_default(),
        remote.unwrap_or_default(),
        diffs,
    );
}

pub(super) fn push_if_diff_opt_f64(
    field: &'static str,
    local: Option<f64>,
    remote: Option<f64>,
    diffs: &mut Vec<DiffEntry>,
) {
    let local = local.map(|v| v.to_string()).unwrap_or_default();
    let remote = remote.map(|v| v.to_string()).unwrap_or_default();
    push_if_diff(field, local.as_str(), remote.as_str(), diffs);
}

pub(super) fn push_if_diff_opt_i64(
    field: &'static str,
    local: Option<i64>,
    remote: Option<i64>,
    diffs: &mut Vec<DiffEntry>,
) {
    let local = local.map(|v| v.to_string()).unwrap_or_default();
    let remote = remote.map(|v| v.to_string()).unwrap_or_default();
    push_if_diff(field, local.as_str(), remote.as_str(), diffs);
}
