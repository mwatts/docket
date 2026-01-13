use anyhow::Result;
use colored::Colorize;

use crate::store::Store;

pub fn link(bug_id: &str, change_id: &str) -> Result<()> {
    let store = Store::open()?;
    let mut bug = store.get_bug(bug_id)?;

    bug.add_change(change_id.to_string());
    store.save_bug(&bug)?;

    println!(
        "{} Linked change {} to bug {}",
        "✓".green(),
        change_id.cyan(),
        bug.id().cyan()
    );

    Ok(())
}
