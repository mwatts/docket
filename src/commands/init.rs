use anyhow::Result;
use colored::Colorize;

use crate::store::Store;

pub fn init() -> Result<()> {
    let store = Store::init()?;
    println!(
        "{} Initialized docket repository at {}",
        "✓".green(),
        store.root().display()
    );
    Ok(())
}
