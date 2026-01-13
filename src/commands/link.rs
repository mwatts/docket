use anyhow::Result;
use colored::Colorize;

use crate::event::Event;
use crate::store::Store;

pub fn link(bug_id: &str, change_id: &str) -> Result<()> {
    let store = Store::open()?;

    // Resolve prefix to full ID
    let full_id = store.resolve_id(bug_id)?;

    // Emit ChangeLinked event
    let event = Event::change_linked(full_id.clone(), change_id.to_string());
    store.append_event(&event)?;

    println!(
        "{} Linked change {} to bug {}",
        "✓".green(),
        change_id.cyan(),
        full_id.cyan()
    );

    Ok(())
}
