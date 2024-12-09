use anyhow::Result;
use clap::Parser;
use ignore::gitignore::Gitignore;
use std::collections::BTreeMap;
use std::fs;
use std::env;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::ZipWriter;

mod file_filter;
use file_filter::FileFilter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// default to pwd
    #[arg(default_value = ".")]
    source: String,

    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long)]
    subfolder: Option<String>,

    #[arg(short, long)]
    zip: bool,

    #[arg(short, long)]
    tree: bool,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, serde::Deserialize)]
struct Config {
    #[serde(default)]
    allowed_extensions: Vec<String>,
    #[serde(default = "default_delimiter")]
    delimiter: String,
    #[serde(default = "default_subfolder")]
    subfolder: String,
    #[serde(default)]
    zip: bool,
    #[serde(default = "default_ignored_directories")]
    ignored_directories: Vec<String>,
    #[serde(default = "default_respect_gitignore")]
    respect_gitignore: bool,
}

fn default_delimiter() -> String { "^".to_string() }
fn default_subfolder() -> String { "context".to_string() }
fn default_respect_gitignore() -> bool { true }

fn default_ignored_directories() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        "venv".to_string(),
        ".venv".to_string(),
        "env".to_string(),
        ".env".to_string(),
        "target".to_string(),
        "build".to_string(),
        "dist".to_string(),
        "__pycache__".to_string(),
        ".git".to_string(),
        ".idea".to_string(),
        ".vs".to_string(),
        ".vscode".to_string(),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            allowed_extensions: vec![],
            delimiter: default_delimiter(),
            subfolder: default_subfolder(),
            zip: false,
            ignored_directories: default_ignored_directories(),
            respect_gitignore: default_respect_gitignore(),
        }
    }
}

struct FileProcessor<'a> {
    source_path: &'a Path,
    output_dir: PathBuf,
    config: &'a Config,
    filter: FileFilter<'a>,
    verbose: bool,
}

impl<'a> FileProcessor<'a> {
    fn new(source: &'a str, config: &'a Config, verbose: bool) -> Result<Self> {
        let source_path = Path::new(source);
        let output_dir = source_path.join(&config.subfolder);
        let filter = FileFilter::new(source_path, config)?;

        Ok(Self {
            source_path,
            output_dir,
            config,
            filter,
            verbose,
        })
    }

    fn prepare_output_directory(&self) -> Result<()> {
        if self.output_dir.exists() {
            if self.verbose {
                eprintln!("Cleaning existing output directory");
            }
            fs::remove_dir_all(&self.output_dir)?;
        }
        fs::create_dir_all(&self.output_dir)?;
        Ok(())
    }

    fn collect_files(&self) -> Result<Vec<(PathBuf, String)>> {
        let mut files_to_process = Vec::new();

        for entry in WalkDir::new(self.source_path).follow_links(false) {
            let entry = entry?;
            let path = entry.path();

            if path.starts_with(&self.output_dir) {
                if self.verbose {
                    eprintln!("Skipping output directory: {}", path.display());
                }
                continue;
            }

            if !self.filter.should_process_file(path)? {
                continue;
            }

            if !entry.file_type().is_file() {
                continue;
            }

            let new_name = self.generate_new_filename(path)?;
            if self.verbose {
                eprintln!("Adding file: {} -> {}", path.display(), new_name);
            }

            files_to_process.push((path.to_path_buf(), new_name));
        }

        if self.verbose {
            eprintln!("Total files to process: {}", files_to_process.len());
        }

        Ok(files_to_process)
    }

    fn generate_new_filename(&self, path: &Path) -> Result<String> {
        let relative_path = path.strip_prefix(self.source_path)?;
        Ok(relative_path
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join(&self.config.delimiter))
    }

    fn process_files(&self, files: Vec<(PathBuf, String)>) -> Result<()> {
        if self.config.zip {
            self.create_zip_archive(files)?;
        } else {
            self.copy_files(files)?;
        }
        Ok(())
    }

    fn create_zip_archive(&self, files: Vec<(PathBuf, String)>) -> Result<()> {
        let zip_path = self.output_dir.join(format!("{}.zip", self.config.subfolder));
        let zip_file = fs::File::create(&zip_path)?;
        let mut zip = ZipWriter::new(BufWriter::new(zip_file));

        for (path, new_name) in files {
            zip.start_file(&new_name, zip::write::FileOptions::default())?;
            let mut file = fs::File::open(path)?;
            std::io::copy(&mut file, &mut zip)?;
        }

        Ok(())
    }

    fn copy_files(&self, files: Vec<(PathBuf, String)>) -> Result<()> {
        for (path, new_name) in files {
            let new_path = self.output_dir.join(&new_name);
            fs::copy(path, &new_path)?;
        }
        Ok(())
    }

    fn generate_tree(&self) -> Result<()> {
        let source_tree = generate_tree_string(
            self.source_path,
            "",
            true,
            &mut BTreeMap::new(),
            &self.config.allowed_extensions,
            &self.config.ignored_directories,
            self.filter.gitignore(),
            self.source_path,
        )?;

        let tree_content = format!("{}\n{}", self.source_path.display(), source_tree);
        println!("{}", tree_content);
        fs::write(self.output_dir.join("filetree.txt"), tree_content)?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut config = load_config(&args.config)?;
    
    if args.verbose {
        eprintln!("Loaded config: {:#?}", config);
    }

    if let Some(subfolder) = args.subfolder {
        config.subfolder = subfolder;
    }

    if args.zip {
        config.zip = true;
    }

    if args.verbose {
        eprintln!("Final config after CLI overrides: {:#?}", config);
    }

    let processor = FileProcessor::new(&args.source, &config, args.verbose)?;
    processor.prepare_output_directory()?;
    
    if args.tree {
        processor.generate_tree()?;
    }

    let files = processor.collect_files()?;
    processor.process_files(files)?;

    Ok(())
}

fn load_config(config_path: &Option<String>) -> Result<Config> {
    if let Some(path) = config_path {
        return load_config_from_path(path);
    }

    // Try home directory configs with both extensions
    if let Ok(home) = env::var("HOME") {
        let home_path = PathBuf::from(home);
        for ext in &[".yml", ".yaml"] {
            let config_path = home_path.join(format!(".lmprep{}", ext));
            if let Ok(config) = load_config_from_path(&config_path.to_string_lossy()) {
                return Ok(config);
            }
        }
    }

    if let Ok(home) = env::var("USERPROFILE") {
        let home_path = PathBuf::from(home);
        for ext in &[".yml", ".yaml"] {
            let config_path = home_path.join(format!(".lmprep{}", ext));
            if let Ok(config) = load_config_from_path(&config_path.to_string_lossy()) {
                return Ok(config);
            }
        }
    }

    // Try local config with both extensions
    for ext in &[".yml", ".yaml"] {
        let config_path = format!(".lmprep{}", ext);
        if let Ok(config) = load_config_from_path(&config_path) {
            return Ok(config);
        }
    }

    eprintln!("No config file found, using defaults");
    Ok(Config::default())
}

fn load_config_from_path(path: &str) -> Result<Config> {
    let path_buf = PathBuf::from(path);
    if path_buf.exists() {
        let contents = fs::read_to_string(&path_buf)?;
        match serde_yaml::from_str(&contents) {
            Ok(config) => return Ok(config),
            Err(e) => eprintln!("Warning: Error parsing config file {}: {}. Using defaults.", path, e),
        }
    }
    Ok(Config::default())
}

fn generate_tree_string(
    path: &Path,
    prefix: &str,
    is_last: bool,
    seen_dirs: &mut BTreeMap<PathBuf, bool>,
    allowed_extensions: &[String],
    ignored_directories: &[String],
    gitignore: Option<&Gitignore>,
    source_path: &Path,
) -> Result<String> {
    let mut output = String::new();
    let entry = path.to_path_buf();
    
    if !FileFilter::should_process_path(
        path,
        source_path,
        allowed_extensions,
        ignored_directories,
        gitignore,
    )? {
        return Ok(String::new());
    }
    
    if path.is_dir() {
        seen_dirs.insert(entry.clone(), true);
        
        let mut children: Vec<_> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .collect();
        
        children.sort_by_key(|a| {
            let p = a.path();
            (p.is_file(), p.to_string_lossy().to_string())
        });
        
        let valid_children: Vec<_> = children.into_iter()
            .filter(|child| {
                let child_path = child.path();
                FileFilter::should_process_path(
                    &child_path,
                    source_path,
                    allowed_extensions,
                    ignored_directories,
                    gitignore,
                ).unwrap_or(false)
            })
            .collect();

        if !valid_children.is_empty() {
            if let Some(name) = entry.file_name().and_then(|n| n.to_str()) {
                output.push_str(&format!("{}{}\n", prefix, name));
            }

            for (i, entry) in valid_children.iter().enumerate() {
                let is_last_entry = i == valid_children.len() - 1;
                let new_prefix = format!("{}",
                    if is_last { "    " } else { "│   " }
                );
                
                let child_output = generate_tree_string(
                    &entry.path(),
                    &format!("{}{}",
                        new_prefix,
                        if is_last_entry { "└── " } else { "├── " }
                    ),
                    is_last_entry,
                    seen_dirs,
                    allowed_extensions,
                    ignored_directories,
                    gitignore,
                    source_path,
                )?;
                
                if !child_output.is_empty() {
                    output.push_str(&child_output);
                }
            }
        }
    } else if let Some(name) = entry.file_name().and_then(|n| n.to_str()) {
        output.push_str(&format!("{}{}\n", prefix, name));
    }
    
    Ok(output)
}