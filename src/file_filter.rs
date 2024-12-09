use anyhow::Result;
use ignore::gitignore::{GitignoreBuilder, Gitignore};
use std::path::Path;

pub struct FileFilter<'a> {
    source_path: &'a Path,
    allowed_extensions: &'a [String],
    ignored_directories: &'a [String],
    gitignore: Option<Gitignore>,
}

impl<'a> FileFilter<'a> {
    pub fn new(source_path: &'a Path, config: &'a crate::Config) -> Result<Self> {
        let gitignore = if config.respect_gitignore {
            let mut builder = GitignoreBuilder::new(source_path);
            let gitignore_path = source_path.join(".gitignore");
            if gitignore_path.exists() {
                builder.add(gitignore_path);
            }
            Some(builder.build()?)
        } else {
            None
        };

        Ok(Self {
            source_path,
            allowed_extensions: &config.allowed_extensions,
            ignored_directories: &config.ignored_directories,
            gitignore,
        })
    }

    pub fn should_process_file(&self, path: &Path) -> Result<bool> {
        if !Self::should_process_path(
            path,
            self.source_path,
            self.allowed_extensions,
            self.ignored_directories,
            self.gitignore.as_ref(),
        )? {
            return Ok(false);
        }

        // additional checks
        if !self.allowed_extensions.is_empty() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if !self.allowed_extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn should_process_path(
        path: &Path,
        source_path: &Path,
        _allowed_extensions: &[String],
        ignored_directories: &[String],
        gitignore: Option<&Gitignore>,
    ) -> Result<bool> {
        // check gitignore if enabled
        if let Some(gitignore) = gitignore {
            let relative_path = path.strip_prefix(source_path)?;
            if gitignore.matched(relative_path, path.is_dir()).is_ignore() {
                return Ok(false);
            }
        }

        // check if ignored dir
        if path.ancestors().any(|ancestor| {
            ancestor
                .file_name()
                .map(|name| {
                    ignored_directories.iter().any(|ignored| {
                        name.to_string_lossy().to_lowercase() == ignored.to_lowercase()
                    })
                })
                .unwrap_or(false)
        }) {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn gitignore(&self) -> Option<&Gitignore> {
        self.gitignore.as_ref()
    }
}
