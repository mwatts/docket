use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::Path;

use crate::bug::Status;
use crate::store::Store;

fn set_status(id: &str, new_status: Status, action: &str) -> Result<()> {
    let store = Store::open()?;
    let mut bug = store.get_bug(id)?;

    let old_status = bug.status().clone();

    // Validate transition
    match (&old_status, &new_status) {
        (Status::Done, _) => {
            return Err(anyhow!(
                "cannot change status of completed bug '{}'",
                bug.id()
            ));
        }
        _ => {}
    }

    bug.set_status(new_status.clone());
    store.save_bug(&bug)?;

    println!(
        "{} {} bug {} ({} -> {})",
        "✓".green(),
        action,
        bug.id().cyan(),
        format!("{}", old_status).dimmed(),
        format!("{}", new_status).green()
    );

    Ok(())
}

pub fn approve(id: &str) -> Result<()> {
    set_status(id, Status::Approved, "Approved")
}

pub fn start(id: &str) -> Result<()> {
    set_status(id, Status::InProgress, "Started")
}

pub fn done(id: &str) -> Result<()> {
    let store = Store::open()?;
    let mut bug = store.get_bug(id)?;

    let old_status = bug.status().clone();

    if matches!(old_status, Status::Done) {
        return Err(anyhow!(
            "cannot change status of completed bug '{}'",
            bug.id()
        ));
    }

    // Try to sync body from current.json if it exists
    if let Some(updated_body) = find_and_read_current_json(bug.id())? {
        bug.body = updated_body;
        println!(
            "{} Synced acceptance criteria from workspace",
            "→".blue()
        );
    }

    bug.set_status(Status::Done);
    store.save_bug(&bug)?;

    println!(
        "{} {} bug {} ({} -> {})",
        "✓".green(),
        "Completed",
        bug.id().cyan(),
        format!("{}", old_status).dimmed(),
        format!("{}", Status::Done).green()
    );

    // Create a fresh jj change if the current one has content
    create_fresh_change_if_needed()?;

    Ok(())
}

/// Check if current jj change is empty and create a new one if needed
fn create_fresh_change_if_needed() -> Result<()> {
    // Check if current change is empty
    let output = std::process::Command::new("jj")
        .args(["log", "-r", "@", "--no-graph", "-T", "if(empty, \"empty\", \"has_changes\")"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if result == "has_changes" {
                // Current change has content, create a fresh one
                let new_output = std::process::Command::new("jj")
                    .args(["new"])
                    .output()?;

                if new_output.status.success() {
                    println!(
                        "{} Created fresh change for next task",
                        "→".blue()
                    );
                } else {
                    // Log the error but don't fail the done command
                    let stderr = String::from_utf8_lossy(&new_output.stderr);
                    eprintln!(
                        "{} Failed to create new change: {}",
                        "!".yellow(),
                        stderr.trim()
                    );
                }
            } else {
                println!(
                    "{} Current change is empty, ready for next task",
                    "→".blue()
                );
            }
        }
        Ok(output) => {
            // jj command failed - might not be in a jj repo
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                // Only warn if there's an actual error message, skip silently if not in jj repo
                eprintln!(
                    "{} Could not check change status: {}",
                    "!".yellow(),
                    stderr.trim()
                );
            }
        }
        Err(_) => {
            // jj not available, silently skip
        }
    }

    Ok(())
}

/// Look for current.json in likely locations and return the body if found
fn find_and_read_current_json(bug_id: &str) -> Result<Option<String>> {
    // First, check if we're in a workspace with current.json
    let local_current = Path::new(".docket/current.json");
    if local_current.exists() {
        if let Some(body) = read_current_json_if_matches(local_current, bug_id)? {
            return Ok(Some(body));
        }
    }

    // Try to find workspace relative to repo root
    if let Ok(output) = std::process::Command::new("jj")
        .args(["workspace", "root"])
        .output()
    {
        if output.status.success() {
            let repo_root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let workspace_current = Path::new(&repo_root)
                .parent()
                .map(|p| p.join(format!("ws-{}", bug_id)).join(".docket/current.json"));

            if let Some(path) = workspace_current {
                if path.exists() {
                    if let Some(body) = read_current_json_if_matches(&path, bug_id)? {
                        return Ok(Some(body));
                    }
                }
            }
        }
    }

    Ok(None)
}

fn read_current_json_if_matches(path: &Path, bug_id: &str) -> Result<Option<String>> {
    let content = std::fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    // Check if this current.json is for the right bug
    if json.get("bug_id").and_then(|v| v.as_str()) == Some(bug_id) {
        if let Some(body) = json.get("body").and_then(|v| v.as_str()) {
            return Ok(Some(body.to_string()));
        }
    }

    Ok(None)
}
