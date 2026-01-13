use anyhow::{anyhow, Context, Result};
use rand::Rng;
use std::fs;
use std::path::{Path, PathBuf};

use crate::bug::Bug;

const DOCKET_DIR: &str = ".docket";
const BUGS_DIR: &str = "bugs";
const ID_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
const ID_LENGTH: usize = 4;

pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Find .docket/ directory by walking up from current directory
    pub fn open() -> Result<Self> {
        let current = std::env::current_dir()?;
        Self::open_from(&current)
    }

    /// Find .docket/ directory by walking up from specified directory
    pub fn open_from(start: &Path) -> Result<Self> {
        let mut current = start.to_path_buf();

        loop {
            let docket_path = current.join(DOCKET_DIR);
            if docket_path.is_dir() {
                return Ok(Store { root: docket_path });
            }

            if !current.pop() {
                return Err(anyhow!(
                    "not a docket repository (or any parent): .docket directory not found"
                ));
            }
        }
    }

    /// Initialize a new .docket/ directory in the current location
    pub fn init() -> Result<Self> {
        let current = std::env::current_dir()?;
        Self::init_at(&current)
    }

    /// Initialize a new .docket/ directory at specified location
    pub fn init_at(path: &Path) -> Result<Self> {
        let root = path.join(DOCKET_DIR);

        if root.exists() {
            return Err(anyhow!("docket already initialized at {}", root.display()));
        }

        let bugs_dir = root.join(BUGS_DIR);
        fs::create_dir_all(&bugs_dir)
            .with_context(|| format!("failed to create {}", bugs_dir.display()))?;

        // Create .gitignore for cache directory
        let gitignore_path = root.join(".gitignore");
        fs::write(&gitignore_path, ".cache/\n")
            .with_context(|| format!("failed to create {}", gitignore_path.display()))?;

        Ok(Store { root })
    }

    /// Path to the bugs directory
    fn bugs_dir(&self) -> PathBuf {
        self.root.join(BUGS_DIR)
    }

    /// Path to a specific bug file
    fn bug_path(&self, id: &str) -> PathBuf {
        self.bugs_dir().join(format!("{}.md", id))
    }

    /// List all bugs
    pub fn list_bugs(&self) -> Result<Vec<Bug>> {
        let pattern = self.bugs_dir().join("*.md");
        let pattern_str = pattern
            .to_str()
            .ok_or_else(|| anyhow!("invalid path encoding"))?;

        let mut bugs = Vec::new();

        for entry in glob::glob(pattern_str)? {
            let path = entry?;
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;

            match Bug::parse(&content) {
                Ok(bug) => bugs.push(bug),
                Err(e) => {
                    eprintln!("warning: failed to parse {}: {}", path.display(), e);
                }
            }
        }

        // Sort by created date, newest first
        bugs.sort_by(|a, b| b.metadata.created.cmp(&a.metadata.created));

        Ok(bugs)
    }

    /// Get a specific bug by ID (supports prefix matching)
    pub fn get_bug(&self, id: &str) -> Result<Bug> {
        // First try exact match
        let exact_path = self.bug_path(id);
        if exact_path.exists() {
            let content = fs::read_to_string(&exact_path)?;
            return Bug::parse(&content);
        }

        // Try prefix match
        let pattern = self.bugs_dir().join(format!("{}*.md", id));
        let pattern_str = pattern
            .to_str()
            .ok_or_else(|| anyhow!("invalid path encoding"))?;

        let matches: Vec<_> = glob::glob(pattern_str)?.collect::<Result<Vec<_>, _>>()?;

        match matches.len() {
            0 => Err(anyhow!("bug not found: {}", id)),
            1 => {
                let content = fs::read_to_string(&matches[0])?;
                Bug::parse(&content)
            }
            _ => {
                let ids: Vec<_> = matches
                    .iter()
                    .filter_map(|p| p.file_stem())
                    .filter_map(|s| s.to_str())
                    .collect();
                Err(anyhow!(
                    "ambiguous bug ID '{}', matches: {}",
                    id,
                    ids.join(", ")
                ))
            }
        }
    }

    /// Save a bug to disk
    pub fn save_bug(&self, bug: &Bug) -> Result<()> {
        let path = self.bug_path(bug.id());
        let content = bug.to_string()?;
        fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    /// Generate a unique bug ID
    pub fn generate_id(&self) -> Result<String> {
        let mut rng = rand::thread_rng();

        // Try up to 100 times to generate a unique ID
        for _ in 0..100 {
            let id: String = (0..ID_LENGTH)
                .map(|_| {
                    let idx = rng.gen_range(0..ID_CHARS.len());
                    ID_CHARS[idx] as char
                })
                .collect();

            let path = self.bug_path(&id);
            if !path.exists() {
                return Ok(id);
            }
        }

        Err(anyhow!("failed to generate unique ID after 100 attempts"))
    }

    /// Get the root .docket directory path
    pub fn root(&self) -> &Path {
        &self.root
    }
}
