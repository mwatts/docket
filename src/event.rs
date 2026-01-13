use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use uuid::Uuid;

use crate::bug::{Bug, BugMetadata, Priority, Status};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum EventData {
    Created {
        title: String,
        priority: Priority,
        body: String,
    },
    StatusChanged {
        from: Status,
        to: Status,
    },
    Updated {
        title: Option<String>,
        body: Option<String>,
    },
    ChangeLinked {
        change_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub bug_id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(flatten)]
    pub data: EventData,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
}

impl Event {
    pub fn new(bug_id: String, data: EventData) -> Self {
        Event {
            id: Uuid::new_v4().to_string(),
            bug_id,
            timestamp: Utc::now(),
            data,
            actor: whoami::fallible::hostname().ok(),
        }
    }

    pub fn created(bug_id: String, title: String, priority: Priority, body: String) -> Self {
        Self::new(
            bug_id,
            EventData::Created {
                title,
                priority,
                body,
            },
        )
    }

    pub fn status_changed(bug_id: String, from: Status, to: Status) -> Self {
        Self::new(bug_id, EventData::StatusChanged { from, to })
    }

    pub fn updated(bug_id: String, title: Option<String>, body: Option<String>) -> Self {
        Self::new(bug_id, EventData::Updated { title, body })
    }

    pub fn change_linked(bug_id: String, change_id: String) -> Self {
        Self::new(bug_id, EventData::ChangeLinked { change_id })
    }
}

/// Append an event to a bug's JSONL file
pub fn append_event(path: &Path, event: &Event) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open {}", path.display()))?;

    let json = serde_json::to_string(event)?;
    writeln!(file, "{}", json)?;
    Ok(())
}

/// Read all events from a bug's JSONL file
pub fn read_events(path: &Path) -> Result<Vec<Event>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("failed to read line {}", line_num + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let event: Event = serde_json::from_str(&line)
            .with_context(|| format!("failed to parse event on line {}", line_num + 1))?;
        events.push(event);
    }

    // Sort by timestamp to ensure correct replay order
    events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(events)
}

/// Derive current bug state by replaying events
pub fn derive_bug(events: &[Event]) -> Result<Bug> {
    if events.is_empty() {
        return Err(anyhow::anyhow!("no events to derive bug from"));
    }

    // Find the Created event to get initial state
    let created = events
        .iter()
        .find(|e| matches!(e.data, EventData::Created { .. }))
        .ok_or_else(|| anyhow::anyhow!("no Created event found"))?;

    let (initial_title, initial_priority, initial_body) = match &created.data {
        EventData::Created {
            title,
            priority,
            body,
        } => (title.clone(), priority.clone(), body.clone()),
        _ => unreachable!(),
    };

    let mut bug = Bug {
        metadata: BugMetadata {
            id: created.bug_id.clone(),
            title: initial_title,
            status: Status::Draft,
            priority: initial_priority,
            created: created.timestamp,
            changes: Vec::new(),
        },
        body: initial_body,
    };

    // Replay all events in order
    for event in events {
        match &event.data {
            EventData::Created { .. } => {
                // Already handled above
            }
            EventData::StatusChanged { to, .. } => {
                bug.metadata.status = to.clone();
            }
            EventData::Updated { title, body } => {
                if let Some(t) = title {
                    bug.metadata.title = t.clone();
                }
                if let Some(b) = body {
                    bug.body = b.clone();
                }
            }
            EventData::ChangeLinked { change_id } => {
                if !bug.metadata.changes.contains(change_id) {
                    bug.metadata.changes.push(change_id.clone());
                }
            }
        }
    }

    Ok(bug)
}
