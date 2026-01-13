use anyhow::{anyhow, Result};
use colored::Colorize;

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
    set_status(id, Status::Done, "Completed")
}
