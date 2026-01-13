use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::store::Store;

pub fn cleanup(id: &str) -> Result<()> {
    let store = Store::open()?;
    let bug = store.get_bug(id)?;

    let bug_id = bug.id().to_string();
    let workspace_name = format!("ws-{}", bug_id);

    // Check if jj is available
    let jj_check = Command::new("jj").arg("--version").output();
    if jj_check.is_err() {
        return Err(anyhow!("jj is not installed or not in PATH"));
    }

    // Get repo root to find workspace path
    let repo_root = Command::new("jj")
        .args(["workspace", "root"])
        .output()
        .context("failed to get jj workspace root")?;

    if !repo_root.status.success() {
        return Err(anyhow!("not in a jj repository"));
    }

    let repo_root = String::from_utf8_lossy(&repo_root.stdout)
        .trim()
        .to_string();

    let workspace_path = format!("{}/../{}", repo_root, workspace_name);
    let workspace_dir = Path::new(&workspace_path);

    // Check if workspace directory exists
    if !workspace_dir.exists() {
        println!(
            "{} Workspace {} does not exist, nothing to clean up",
            "→".blue(),
            workspace_name.cyan()
        );
        return Ok(());
    }

    // Check for uncommitted changes in the workspace
    let status_output = Command::new("jj")
        .args(["status"])
        .current_dir(&workspace_path)
        .output()
        .context("failed to check workspace status")?;

    if status_output.status.success() {
        let status_text = String::from_utf8_lossy(&status_output.stdout);
        // jj status shows "Working copy changes:" when there are uncommitted changes
        // An empty working copy shows "The working copy is clean"
        if status_text.contains("Working copy changes:") {
            eprintln!(
                "{} Workspace {} has uncommitted changes!",
                "!".yellow(),
                workspace_name.cyan()
            );
            eprintln!("{}", status_text.dimmed());
            return Err(anyhow!(
                "refusing to clean up workspace with uncommitted changes. \
                 Commit or discard changes first."
            ));
        }
    }

    // Run jj workspace forget from the main repo
    println!(
        "{} Forgetting jj workspace {}...",
        "→".blue(),
        workspace_name.cyan()
    );

    let forget_status = Command::new("jj")
        .args(["workspace", "forget", &workspace_name])
        .current_dir(&repo_root)
        .status()
        .context("failed to forget jj workspace")?;

    if !forget_status.success() {
        // Workspace might not exist in jj (already forgotten), continue with directory removal
        println!(
            "{} Workspace {} not found in jj (may already be forgotten)",
            "!".yellow(),
            workspace_name.cyan()
        );
    }

    // Remove the workspace directory
    println!(
        "{} Removing workspace directory {}...",
        "→".blue(),
        workspace_path.cyan()
    );

    fs::remove_dir_all(&workspace_path)
        .with_context(|| format!("failed to remove workspace directory {}", workspace_path))?;

    println!(
        "{} Cleaned up workspace for bug {}",
        "✓".green(),
        bug_id.cyan()
    );

    Ok(())
}
