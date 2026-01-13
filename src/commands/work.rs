use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::bug::Status;
use crate::store::Store;

pub fn work(id: &str) -> Result<()> {
    let store = Store::open()?;
    let bug = store.get_bug(id)?;

    // Warn if not approved
    match bug.status() {
        Status::Approved | Status::InProgress => {}
        Status::Draft => {
            eprintln!(
                "{} Bug {} is still in draft status. Consider approving it first.",
                "!".yellow(),
                bug.id().cyan()
            );
        }
        Status::Done => {
            return Err(anyhow!("bug {} is already done", bug.id()));
        }
    }

    let bug_id = bug.id().to_string();
    let workspace_name = format!("ws-{}", bug_id);

    // Check if jj is available
    let jj_check = Command::new("jj").arg("--version").output();
    if jj_check.is_err() {
        return Err(anyhow!("jj is not installed or not in PATH"));
    }

    // Get repo root
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

    // Check if workspace already exists
    let workspace_exists = std::path::Path::new(&workspace_path).exists();

    if !workspace_exists {
        println!(
            "{} Creating jj workspace {}...",
            "→".blue(),
            workspace_name.cyan()
        );

        let status = Command::new("jj")
            .args(["workspace", "add", &workspace_path])
            .status()
            .context("failed to create jj workspace")?;

        if !status.success() {
            return Err(anyhow!("failed to create workspace"));
        }
    } else {
        println!(
            "{} Using existing workspace {}",
            "→".blue(),
            workspace_name.cyan()
        );
    }

    // Change to workspace directory
    std::env::set_current_dir(&workspace_path)
        .context("failed to change to workspace directory")?;

    // Write context file for Claude to read
    let context = serde_json::json!({
        "bug_id": bug.id(),
        "title": bug.title(),
        "status": format!("{}", bug.status()),
        "priority": format!("{}", bug.priority()),
        "body": bug.body,
    });

    let docket_dir = std::path::Path::new(".docket");
    if !docket_dir.exists() {
        std::fs::create_dir_all(docket_dir)?;
    }

    let context_path = docket_dir.join("current.json");
    std::fs::write(&context_path, serde_json::to_string_pretty(&context)?)
        .context("failed to write context file")?;

    println!(
        "{} Starting work on {} - {}",
        "✓".green(),
        bug.id().cyan(),
        bug.title()
    );
    println!("{} Workspace: {}", "→".blue(), workspace_path);
    println!("{} Launching Claude Code...", "→".blue());
    println!();

    // Exec claude with environment variable
    let err = Command::new("claude")
        .env("DOCKET_BUG", &bug_id)
        .exec();

    // If we get here, exec failed
    Err(anyhow!("failed to exec claude: {}", err))
}
