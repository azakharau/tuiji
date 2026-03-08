use std::time::{Instant, SystemTime};

use crate::{
    contracts::sync::{SyncJob, SyncJobKind, SyncSource},
    data::SyncLogFilter,
};

pub(super) fn format_filter(filter: SyncLogFilter) -> &'static str {
    match filter {
        SyncLogFilter::All => "All",
        SyncLogFilter::Pull => "Pull",
        SyncLogFilter::Push => "Push",
    }
}

pub(super) fn format_job_kind(kind: SyncJobKind) -> &'static str {
    match kind {
        SyncJobKind::Pull => "Pull",
        SyncJobKind::Push => "Push",
    }
}

pub(super) fn format_job_source(source: SyncSource) -> &'static str {
    match source {
        SyncSource::Manual => "Manual",
        SyncSource::Button => "Button",
        SyncSource::Startup => "Startup",
        SyncSource::Interval => "Interval",
    }
}

pub(super) fn format_next_attempt(next_attempt: Option<Instant>) -> String {
    match next_attempt {
        None => "ready".to_string(),
        Some(instant) => {
            let now = Instant::now();
            if instant <= now {
                "ready".to_string()
            } else {
                let secs = instant.duration_since(now).as_secs();
                format!("in {secs}s")
            }
        }
    }
}

pub(super) fn format_time(value: Option<SystemTime>) -> String {
    let Some(value) = value else {
        return "never".to_string();
    };
    let Ok(datetime) =
        time::OffsetDateTime::from(value).format(&time::format_description::well_known::Rfc3339)
    else {
        return "unknown".to_string();
    };
    datetime
}

pub(super) fn active_job_label(job: Option<&SyncJob>) -> &'static str {
    let Some(job) = job else {
        return "Idle";
    };
    format_job_kind(job.kind)
}
