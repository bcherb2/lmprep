use anyhow::Result;
use clap::Parser;
use ignore::gitignore::Gitignore;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::ZipWriter;

mod file_filter;
use file_filter::FileFilter;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "A tool for preparing your codebase for use with LLMs")]
struct Args {
    #[arg(default_value = ".")]
    source: String,

    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long)]
    subfolder: Option<String>,

    #[arg(short = 'z', long)]
    zip: bool,

    #[arg(short, long)]
    tree: bool,

    #[arg(short, long)]
    verbose: bool,

    #[arg(long)]
    init_config: bool,
}

#[derive(Debug, serde::Deserialize, Clone)]
struct Config {
    #[serde(default)]
    allowed_extensions: Vec<String>,
    #[serde(default = "default_delimiter")]
    delimiter: String,
    #[serde(default = "default_subfolder")]
    subfolder: String,
    #[serde(default)]
    zip: bool,
    #[serde(default)]
    tree: bool,
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

// Include default config at compile time
const DEFAULT_CONFIG: &str = include_str!("../default_config.yml");

impl Default for Config {
    fn default() -> Self {
        match serde_yaml::from_str(DEFAULT_CONFIG) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: Error parsing default config: {}. Using hardcoded defaults.", e);
                Self {
                    allowed_extensions: vec![],
                    delimiter: default_delimiter(),
                    subfolder: default_subfolder(),
                    zip: false,
                    tree: false,
                    ignored_directories: default_ignored_directories(),
                    respect_gitignore: default_respect_gitignore(),
                }
            }
        }
    }
}

struct FileProcessor<'a> {
    source_path: &'a Path,
    output_dir: PathBuf,
    config: &'a Config,
    filter: FileFilter<'a>,
    verbose: bool,
    args: &'a Args,
}

impl<'a> FileProcessor<'a> {
    fn new(source: &'a str, config: &'a Config, verbose: bool, args: &'a Args) -> Result<Self> {
        let source_path = Path::new(source);
        let output_dir = source_path.join(&config.subfolder);
        let filter = FileFilter::new(source_path, config)?;
        
        Ok(Self {
            source_path,
            output_dir,
            config,
            filter,
            verbose,
            args,
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

    fn process(&self) -> Result<()> {
        let files = self.collect_files()?;
        
        if self.args.tree {
            self.generate_tree()?;
        }

        if self.args.zip {
            self.create_zip(files)?;
        } else {
            self.copy_files(files)?;
        }
        
        Ok(())
    }

    fn copy_files(&self, files: Vec<(PathBuf, String)>) -> Result<()> {
        if self.verbose {
            println!("Copying {} files to {:?}", files.len(), self.output_dir);
        }

        for (source_path, new_name) in files {
            if !self.filter.should_process_file(&source_path)? {
                if self.verbose {
                    println!("Skipping ignored file: {:?}", source_path);
                }
                continue;
            }

            let target_path = self.output_dir.join(&new_name);

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(&source_path, &target_path)?;

            if self.verbose {
                println!("Copied {:?} to {:?}", source_path, target_path);
            }
        }

        Ok(())
    }

    fn generate_tree(&self) -> Result<()> {
        if self.verbose {
            println!("Generating file tree...");
        }

        let mut seen_dirs = BTreeMap::new();
        let tree_string = generate_tree_string(
            self.source_path,
            "",
            true,
            &mut seen_dirs,
            &self.config.allowed_extensions,
            &self.config.ignored_directories,
            self.filter.gitignore(),
            self.source_path,
        )?;

        let tree_file_path = self.output_dir.join("filetree.txt");
        fs::write(&tree_file_path, tree_string)?;
        
        if self.verbose {
            println!("Tree written to {:?}", tree_file_path);
        }

        Ok(())
    }

    fn create_zip(&self, files: Vec<(PathBuf, String)>) -> Result<()> {
        if self.verbose {
            println!("Starting to create zip archive");
        }

        let zip_path = self.output_dir.join("context.zip");

        let zip_file = fs::File::create(&zip_path)?;
        let mut zip = ZipWriter::new(BufWriter::new(zip_file));

        for (source_path, new_name) in files {
            // Skip if this path should be ignored
            if !self.filter.should_process_file(&source_path)? {
                if self.verbose {
                    println!("Skipping ignored file: {:?}", source_path);
                }
                continue;
            }

            zip.start_file(&new_name, Default::default())?;
            let mut file = fs::File::open(&source_path)?;
            std::io::copy(&mut file, &mut zip)?;
        }

        zip.finish()?;

        if self.verbose {
            println!("Created zip archive at {:?}", zip_path);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.init_config {
        let config_path = Path::new(".lmprep.yml");
        if config_path.exists() {
            eprintln!("Error: Config file already exists at {}", config_path.display());
            std::process::exit(1);
        }
        fs::write(config_path, DEFAULT_CONFIG)?;
        println!("Created default config file at {}", config_path.display());
        return Ok(());
    }

    let mut config = load_config(&args.config)?;

    if let Some(ref subfolder) = args.subfolder {
        config.subfolder = subfolder.clone();
    }

    if args.zip {
        config.zip = true;
    }

    if args.tree {
        config.tree = true;
    }

    if args.verbose {
        eprintln!("Final config after CLI overrides: {:#?}", config);
    }

    let processor = FileProcessor::new(&args.source, &config, args.verbose, &args)?;
    processor.prepare_output_directory()?;
    
    processor.process()?;

    Ok(())
}

fn load_config(config_path: &Option<String>) -> Result<Config> {
    if let Some(path) = config_path {
        return load_config_from_path(path);
    }

    for ext in &[".yml", ".yaml"] {
        let config_path = format!(".lmprep{}", ext);
        if let Ok(config) = load_config_from_path(&config_path) {
            return Ok(config);
        }
    }

    if let Ok(home) = env::var("HOME") {
        let home_path = PathBuf::from(home);
        for ext in &[".yml", ".yaml"] {
            let config_path = home_path.join(format!(".lmprep{}", ext));
            match load_config_from_path(&config_path.to_string_lossy()) {
                Ok(config) => return Ok(config),
                Err(e) if e.to_string().contains("Permission denied") => {
                    eprintln!("Warning: Could not read config from home directory (permission denied)");
                    break;  // Don't try other extensions if we have permission issues
                }
                Err(_) => continue,  // Try next extension
            }
        }
    }

    if let Ok(home) = env::var("USERPROFILE") {
        let home_path = PathBuf::from(home);
        for ext in &[".yml", ".yaml"] {
            let config_path = home_path.join(format!(".lmprep{}", ext));
            match load_config_from_path(&config_path.to_string_lossy()) {
                Ok(config) => return Ok(config),
                Err(e) if e.to_string().contains("Permission denied") => {
                    eprintln!("Warning: Could not read config from home directory (permission denied)");
                    break;  // Don't try other extensions if we have permission issues
                }
                Err(_) => continue,  // Try next extension
            }
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
    let mut result = String::new();

    if path.eq(source_path) {
        result.push_str(&format!(".\n"));
    } else {
        if !FileFilter::should_process_path(path, source_path, allowed_extensions, ignored_directories, gitignore)? {
            return Ok(result);
        }

        if path.is_file() {
            if !allowed_extensions.is_empty() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if !allowed_extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                        return Ok(result);
                    }
                } else {
                    return Ok(result);
                }
            }
        }

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if file_name.is_empty() {
            return Ok(result);
        }

        if path.is_dir() {
            seen_dirs.insert(path.to_path_buf(), true);
        }

        result.push_str(&format!("{}{}{}\n", 
            prefix,
            if is_last { "└── " } else { "├── " },
            if path.is_dir() { format!("{}/", file_name) } else { file_name.to_string() }
        ));
    }

    if path.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                if let Ok(ft) = e.file_type() {
                    ft.is_file() || ft.is_dir()
                } else {
                    false
                }
            })
            .collect();

        entries.sort_by_key(|e| {
            let p = e.path();
            (p.is_file(), e.file_name())
        });

        let num_entries = entries.len();
        for (i, entry) in entries.into_iter().enumerate() {
            let is_last_entry = i == num_entries - 1;
            let new_prefix = format!("{}{}",
                prefix,
                if is_last { "    " } else { "│   " }
            );

            let child_output = generate_tree_string(
                &entry.path(),
                &new_prefix,
                is_last_entry,
                seen_dirs,
                allowed_extensions,
                ignored_directories,
                gitignore,
                source_path,
            )?;
            
            result.push_str(&child_output);
        }
    }

    Ok(result)
}