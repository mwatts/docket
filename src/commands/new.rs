use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select};

use crate::bug::{Bug, Priority};
use crate::store::Store;

pub fn new(title: Option<String>, priority_str: &str, interactive: bool) -> Result<()> {
    let store = Store::open()?;

    // Get title interactively if not provided
    let title = match title {
        Some(t) => t,
        None => Input::new()
            .with_prompt("Bug title")
            .interact_text()?,
    };

    // Parse priority - prompt interactively only if in interactive mode and using default
    let priority: Priority = if interactive && priority_str == "medium" {
        let options = vec!["low", "medium", "high"];
        let selection = Select::new()
            .with_prompt("Priority")
            .items(&options)
            .default(1)
            .interact()?;
        options[selection].parse().unwrap_or(Priority::Medium)
    } else {
        priority_str.parse().unwrap_or_else(|_| {
            eprintln!(
                "{} Invalid priority '{}', using 'medium'",
                "!".yellow(),
                priority_str
            );
            Priority::Medium
        })
    };

    // Generate unique ID
    let id = store.generate_id()?;

    // Create bug
    let bug = Bug::new(id.clone(), title.clone(), priority);
    store.save_bug(&bug)?;

    println!("{} Created bug {} - {}", "✓".green(), id.cyan(), title);
    println!(
        "  Edit with: {} {}",
        "docket show".dimmed(),
        id.dimmed()
    );

    Ok(())
}
