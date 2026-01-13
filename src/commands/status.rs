use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::Path;

use crate::bug::Status;
use crate::event::Event;
use crate::store::Store;

fn set_status(id: &str, new_status: Status, action: &str) -> Result<()> {
    let store = Store::open()?;
    let bug = store.get_bug(id)?;

    let old_status = bug.status().clone();
    let bug_id = bug.id().to_string();

    // Validate transition
    if matches!(old_status, Status::Done) {
        return Err(anyhow!(
            "cannot change status of completed bug '{}'",
            bug_id
        ));
    }

    // Emit StatusChanged event
    let event = Event::status_changed(bug_id.clone(), old_status.clone(), new_status.clone());
    store.append_event(&event)?;

    println!(
        "{} {} bug {} ({} -> {})",
        "✓".green(),
        action,
        bug_id.cyan(),
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
    let bug = store.get_bug(id)?;

    let old_status = bug.status().clone();
    let bug_id = bug.id().to_string();

    if matches!(old_status, Status::Done) {
        return Err(anyhow!(
            "cannot change status of completed bug '{}'",
            bug_id
        ));
    }

    // Check if we're running from a workspace
    let workspace_ctx = detect_workspace(&bug_id)?;
    let in_workspace = workspace_ctx.is_some();

    if let Some(ref ctx) = workspace_ctx {
        // Running from workspace - do the full workflow
        println!(
            "{} Running from workspace for bug {}",
            "→".blue(),
            bug_id.cyan()
        );

        // 1. Snapshot any uncommitted changes
        jj_snapshot()?;

        // 2. Generate and set commit message
        let commit_message = format!("Implement {} ({})", ctx.title, ctx.bug_id);
        jj_describe(&commit_message)?;

        // 3. Link the change to the bug
        if let Some(change_id) = get_current_change_id()? {
            let link_event = Event::change_linked(bug_id.clone(), change_id.clone());
            store.append_event(&link_event)?;
            println!(
                "{} Linked change {} to bug {}",
                "✓".green(),
                change_id.cyan(),
                bug_id.cyan()
            );
        }

        // 4. Sync updated body from workspace
        let update_event = Event::updated(bug_id.clone(), None, Some(ctx.body.clone()));
        store.append_event(&update_event)?;
        println!(
            "{} Synced acceptance criteria from workspace",
            "→".blue()
        );
    } else {
        // Not in workspace - try to sync body from current.json if it exists
        if let Some(updated_body) = find_and_read_current_json(&bug_id)? {
            let update_event = Event::updated(bug_id.clone(), None, Some(updated_body));
            store.append_event(&update_event)?;
            println!(
                "{} Synced acceptance criteria from workspace",
                "→".blue()
            );
        }
    }

    // Emit StatusChanged event
    let status_event = Event::status_changed(bug_id.clone(), old_status.clone(), Status::Done);
    store.append_event(&status_event)?;

    println!(
        "{} {} bug {} ({} -> {})",
        "✓".green(),
        "Completed",
        bug_id.cyan(),
        format!("{}", old_status).dimmed(),
        format!("{}", Status::Done).green()
    );

    // Only create a fresh jj change if NOT in a workspace
    // (workspace changes stay as-is for review/submission)
    if !in_workspace {
        create_fresh_change_if_needed()?;
    }

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

/// Workspace context returned when running from a ws-* directory
struct WorkspaceContext {
    bug_id: String,
    title: String,
    body: String,
}

/// Check if we're running from a workspace directory
/// Returns workspace context if found, None otherwise
fn detect_workspace(bug_id: &str) -> Result<Option<WorkspaceContext>> {
    // First, check if we're in a workspace with current.json
    let local_current = Path::new(".docket/current.json");
    if local_current.exists() {
        if let Some(ctx) = read_workspace_context(local_current, bug_id)? {
            return Ok(Some(ctx));
        }
    }

    // Also check current directory name for ws-* pattern
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(dir_name) = cwd.file_name().and_then(|n| n.to_str()) {
            if dir_name.starts_with("ws-") {
                // We're in a workspace directory, check for current.json
                let current_json = cwd.join(".docket/current.json");
                if current_json.exists() {
                    if let Some(ctx) = read_workspace_context(&current_json, bug_id)? {
                        return Ok(Some(ctx));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Read workspace context from current.json if it matches the bug_id
fn read_workspace_context(path: &Path, bug_id: &str) -> Result<Option<WorkspaceContext>> {
    let content = std::fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    // Check if this current.json is for the right bug
    if json.get("bug_id").and_then(|v| v.as_str()) == Some(bug_id) {
        let title = json.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let body = json.get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        return Ok(Some(WorkspaceContext {
            bug_id: bug_id.to_string(),
            title,
            body,
        }));
    }

    Ok(None)
}

/// Trigger jj to snapshot any uncommitted changes
/// jj auto-snapshots on most commands, so we run `jj log -n0` (no output, just snapshot)
fn jj_snapshot() -> Result<()> {
    let output = std::process::Command::new("jj")
        .args(["log", "-n0"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("{} Snapshotted working copy changes", "→".blue());
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("jj log failed: {}", stderr.trim()))
        }
        Err(e) => Err(anyhow!("failed to run jj log: {}", e)),
    }
}

/// Set the commit description using jj describe
fn jj_describe(message: &str) -> Result<()> {
    let output = std::process::Command::new("jj")
        .args(["describe", "-m", message])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("{} Set commit message", "→".blue());
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("jj describe failed: {}", stderr.trim()))
        }
        Err(e) => Err(anyhow!("failed to run jj describe: {}", e)),
    }
}

/// Get the current change ID
fn get_current_change_id() -> Result<Option<String>> {
    let output = std::process::Command::new("jj")
        .args(["log", "-r", "@", "--no-graph", "-T", "change_id"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let change_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if change_id.is_empty() {
                Ok(None)
            } else {
                Ok(Some(change_id))
            }
        }
        _ => Ok(None),
    }
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
