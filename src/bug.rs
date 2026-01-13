use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Draft,
    Approved,
    InProgress,
    Done,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Draft => write!(f, "draft"),
            Status::Approved => write!(f, "approved"),
            Status::InProgress => write!(f, "in-progress"),
            Status::Done => write!(f, "done"),
        }
    }
}

impl FromStr for Status {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(Status::Draft),
            "approved" => Ok(Status::Approved),
            "in-progress" | "in_progress" | "inprogress" => Ok(Status::InProgress),
            "done" => Ok(Status::Done),
            _ => Err(anyhow!("unknown status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Low => write!(f, "low"),
            Priority::Medium => write!(f, "medium"),
            Priority::High => write!(f, "high"),
        }
    }
}

impl FromStr for Priority {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" | "med" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            _ => Err(anyhow!("unknown priority: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugMetadata {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub priority: Priority,
    pub created: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Bug {
    pub metadata: BugMetadata,
    pub body: String,
}

impl Bug {
    pub fn new(id: String, title: String, priority: Priority) -> Self {
        let metadata = BugMetadata {
            id,
            title,
            status: Status::Draft,
            priority,
            created: Utc::now(),
            changes: Vec::new(),
        };

        let body = r#"
## Goal

<!-- One-sentence description of success -->

## Acceptance Criteria

- [ ] First criterion

## Context

<!-- Background information, constraints, relevant details -->

## Log

<!-- Notes added during implementation -->
"#
        .trim_start()
        .to_string();

        Bug { metadata, body }
    }

    pub fn parse(content: &str) -> Result<Self> {
        let parts: Vec<&str> = content.splitn(3, "---").collect();

        if parts.len() < 3 {
            return Err(anyhow!(
                "invalid bug file format: expected YAML frontmatter between --- delimiters"
            ));
        }

        let yaml_content = parts[1].trim();
        let body = parts[2].trim().to_string();

        let metadata: BugMetadata = serde_yaml::from_str(yaml_content)?;

        Ok(Bug { metadata, body })
    }

    pub fn to_string(&self) -> Result<String> {
        let yaml = serde_yaml::to_string(&self.metadata)?;
        Ok(format!("---\n{}---\n\n{}\n", yaml, self.body))
    }

    pub fn id(&self) -> &str {
        &self.metadata.id
    }

    pub fn title(&self) -> &str {
        &self.metadata.title
    }

    pub fn status(&self) -> &Status {
        &self.metadata.status
    }

    pub fn priority(&self) -> &Priority {
        &self.metadata.priority
    }

    pub fn set_status(&mut self, status: Status) {
        self.metadata.status = status;
    }

    pub fn changes(&self) -> &[String] {
        &self.metadata.changes
    }

    pub fn add_change(&mut self, change_id: String) {
        if !self.metadata.changes.contains(&change_id) {
            self.metadata.changes.push(change_id);
        }
    }
}
