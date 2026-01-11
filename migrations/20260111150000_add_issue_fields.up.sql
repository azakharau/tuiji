-- Add new columns to issues table

-- Main content
ALTER TABLE issues ADD COLUMN description TEXT;

-- Authorship
ALTER TABLE issues ADD COLUMN reporter TEXT;
ALTER TABLE issues ADD COLUMN creator TEXT;

-- Timestamps
ALTER TABLE issues ADD COLUMN created_at INTEGER;
ALTER TABLE issues ADD COLUMN resolution_date INTEGER;

-- Status and completion
ALTER TABLE issues ADD COLUMN resolution TEXT;

-- Organization (JSON arrays)
ALTER TABLE issues ADD COLUMN labels TEXT;
ALTER TABLE issues ADD COLUMN fix_versions TEXT;
ALTER TABLE issues ADD COLUMN parent_key TEXT;

-- Environment
ALTER TABLE issues ADD COLUMN environment TEXT;

-- Time tracking
ALTER TABLE issues ADD COLUMN time_estimate TEXT;
ALTER TABLE issues ADD COLUMN time_spent TEXT;
ALTER TABLE issues ADD COLUMN time_remaining TEXT;

-- Custom fields (JSON blob)
ALTER TABLE issues ADD COLUMN custom_fields TEXT;

-- Add same columns to issue_history for tracking changes
ALTER TABLE issue_history ADD COLUMN description TEXT;
ALTER TABLE issue_history ADD COLUMN reporter TEXT;
ALTER TABLE issue_history ADD COLUMN creator TEXT;
ALTER TABLE issue_history ADD COLUMN created_at INTEGER;
ALTER TABLE issue_history ADD COLUMN resolution_date INTEGER;
ALTER TABLE issue_history ADD COLUMN resolution TEXT;
ALTER TABLE issue_history ADD COLUMN labels TEXT;
ALTER TABLE issue_history ADD COLUMN fix_versions TEXT;
ALTER TABLE issue_history ADD COLUMN parent_key TEXT;
ALTER TABLE issue_history ADD COLUMN environment TEXT;
ALTER TABLE issue_history ADD COLUMN time_estimate TEXT;
ALTER TABLE issue_history ADD COLUMN time_spent TEXT;
ALTER TABLE issue_history ADD COLUMN time_remaining TEXT;
ALTER TABLE issue_history ADD COLUMN custom_fields TEXT;

-- Add indexes for search optimization
CREATE INDEX idx_issues_labels ON issues(profile_id, labels) WHERE labels IS NOT NULL;
CREATE INDEX idx_issues_resolution ON issues(profile_id, resolution) WHERE resolution IS NOT NULL;
CREATE INDEX idx_issues_parent_key ON issues(profile_id, parent_key) WHERE parent_key IS NOT NULL;
CREATE INDEX idx_issues_created_at ON issues(profile_id, created_at);
CREATE INDEX idx_issues_resolution_date ON issues(profile_id, resolution_date) WHERE resolution_date IS NOT NULL;
