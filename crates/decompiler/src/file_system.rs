use std::fs;
use std::path::PathBuf;

use indexmap::IndexMap;

/// Receives generated goboscript text files.
pub trait FS {
    /// Writes one generated text file at a path relative to the goboscript project root.
    fn write_text_file(&mut self, path: &str, text: &str) -> std::io::Result<()>;
}

/// Writes generated goboscript text files to a filesystem directory.
#[derive(Debug)]
pub struct DiskFS {
    root: PathBuf,
}

impl DiskFS {
    /// Creates a filesystem writer rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl FS for DiskFS {
    fn write_text_file(&mut self, path: &str, text: &str) -> std::io::Result<()> {
        let path = self.root.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, text)
    }
}

/// Collects generated goboscript text files in memory.
#[derive(Debug, Default)]
pub struct InMemoryFS {
    files: IndexMap<String, String>,
}

impl InMemoryFS {
    /// Creates an empty in-memory file writer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns collected files keyed by path relative to the goboscript project root.
    pub fn into_files(self) -> IndexMap<String, String> {
        self.files
    }
}

impl FS for InMemoryFS {
    fn write_text_file(&mut self, path: &str, text: &str) -> std::io::Result<()> {
        self.files.insert(path.to_string(), text.to_string());
        Ok(())
    }
}
