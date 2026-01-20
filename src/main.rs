//! null-e CLI - The Friendly Disk Cleanup Robot
//!
//! 🤖 Send your dev cruft to /dev/null with style!

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use null_e::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// ASCII art robot mascot for null-e
const ROBOT_BANNER: &str = r#"
     .---.
    |o   o|    null-e
    |  ^  |    ═══════════════════════
    | === |    The friendly disk cleanup robot
    `-----'    Send your cruft to /dev/null!
     /| |\
"#;

/// Small robot for inline display
const ROBOT_SMALL: &str = "🤖";

/// 🤖 null-e - The Friendly Disk Cleanup Robot
///
/// Find and clean node_modules, target, .venv, and 30+ more artifact types.
/// Protects your uncommitted changes and supports safe deletion via trash.
#[derive(Parser)]
#[command(name = "null-e")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Directories to scan (default: current directory)
    #[arg(global = true)]
    paths: Vec<PathBuf>,

    /// Maximum depth to scan
    #[arg(short = 'd', long, global = true)]
    max_depth: Option<usize>,

    /// Minimum artifact size to show (e.g., 1MB, 100KB) [default: 1MB]
    #[arg(short = 's', long, global = true)]
    min_size: Option<String>,

    /// Show top N largest projects [default: 25, use 0 for all]
    #[arg(short = 't', long, global = true)]
    top: Option<usize>,

    /// Protection level for git repos
    #[arg(short = 'p', long, global = true, value_enum, default_value = "warn")]
    protection: ProtectionArg,

    /// Delete method
    #[arg(short = 'm', long, global = true, value_enum, default_value = "trash")]
    method: DeleteMethodArg,

    /// Force clean (skip confirmations)
    #[arg(short = 'f', long, global = true)]
    force: bool,

    /// Dry run (don't actually delete)
    #[arg(short = 'n', long, global = true)]
    dry_run: bool,

    /// Verbose output (show all projects)
    #[arg(short = 'v', long, global = true)]
    verbose: bool,

    /// Output format
    #[arg(long, global = true, value_enum, default_value = "pretty")]
    output: OutputFormat,

    /// Show all projects (no limit)
    #[arg(short = 'a', long, global = true)]
    all: bool,

    /// Skip cache, force full rescan
    #[arg(long, global = true)]
    no_cache: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan directories for cleanable artifacts
    Scan {
        /// Show detailed artifact information
        #[arg(long)]
        detailed: bool,
    },

    /// Clean (delete) artifacts
    Clean {
        /// Only clean specific artifact types
        #[arg(long)]
        only: Vec<String>,

        /// Exclude specific artifact types
        #[arg(long)]
        exclude: Vec<String>,
    },

    /// Show configuration
    Config {
        /// Initialize default config file
        #[arg(long)]
        init: bool,

        /// Show config file path
        #[arg(long)]
        path: bool,
    },

    /// List supported project types
    List,

    /// Manage global developer caches (npm, pip, cargo, etc.)
    Caches {
        /// Clean selected caches
        #[arg(long)]
        clean: bool,

        /// Clean all caches without prompting
        #[arg(long)]
        clean_all: bool,

        /// Use official clean commands when available
        #[arg(long, default_value = "true")]
        official: bool,
    },

    /// Deep sweep: find ALL cleanable items (Xcode, Android, Docker, ML, IDEs, logs)
    Sweep {
        /// Clean selected items
        #[arg(long)]
        clean: bool,

        /// Filter by category (xcode, android, docker, ml, ide, logs)
        #[arg(long)]
        category: Option<String>,
    },

    /// Clean Xcode artifacts (DerivedData, Archives, Simulators)
    Xcode {
        /// Clean all Xcode artifacts
        #[arg(long)]
        clean: bool,
    },

    /// Clean Android development artifacts (AVD, Gradle, SDK)
    Android {
        /// Clean all Android artifacts
        #[arg(long)]
        clean: bool,
    },

    /// Clean Docker resources (images, containers, volumes)
    Docker {
        /// Clean all Docker resources
        #[arg(long)]
        clean: bool,

        /// Include volumes (data may be lost!)
        #[arg(long)]
        volumes: bool,
    },

    /// Clean ML/AI caches (Huggingface, Ollama, PyTorch)
    Ml {
        /// Clean selected ML caches
        #[arg(long)]
        clean: bool,
    },

    /// Clean IDE caches (JetBrains, VS Code, Cursor)
    Ide {
        /// Clean IDE caches
        #[arg(long)]
        clean: bool,
    },

    /// Clean Homebrew caches and old versions
    Homebrew {
        /// Clean Homebrew caches
        #[arg(long)]
        clean: bool,

        /// Scrub cache (remove even latest version downloads)
        #[arg(long)]
        scrub: bool,
    },

    /// Clean iOS dependency caches (CocoaPods, Carthage, SPM)
    IosDeps {
        /// Clean iOS dependency caches
        #[arg(long)]
        clean: bool,
    },

    /// Clean Electron app caches (Slack, Discord, Spotify, etc.)
    Electron {
        /// Clean Electron app caches
        #[arg(long)]
        clean: bool,
    },

    /// Clean game development caches (Unity, Unreal, Godot)
    Gamedev {
        /// Clean game dev caches
        #[arg(long)]
        clean: bool,
    },

    /// Clean cloud CLI caches (AWS, GCP, Azure, kubectl, Terraform)
    Cloud {
        /// Clean cloud CLI caches
        #[arg(long)]
        clean: bool,
    },

    /// Clean macOS system caches (orphaned containers, app remnants)
    #[cfg(target_os = "macos")]
    Macos {
        /// Clean macOS caches
        #[arg(long)]
        clean: bool,
    },

    /// Analyze git repositories (find large .git, suggest gc)
    GitAnalyze {
        /// Run git gc on repositories that need it
        #[arg(long)]
        fix: bool,
    },

    /// Find stale projects not touched in months
    Stale {
        /// Days since last activity to consider stale (default: 180)
        #[arg(long, default_value = "180")]
        days: u64,

        /// Clean build artifacts from stale projects
        #[arg(long)]
        clean: bool,
    },

    /// Find duplicate dependencies across projects
    Duplicates,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum ProtectionArg {
    None,
    Warn,
    Block,
    Paranoid,
}

impl From<ProtectionArg> for ProtectionLevel {
    fn from(arg: ProtectionArg) -> Self {
        match arg {
            ProtectionArg::None => ProtectionLevel::None,
            ProtectionArg::Warn => ProtectionLevel::Warn,
            ProtectionArg::Block => ProtectionLevel::Block,
            ProtectionArg::Paranoid => ProtectionLevel::Paranoid,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum DeleteMethodArg {
    Trash,
    Permanent,
    DryRun,
}

impl From<DeleteMethodArg> for DeleteMethod {
    fn from(arg: DeleteMethodArg) -> Self {
        match arg {
            DeleteMethodArg::Trash => DeleteMethod::Trash,
            DeleteMethodArg::Permanent => DeleteMethod::Permanent,
            DeleteMethodArg::DryRun => DeleteMethod::DryRun,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Pretty,
    Json,
    Compact,
}

fn main() {
    let cli = Cli::parse();

    // Set up logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("null_e=debug")
            .init();
    }

    // Run the appropriate command
    let result = match &cli.command {
        Some(Commands::Scan { detailed }) => cmd_scan(&cli, *detailed),
        Some(Commands::Clean { only, exclude }) => cmd_clean(&cli, only, exclude),
        Some(Commands::Config { init, path }) => cmd_config(*init, *path),
        Some(Commands::List) => cmd_list(),
        Some(Commands::Caches { clean, clean_all, official }) => {
            cmd_caches(&cli, *clean, *clean_all, *official)
        }
        Some(Commands::Sweep { clean, category }) => {
            cmd_sweep(&cli, *clean, category.as_deref())
        }
        Some(Commands::Xcode { clean }) => cmd_xcode(&cli, *clean),
        Some(Commands::Android { clean }) => cmd_android(&cli, *clean),
        Some(Commands::Docker { clean, volumes }) => cmd_docker(&cli, *clean, *volumes),
        Some(Commands::Ml { clean }) => cmd_ml(&cli, *clean),
        Some(Commands::Ide { clean }) => cmd_ide(&cli, *clean),
        Some(Commands::Homebrew { clean, scrub }) => cmd_homebrew(&cli, *clean, *scrub),
        Some(Commands::IosDeps { clean }) => cmd_ios_deps(&cli, *clean),
        Some(Commands::Electron { clean }) => cmd_electron(&cli, *clean),
        Some(Commands::Gamedev { clean }) => cmd_gamedev(&cli, *clean),
        Some(Commands::Cloud { clean }) => cmd_cloud(&cli, *clean),
        #[cfg(target_os = "macos")]
        Some(Commands::Macos { clean }) => cmd_macos(&cli, *clean),
        Some(Commands::GitAnalyze { fix }) => cmd_git_analyze(&cli, *fix),
        Some(Commands::Stale { days, clean }) => cmd_stale(&cli, *days, *clean),
        Some(Commands::Duplicates) => cmd_duplicates(&cli),
        None => cmd_scan(&cli, false), // Default to scan
    };

    // Handle errors
    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        if let Some(suggestion) = e.suggested_action() {
            eprintln!("{} {}", "Hint:".yellow(), suggestion);
        }
        std::process::exit(e.exit_code());
    }
}

fn cmd_scan(cli: &Cli, detailed: bool) -> Result<()> {
    let paths = get_scan_paths(cli)?;
    let use_cache = !cli.no_cache;

    println!("{}", ROBOT_BANNER.green());
    println!(
        "{} {}",
        format!("{} null-e", ROBOT_SMALL).green().bold(),
        format!("v{}", null_e::VERSION).dimmed()
    );

    // Try to load cache
    let mut cache = if use_cache {
        null_e::cache::load_cache().unwrap_or_default()
    } else {
        null_e::cache::ScanCache::new()
    };

    // Check if we can use cached results
    let cache_valid = use_cache && cache.is_valid() && cache.project_count() > 0;

    // If cache is valid and no specific paths requested, use cached data
    if cache_valid && paths.len() == 1 {
        let cached_projects = cache.get_all_valid_projects();
        if !cached_projects.is_empty() {
            // Filter to projects under the requested path
            let root = &paths[0];
            let mut projects: Vec<_> = cached_projects
                .into_iter()
                .filter(|p| p.root.starts_with(root))
                .collect();

            if !projects.is_empty() {
                println!("{} {}", "⚡".yellow(), "Using cached results (use --no-cache to rescan)".dimmed());
                println!();

                // Apply min_size filter
                let min_size = if let Some(ref size_str) = cli.min_size {
                    parse_size(size_str).unwrap_or(0)
                } else if !cli.verbose && !cli.all {
                    1_000_000
                } else {
                    0
                };

                projects.retain(|p| p.cleanable_size >= min_size);
                projects.sort_by(|a, b| b.cleanable_size.cmp(&a.cleanable_size));

                // Create a minimal config for display
                let mut cached_config = ScanConfig::default();
                cached_config.roots = paths.clone();

                return display_scan_results(cli, &cached_config, projects, 0, Duration::from_millis(1), detailed);
            }
        }
    }

    // Create scanner
    let registry = Arc::new(PluginRegistry::with_builtins());
    let scanner = ParallelScanner::new(registry);

    // Build config with smart defaults
    let mut config = ScanConfig::default();
    config.roots = paths.clone();

    if let Some(depth) = cli.max_depth {
        config.max_depth = Some(depth);
    }

    // Default min_size to 1MB unless specified or verbose mode
    if let Some(ref size_str) = cli.min_size {
        config.min_size = parse_size(size_str);
    } else if !cli.verbose && !cli.all {
        config.min_size = Some(1_000_000); // 1MB default
    }

    // Get progress handle for real-time updates
    let progress = scanner.progress();

    // Create progress bar with real-time updates
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(100));

    // Clone progress for the display thread
    let progress_clone = Arc::clone(&progress);
    let pb_clone = pb.clone();

    // Spawn thread to update progress bar
    let progress_thread = thread::spawn(move || {
        loop {
            let snapshot = progress_clone.snapshot();
            if snapshot.is_complete {
                break;
            }

            let size_str = format_size(snapshot.total_size_found);
            let msg = format!(
                "Scanning... {} dirs | {} projects | {} found",
                snapshot.directories_scanned,
                snapshot.projects_found,
                size_str.yellow()
            );
            pb_clone.set_message(msg);

            thread::sleep(Duration::from_millis(50));
        }
    });

    // Scan
    let scan_result = scanner.scan(&config)?;

    // Wait for progress thread
    let _ = progress_thread.join();
    pb.finish_and_clear();

    let mut projects = scan_result.projects;

    // Update cache with new results
    if use_cache {
        for project in &projects {
            cache.cache_project(project.clone());
        }
        cache.touch();
        let _ = null_e::cache::save_cache(&cache); // Ignore save errors
    }

    // Sort by size (largest first)
    projects.sort_by(|a, b| b.cleanable_size.cmp(&a.cleanable_size));

    display_scan_results(cli, &config, projects, scan_result.directories_scanned, scan_result.duration, detailed)
}

fn display_scan_results(
    cli: &Cli,
    config: &ScanConfig,
    projects: Vec<Project>,
    dirs_scanned: usize,
    duration: Duration,
    detailed: bool,
) -> Result<()> {
    // Determine display limit
    let display_limit = if cli.all || cli.verbose {
        projects.len()
    } else {
        cli.top.unwrap_or(25)
    };

    let total_projects = projects.len();
    let total_size: u64 = projects.iter().map(|p| p.cleanable_size).sum();

    // Split into displayed and hidden
    let (displayed, hidden): (Vec<_>, Vec<_>) = if display_limit > 0 && display_limit < projects.len() {
        let (d, h) = projects.split_at(display_limit);
        (d.to_vec(), h.to_vec())
    } else {
        (projects, vec![])
    };

    // Display header
    println!(
        "{} Found {} projects with {} cleanable",
        "✓".green(),
        total_projects.to_string().cyan(),
        format_size(total_size).yellow().bold()
    );

    if dirs_scanned > 0 {
        println!(
            "  {} Scanned {} directories in {:.2}s",
            "│".dimmed(),
            dirs_scanned.to_string().dimmed(),
            duration.as_secs_f64()
        );
    }

    println!();

    if displayed.is_empty() {
        println!("  No cleanable artifacts found (min size: 1MB).");
        println!("  {}", "Use --min-size 0 or -v to see smaller items".dimmed());
        return Ok(());
    }

    // Display top projects
    if !hidden.is_empty() {
        println!(
            "{} {} (use {} to see all)",
            "Top".bold(),
            format!("{}", display_limit).cyan(),
            "-a".cyan()
        );
        println!();
    }

    // Display projects with better formatting
    for (i, project) in displayed.iter().enumerate() {
        // Format size with padding for alignment
        let size_str = format_size(project.cleanable_size);
        let padded_size = format!("{:>10}", size_str);

        // Get relative path if possible
        let display_path = if let Some(first_root) = config.roots.first() {
            project.root.strip_prefix(first_root)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| project.root.display().to_string())
        } else {
            project.root.display().to_string()
        };

        // Main line
        println!(
            "{} {} {} {} {}",
            "•".cyan(),
            project.kind.icon(),
            padded_size.yellow(),
            project.name.bold(),
            format!("({})", display_path).dimmed()
        );

        // Show artifacts in detailed mode
        if detailed {
            for artifact in &project.artifacts {
                println!(
                    "       {} {} {}",
                    "├──".dimmed(),
                    artifact.name(),
                    format_size(artifact.size).dimmed()
                );
            }
        }

        // Add spacing between entries for readability
        if i < displayed.len() - 1 && (i + 1) % 10 == 0 {
            println!();
        }
    }

    // Show summary of hidden projects
    if !hidden.is_empty() {
        let hidden_size: u64 = hidden.iter().map(|p| p.cleanable_size).sum();
        println!();
        println!(
            "{} {} more projects ({}) - use {} or {} to see all",
            "...".dimmed(),
            hidden.len().to_string().dimmed(),
            format_size(hidden_size).dimmed(),
            "-a".cyan(),
            "-v".cyan()
        );
    }

    println!();

    // Quick actions hint
    if total_size > 100_000_000 { // > 100MB
        println!(
            "{} {} {} {}",
            "💡".dimmed(),
            "Quick clean:".dimmed(),
            "null-eclean --dry-run".cyan(),
            "(preview what would be deleted)".dimmed()
        );
    }

    Ok(())
}

fn cmd_clean(cli: &Cli, only: &[String], exclude: &[String]) -> Result<()> {
    let paths = get_scan_paths(cli)?;

    println!(
        "{} {}",
        "🤖 null-e Clean".green().bold(),
        format!("v{}", null_e::VERSION).dimmed()
    );
    println!();

    // Create scanner
    let registry = Arc::new(PluginRegistry::with_builtins());
    let scanner = ParallelScanner::new(registry);

    // Build config
    let mut config = ScanConfig::default();
    config.roots = paths;
    if let Some(depth) = cli.max_depth {
        config.max_depth = Some(depth);
    }

    // Scan
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Scanning for cleanable artifacts...");

    let result = scanner.scan(&config)?;
    pb.finish_and_clear();

    let mut projects = result.projects;
    null_e::git::enrich_with_git_status(&mut projects)?;

    if projects.is_empty() {
        println!("  No cleanable artifacts found.");
        return Ok(());
    }

    // Filter projects by protection level
    let protection_level: ProtectionLevel = cli.protection.into();
    let (cleanable, blocked): (Vec<_>, Vec<_>) = projects.into_iter().partition(|p| {
        let check = null_e::git::check_project_protection(p, protection_level);
        check.allowed || cli.force
    });

    // Show blocked projects
    if !blocked.is_empty() {
        println!(
            "{} {} projects blocked (use --force to override):",
            "🔒".red(),
            blocked.len()
        );
        for project in &blocked {
            println!("    {} {}", "•".red(), project.name);
        }
        println!();
    }

    if cleanable.is_empty() {
        println!("  No projects available to clean.");
        return Ok(());
    }

    // Calculate total
    let total_size: u64 = cleanable.iter().map(|p| p.cleanable_size).sum();
    let total_artifacts: usize = cleanable.iter().map(|p| p.artifacts.len()).sum();

    println!(
        "Will clean {} artifacts from {} projects ({})",
        total_artifacts.to_string().cyan(),
        cleanable.len().to_string().cyan(),
        format_size(total_size).yellow().bold()
    );

    // Determine delete method
    let method = if cli.dry_run {
        DeleteMethod::DryRun
    } else {
        cli.method.into()
    };

    // Confirm unless force or dry-run
    if !cli.force && method != DeleteMethod::DryRun {
        println!();
        println!(
            "{}",
            format!(
                "Delete method: {}",
                match method {
                    DeleteMethod::Trash => "Move to trash (recoverable)",
                    DeleteMethod::Permanent => "PERMANENT DELETE (not recoverable!)",
                    DeleteMethod::DryRun => "Dry run",
                }
            )
            .dimmed()
        );
        println!();

        print!("Continue? [y/N] ");
        use std::io::Write;
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Clean
    println!();
    let pb = ProgressBar::new(total_artifacts as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );

    let mut cleaned_size = 0u64;
    let mut cleaned_count = 0usize;
    let mut failed_count = 0usize;

    for project in &cleanable {
        for artifact in &project.artifacts {
            pb.set_message(format!("{}/{}", project.name, artifact.name()));

            let result = delete_artifact(artifact, method);

            if result.success {
                cleaned_size += result.bytes_freed;
                cleaned_count += 1;
            } else {
                failed_count += 1;
                if cli.verbose {
                    if let Some(err) = &result.error {
                        eprintln!("  {} Failed: {}", "✗".red(), err);
                    }
                }
            }

            pb.inc(1);
        }
    }

    pb.finish_and_clear();

    // Summary
    println!();
    if method == DeleteMethod::DryRun {
        println!(
            "{} Dry run complete. Would clean {} from {} artifacts.",
            "✓".green(),
            format_size(cleaned_size).yellow().bold(),
            cleaned_count
        );
    } else {
        println!(
            "{} Cleaned {} from {} artifacts.",
            "✓".green(),
            format_size(cleaned_size).yellow().bold(),
            cleaned_count
        );
    }

    if failed_count > 0 {
        println!(
            "{} {} artifacts failed to clean",
            "⚠".yellow(),
            failed_count
        );
    }

    Ok(())
}

fn cmd_config(init: bool, show_path: bool) -> Result<()> {
    if init {
        let path = null_e::config::init_config()?;
        println!("{} Created config file at:", "✓".green());
        println!("  {}", path.display());
        return Ok(());
    }

    if show_path {
        let path = null_e::config::default_config_path()?;
        println!("{}", path.display());
        return Ok(());
    }

    // Show current config
    let config = null_e::config::load_default_config()?;
    println!("{}", toml::to_string_pretty(&config)?);

    Ok(())
}

fn cmd_list() -> Result<()> {
    println!("{}", "Supported Project Types:".bold());
    println!();

    let registry = PluginRegistry::with_builtins();

    for plugin in registry.all() {
        println!(
            "  {} {} ({})",
            "•".green(),
            plugin.name().bold(),
            plugin.id().dimmed()
        );

        let dirs: Vec<_> = plugin.cleanable_dirs().iter().take(5).collect();
        if !dirs.is_empty() {
            println!(
                "    Cleans: {}",
                dirs.iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    Ok(())
}

fn cmd_caches(cli: &Cli, clean: bool, clean_all: bool, use_official: bool) -> Result<()> {
    use null_e::caches::{detect_caches, calculate_all_sizes, clean_cache, CachesSummary};

    println!(
        "{} {}",
        "🤖 null-e Caches".green().bold(),
        format!("v{}", null_e::VERSION).dimmed()
    );
    println!();

    // Detect existing caches
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Detecting global caches...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut caches = detect_caches()?;

    if caches.is_empty() {
        pb.finish_and_clear();
        println!("  No global developer caches found.");
        return Ok(());
    }

    pb.set_message(format!("Calculating sizes for {} caches...", caches.len()));

    // Calculate sizes
    calculate_all_sizes(&mut caches)?;

    // Filter out empty caches
    caches.retain(|c| c.size > 0);

    // Sort by size descending
    caches.sort_by(|a, b| b.size.cmp(&a.size));

    pb.finish_and_clear();

    if caches.is_empty() {
        println!("  No caches with data found.");
        return Ok(());
    }

    // Calculate summary
    let summary = CachesSummary::from_caches(&caches);

    // Display header
    println!(
        "{} Found {} caches with {} total",
        "✓".green(),
        caches.len().to_string().cyan(),
        format_size(summary.total_size).yellow().bold()
    );
    println!();

    // Table header
    println!(
        "   {:3} {:<24} {:>12}   {:>12}   {}",
        "".dimmed(),
        "Cache".bold(),
        "Size".bold(),
        "Last Used".bold(),
        "Clean Command".dimmed()
    );
    println!("   {}", "─".repeat(80).dimmed());

    // Display each cache with selection number
    for (i, cache) in caches.iter().enumerate() {
        let num = format!("[{}]", i + 1);
        let size_str = format_size(cache.size);
        let last_used = cache.last_used_display();
        let cmd = cache.clean_command.unwrap_or("-");

        // Color code by size
        let size_colored = if cache.size > 1_000_000_000 {
            size_str.red().bold().to_string()
        } else if cache.size > 100_000_000 {
            size_str.yellow().to_string()
        } else {
            size_str.normal().to_string()
        };

        println!(
            "   {} {} {:<22} {:>12}   {:>12}   {}",
            num.cyan(),
            cache.icon,
            cache.name,
            size_colored,
            last_used.dimmed(),
            cmd.dimmed()
        );
    }

    println!("   {}", "─".repeat(80).dimmed());
    println!(
        "   {:3} {:<24} {:>12}",
        "",
        "Total".bold(),
        format_size(summary.total_size).yellow().bold()
    );
    println!();

    // If not cleaning, show hints
    if !clean && !clean_all {
        println!(
            "{} {}",
            "💡".dimmed(),
            "Commands:".dimmed()
        );
        println!(
            "   {} {}",
            "null-ecaches --clean".cyan(),
            "Interactive selection".dimmed()
        );
        println!(
            "   {} {}",
            "null-ecaches --clean-all".cyan(),
            "Clean all caches".dimmed()
        );
        return Ok(());
    }

    // Clean all without prompting
    if clean_all {
        if !cli.force && !cli.dry_run {
            println!(
                "{} This will clean ALL {} caches ({})!",
                "⚠️".yellow(),
                caches.len(),
                format_size(summary.total_size).yellow()
            );
            print!("Continue? [y/N] ");
            use std::io::Write;
            std::io::stdout().flush().unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }

        return clean_selected_caches(&caches, cli, use_official);
    }

    // Interactive selection
    if clean {
        println!(
            "Enter cache numbers to clean (e.g., {} or {} or {}):",
            "1,3,5".cyan(),
            "1-5".cyan(),
            "all".cyan()
        );
        print!("> ");
        use std::io::Write;
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        if input.is_empty() || input == "q" || input == "quit" {
            println!("Aborted.");
            return Ok(());
        }

        let selected_indices: Vec<usize> = if input == "all" || input == "a" {
            (0..caches.len()).collect()
        } else {
            parse_selection(&input, caches.len())
        };

        if selected_indices.is_empty() {
            println!("No valid selection.");
            return Ok(());
        }

        let selected_caches: Vec<_> = selected_indices
            .iter()
            .filter_map(|&i| caches.get(i).cloned())
            .collect();

        let selected_size: u64 = selected_caches.iter().map(|c| c.size).sum();

        println!();
        println!(
            "Selected {} caches ({})",
            selected_caches.len().to_string().cyan(),
            format_size(selected_size).yellow()
        );

        if !cli.force && !cli.dry_run {
            print!("Continue? [y/N] ");
            std::io::stdout().flush().unwrap();

            let mut confirm = String::new();
            std::io::stdin().read_line(&mut confirm).unwrap();

            if !confirm.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }

        return clean_selected_caches(&selected_caches, cli, use_official);
    }

    Ok(())
}

fn clean_selected_caches(
    caches: &[null_e::caches::GlobalCache],
    cli: &Cli,
    use_official: bool,
) -> Result<()> {
    use null_e::caches::clean_cache;

    println!();

    let pb = ProgressBar::new(caches.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );

    let mut cleaned_size = 0u64;
    let mut cleaned_count = 0usize;
    let mut failed_count = 0usize;

    for cache in caches {
        pb.set_message(cache.name.clone());

        if cli.dry_run {
            // Dry run - just pretend
            cleaned_size += cache.size;
            cleaned_count += 1;
        } else {
            match clean_cache(cache, use_official) {
                Ok(result) => {
                    if result.success {
                        cleaned_size += result.bytes_freed;
                        cleaned_count += 1;
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    if cli.verbose {
                        eprintln!("  {} Failed to clean {}: {}", "✗".red(), cache.name, e);
                    }
                }
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    // Summary
    println!();
    if cli.dry_run {
        println!(
            "{} Dry run complete. Would free {} from {} caches.",
            "✓".green(),
            format_size(cleaned_size).yellow().bold(),
            cleaned_count
        );
    } else {
        println!(
            "{} Cleaned {} from {} caches.",
            "✓".green(),
            format_size(cleaned_size).yellow().bold(),
            cleaned_count
        );
    }

    if failed_count > 0 {
        println!(
            "{} {} caches failed to clean",
            "⚠".yellow(),
            failed_count
        );
    }

    Ok(())
}

/// Parse selection like "1,3,5" or "1-5" or "1,3-5,7"
fn parse_selection(input: &str, max: usize) -> Vec<usize> {
    let mut result = Vec::new();

    for part in input.split(',') {
        let part = part.trim();

        if part.contains('-') {
            // Range like "1-5"
            let parts: Vec<&str> = part.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                    for i in start..=end {
                        if i >= 1 && i <= max {
                            result.push(i - 1); // Convert to 0-indexed
                        }
                    }
                }
            }
        } else {
            // Single number
            if let Ok(n) = part.parse::<usize>() {
                if n >= 1 && n <= max {
                    result.push(n - 1); // Convert to 0-indexed
                }
            }
        }
    }

    result.sort();
    result.dedup();
    result
}

fn get_scan_paths(cli: &Cli) -> Result<Vec<PathBuf>> {
    if cli.paths.is_empty() {
        // Use current directory
        Ok(vec![std::env::current_dir()?])
    } else {
        // Validate paths
        for path in &cli.paths {
            if !path.exists() {
                return Err(DevSweepError::PathNotFound(path.clone()));
            }
        }
        Ok(cli.paths.clone())
    }
}

fn format_size(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::BINARY)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Sweep Command - The Big One!
// ═══════════════════════════════════════════════════════════════════════════════

fn cmd_sweep(cli: &Cli, clean: bool, category: Option<&str>) -> Result<()> {
    use null_e::cleaners::{
        xcode::XcodeCleaner,
        android::AndroidCleaner,
        docker::DockerCleaner,
        ml::MlCleaner,
        ide::IdeCleaner,
        logs::LogsCleaner,
        homebrew::HomebrewCleaner,
        ios_deps::IosDependencyCleaner,
        electron::ElectronCleaner,
        gamedev::GameDevCleaner,
        cloud::CloudCliCleaner,
        macos::MacOsCleaner,
        CleanableItem, CleanerSummary,
    };

    println!(
        "{} {}",
        "🤖 null-e Deep Scan".green().bold(),
        format!("v{}", null_e::VERSION).dimmed()
    );
    println!();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut all_items: Vec<CleanableItem> = Vec::new();

    // Collect items from all cleaners based on category filter
    let categories: Vec<&str> = match category {
        Some(c) => vec![c],
        None => vec!["xcode", "android", "docker", "ml", "ide", "logs", "homebrew", "ios", "electron", "gamedev", "cloud", "macos"],
    };

    for cat in &categories {
        match *cat {
            "xcode" => {
                pb.set_message("Scanning Xcode...");
                if let Some(cleaner) = XcodeCleaner::new() {
                    if let Ok(items) = cleaner.detect() {
                        all_items.extend(items);
                    }
                }
            }
            "android" => {
                pb.set_message("Scanning Android...");
                if let Some(cleaner) = AndroidCleaner::new() {
                    if let Ok(items) = cleaner.detect() {
                        all_items.extend(items);
                    }
                }
            }
            "docker" => {
                pb.set_message("Scanning Docker...");
                let cleaner = DockerCleaner::new();
                if let Ok(items) = cleaner.detect() {
                    all_items.extend(items);
                }
            }
            "ml" => {
                pb.set_message("Scanning ML/AI...");
                if let Some(cleaner) = MlCleaner::new() {
                    if let Ok(items) = cleaner.detect() {
                        all_items.extend(items);
                    }
                }
            }
            "ide" => {
                pb.set_message("Scanning IDEs...");
                if let Some(cleaner) = IdeCleaner::new() {
                    if let Ok(items) = cleaner.detect() {
                        all_items.extend(items);
                    }
                }
            }
            "logs" => {
                pb.set_message("Scanning Logs...");
                if let Some(cleaner) = LogsCleaner::new() {
                    if let Ok(items) = cleaner.detect() {
                        all_items.extend(items);
                    }
                }
            }
            "homebrew" => {
                pb.set_message("Scanning Homebrew...");
                if let Some(cleaner) = HomebrewCleaner::new() {
                    if cleaner.is_available() {
                        if let Ok(items) = cleaner.detect() {
                            all_items.extend(items);
                        }
                    }
                }
            }
            "ios" | "ios-deps" => {
                pb.set_message("Scanning iOS Dependencies...");
                if let Some(cleaner) = IosDependencyCleaner::new() {
                    if let Ok(items) = cleaner.detect() {
                        all_items.extend(items);
                    }
                }
            }
            "electron" => {
                pb.set_message("Scanning Electron Apps...");
                if let Some(cleaner) = ElectronCleaner::new() {
                    if let Ok(items) = cleaner.detect() {
                        all_items.extend(items);
                    }
                }
            }
            "gamedev" | "unity" | "unreal" => {
                pb.set_message("Scanning Game Dev...");
                if let Some(cleaner) = GameDevCleaner::new() {
                    if let Ok(items) = cleaner.detect() {
                        all_items.extend(items);
                    }
                }
            }
            "cloud" | "aws" | "gcp" | "azure" => {
                pb.set_message("Scanning Cloud CLI...");
                if let Some(cleaner) = CloudCliCleaner::new() {
                    if let Ok(items) = cleaner.detect() {
                        all_items.extend(items);
                    }
                }
            }
            "macos" | "system" => {
                pb.set_message("Scanning macOS System...");
                if let Some(cleaner) = MacOsCleaner::new() {
                    if let Ok(items) = cleaner.detect() {
                        all_items.extend(items);
                    }
                }
            }
            _ => {}
        }
    }

    pb.finish_and_clear();

    if all_items.is_empty() {
        println!("  No cleanable items found.");
        return Ok(());
    }

    // Sort by size descending
    all_items.sort_by(|a, b| b.size.cmp(&a.size));

    // Calculate summary
    let summary = CleanerSummary::from_items(&all_items);

    // Display header
    println!(
        "{} Found {} items with {} total",
        "✓".green(),
        all_items.len().to_string().cyan(),
        format_size(summary.total_size).yellow().bold()
    );
    println!();

    // Display by category
    let mut sorted_categories: Vec<_> = summary.by_category.values().collect();
    sorted_categories.sort_by(|a, b| b.total_size.cmp(&a.total_size));

    println!("   {}", "By Category:".bold());
    for cat in &sorted_categories {
        println!(
            "   {} {:<20} {:>12}  ({} items)",
            cat.icon,
            cat.name,
            format_size(cat.total_size).yellow(),
            cat.item_count
        );
    }
    println!();

    // Table header
    println!(
        "   {:3} {:<40} {:>12}   {}",
        "".dimmed(),
        "Item".bold(),
        "Size".bold(),
        "Safety".bold()
    );
    println!("   {}", "─".repeat(75).dimmed());

    // Display items (top 30 or all if verbose)
    let display_count = if cli.verbose || cli.all { all_items.len() } else { 30.min(all_items.len()) };

    for (i, item) in all_items.iter().take(display_count).enumerate() {
        let num = format!("[{}]", i + 1);
        let size_str = format_size(item.size);

        // Color code by size
        let size_colored = if item.size > 1_000_000_000 {
            size_str.red().bold().to_string()
        } else if item.size > 100_000_000 {
            size_str.yellow().to_string()
        } else {
            size_str.normal().to_string()
        };

        // Safety indicator
        let safety = match item.safe_to_delete {
            null_e::cleaners::SafetyLevel::Safe => "✓ Safe".green().to_string(),
            null_e::cleaners::SafetyLevel::SafeWithCost => "~ Rebuild".yellow().to_string(),
            null_e::cleaners::SafetyLevel::Caution => "! Caution".red().to_string(),
            null_e::cleaners::SafetyLevel::Dangerous => "⚠ Danger".magenta().to_string(),
        };

        let name = if item.name.len() > 38 {
            format!("{}...", &item.name[..35])
        } else {
            item.name.clone()
        };

        println!(
            "   {} {} {:<38} {:>12}   {}",
            num.cyan(),
            item.icon,
            name,
            size_colored,
            safety
        );
    }

    if all_items.len() > display_count {
        println!(
            "   {} {} more items (use -v to show all)",
            "...".dimmed(),
            all_items.len() - display_count
        );
    }

    println!("   {}", "─".repeat(75).dimmed());
    println!(
        "   {:3} {:<40} {:>12}",
        "",
        "Total".bold(),
        format_size(summary.total_size).yellow().bold()
    );
    println!();

    // If not cleaning, show hints
    if !clean {
        println!("{} {}", "💡".dimmed(), "Commands:".dimmed());
        println!(
            "   {} {}",
            "null-esweep --clean".cyan(),
            "Interactive selection".dimmed()
        );
        println!(
            "   {} {}",
            "null-esweep --category xcode".cyan(),
            "Filter by category".dimmed()
        );
        return Ok(());
    }

    // Interactive cleaning
    clean_items_interactive(&all_items, cli)
}

fn clean_items_interactive(items: &[null_e::cleaners::CleanableItem], cli: &Cli) -> Result<()> {
    println!(
        "Enter item numbers to clean (e.g., {} or {} or {}):",
        "1,3,5".cyan(),
        "1-5".cyan(),
        "all".cyan()
    );
    print!("> ");
    use std::io::Write;
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();

    if input.is_empty() || input == "q" || input == "quit" {
        println!("Aborted.");
        return Ok(());
    }

    let selected_indices: Vec<usize> = if input == "all" || input == "a" {
        (0..items.len()).collect()
    } else {
        parse_selection(&input, items.len())
    };

    if selected_indices.is_empty() {
        println!("No valid selection.");
        return Ok(());
    }

    let selected_items: Vec<_> = selected_indices
        .iter()
        .filter_map(|&i| items.get(i))
        .collect();

    let selected_size: u64 = selected_items.iter().map(|i| i.size).sum();

    println!();
    println!(
        "Selected {} items ({})",
        selected_items.len().to_string().cyan(),
        format_size(selected_size).yellow()
    );

    // Show what will be deleted
    for item in &selected_items {
        println!("  {} {} {}", item.icon, item.name, format_size(item.size).dimmed());
    }

    if !cli.force && !cli.dry_run {
        print!("\nContinue? [y/N] ");
        std::io::stdout().flush().unwrap();

        let mut confirm = String::new();
        std::io::stdin().read_line(&mut confirm).unwrap();

        if !confirm.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Clean selected items
    let pb = ProgressBar::new(selected_items.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );

    let mut cleaned_size = 0u64;
    let mut cleaned_count = 0usize;
    let mut failed_count = 0usize;

    let delete_method: DeleteMethod = if cli.dry_run {
        DeleteMethod::DryRun
    } else {
        cli.method.into()
    };

    for item in selected_items {
        pb.set_message(item.name.clone());

        if cli.dry_run {
            cleaned_size += item.size;
            cleaned_count += 1;
        } else {
            // Use official command if available, otherwise delete directly
            if let Some(ref cmd) = item.clean_command {
                // Try running the clean command
                if run_clean_command_silent(cmd).is_ok() {
                    cleaned_size += item.size;
                    cleaned_count += 1;
                } else {
                    // Fall back to direct deletion
                    match null_e::trash::delete_path(&item.path, delete_method) {
                        Ok(_) => {
                            cleaned_size += item.size;
                            cleaned_count += 1;
                        }
                        Err(e) => {
                            failed_count += 1;
                            if cli.verbose {
                                eprintln!("  {} Failed: {}: {}", "✗".red(), item.name, e);
                            }
                        }
                    }
                }
            } else {
                match null_e::trash::delete_path(&item.path, delete_method) {
                    Ok(_) => {
                        cleaned_size += item.size;
                        cleaned_count += 1;
                    }
                    Err(e) => {
                        failed_count += 1;
                        if cli.verbose {
                            eprintln!("  {} Failed: {}: {}", "✗".red(), item.name, e);
                        }
                    }
                }
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    println!();
    if cli.dry_run {
        println!(
            "{} Dry run complete. Would free {} from {} items.",
            "✓".green(),
            format_size(cleaned_size).yellow().bold(),
            cleaned_count
        );
    } else {
        println!(
            "{} Cleaned {} from {} items.",
            "✓".green(),
            format_size(cleaned_size).yellow().bold(),
            cleaned_count
        );
    }

    if failed_count > 0 {
        println!("{} {} items failed to clean", "⚠".yellow(), failed_count);
    }

    Ok(())
}

fn run_clean_command_silent(cmd: &str) -> Result<()> {
    use std::process::Command;

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err(DevSweepError::Config("Empty clean command".into()));
    }

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(DevSweepError::CleanFailed {
            path: std::path::PathBuf::from(cmd),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Individual Category Commands
// ═══════════════════════════════════════════════════════════════════════════════

fn cmd_xcode(cli: &Cli, clean: bool) -> Result<()> {
    use null_e::cleaners::xcode::XcodeCleaner;

    println!("{} {}", "🍎 Xcode Cleanup".green().bold(), format!("v{}", null_e::VERSION).dimmed());
    println!();

    let cleaner = match XcodeCleaner::new() {
        Some(c) => c,
        None => {
            println!("  Xcode cleanup is only available on macOS.");
            return Ok(());
        }
    };

    let items = cleaner.detect()?;
    display_and_optionally_clean(&items, cli, clean, "Xcode")
}

fn cmd_android(cli: &Cli, clean: bool) -> Result<()> {
    use null_e::cleaners::android::AndroidCleaner;

    println!("{} {}", "🤖 Android Cleanup".green().bold(), format!("v{}", null_e::VERSION).dimmed());
    println!();

    let cleaner = match AndroidCleaner::new() {
        Some(c) => c,
        None => {
            println!("  Could not initialize Android cleaner.");
            return Ok(());
        }
    };

    let items = cleaner.detect()?;
    display_and_optionally_clean(&items, cli, clean, "Android")
}

fn cmd_docker(cli: &Cli, clean: bool, include_volumes: bool) -> Result<()> {
    use null_e::cleaners::docker::DockerCleaner;

    println!("{} {}", "🐳 Docker Cleanup".green().bold(), format!("v{}", null_e::VERSION).dimmed());
    println!();

    let cleaner = DockerCleaner::new();
    if !cleaner.is_available() {
        println!("  Docker is not available or not running.");
        return Ok(());
    }

    let items = cleaner.detect()?;

    if items.is_empty() {
        println!("  No Docker resources to clean.");
        return Ok(());
    }

    if clean {
        let total_size: u64 = items.iter().map(|i| i.size).sum();
        println!(
            "This will clean {} of Docker resources.",
            format_size(total_size).yellow()
        );

        if include_volumes {
            println!("{} Including volumes - data may be permanently lost!", "⚠️".yellow());
        }

        if !cli.force && !cli.dry_run {
            print!("Continue? [y/N] ");
            use std::io::Write;
            std::io::stdout().flush().unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }

        if cli.dry_run {
            println!("{} Dry run: would free {}", "✓".green(), format_size(total_size).yellow());
        } else {
            match cleaner.clean_all(include_volumes) {
                Ok(freed) => {
                    println!("{} Cleaned {}", "✓".green(), format_size(freed).yellow());
                }
                Err(e) => {
                    println!("{} Failed: {}", "✗".red(), e);
                }
            }
        }
    } else {
        display_and_optionally_clean(&items, cli, false, "Docker")?;
    }

    Ok(())
}

fn cmd_ml(cli: &Cli, clean: bool) -> Result<()> {
    use null_e::cleaners::ml::MlCleaner;

    println!("{} {}", "🤗 ML/AI Cleanup".green().bold(), format!("v{}", null_e::VERSION).dimmed());
    println!();

    let cleaner = match MlCleaner::new() {
        Some(c) => c,
        None => {
            println!("  Could not initialize ML cleaner.");
            return Ok(());
        }
    };

    let items = cleaner.detect()?;
    display_and_optionally_clean(&items, cli, clean, "ML/AI")
}

fn cmd_ide(cli: &Cli, clean: bool) -> Result<()> {
    use null_e::cleaners::ide::IdeCleaner;

    println!("{} {}", "💻 IDE Cleanup".green().bold(), format!("v{}", null_e::VERSION).dimmed());
    println!();

    let cleaner = match IdeCleaner::new() {
        Some(c) => c,
        None => {
            println!("  Could not initialize IDE cleaner.");
            return Ok(());
        }
    };

    let items = cleaner.detect()?;
    display_and_optionally_clean(&items, cli, clean, "IDE")
}

fn cmd_homebrew(cli: &Cli, clean: bool, scrub: bool) -> Result<()> {
    use null_e::cleaners::homebrew::HomebrewCleaner;

    println!("{} {}", "🍺 Homebrew Cleanup".green().bold(), format!("v{}", null_e::VERSION).dimmed());
    println!();

    let cleaner = match HomebrewCleaner::new() {
        Some(c) => c,
        None => {
            println!("  Could not initialize Homebrew cleaner.");
            return Ok(());
        }
    };

    if !cleaner.is_available() {
        println!("  Homebrew is not installed.");
        return Ok(());
    }

    let items = cleaner.detect()?;

    if items.is_empty() {
        println!("  No Homebrew caches found to clean.");
        return Ok(());
    }

    if clean {
        let total_size: u64 = items.iter().map(|i| i.size).sum();
        println!(
            "This will clean {} of Homebrew caches.",
            format_size(total_size).yellow()
        );

        if !cli.force && !cli.dry_run {
            print!("Continue? [y/N] ");
            use std::io::Write;
            std::io::stdout().flush().unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }

        if cli.dry_run {
            println!("{} Dry run: would run 'brew cleanup{}'", "✓".green(), if scrub { " -s" } else { "" });
        } else {
            match cleaner.clean_all(scrub) {
                Ok(_) => {
                    println!("{} Homebrew cleanup complete", "✓".green());
                }
                Err(e) => {
                    println!("{} Failed: {}", "✗".red(), e);
                }
            }
        }
    } else {
        display_and_optionally_clean(&items, cli, false, "Homebrew")?;
    }

    Ok(())
}

fn cmd_ios_deps(cli: &Cli, clean: bool) -> Result<()> {
    use null_e::cleaners::ios_deps::IosDependencyCleaner;

    println!("{} {}", "📱 iOS Dependencies Cleanup".green().bold(), format!("v{}", null_e::VERSION).dimmed());
    println!();

    let cleaner = match IosDependencyCleaner::new() {
        Some(c) => c,
        None => {
            println!("  Could not initialize iOS dependencies cleaner.");
            return Ok(());
        }
    };

    let items = cleaner.detect()?;
    display_and_optionally_clean(&items, cli, clean, "iOS Dependencies")
}

fn cmd_electron(cli: &Cli, clean: bool) -> Result<()> {
    use null_e::cleaners::electron::ElectronCleaner;

    println!("{} {}", "⚡ Electron Apps Cleanup".green().bold(), format!("v{}", null_e::VERSION).dimmed());
    println!();

    let cleaner = match ElectronCleaner::new() {
        Some(c) => c,
        None => {
            println!("  Could not initialize Electron cleaner.");
            return Ok(());
        }
    };

    let items = cleaner.detect()?;
    display_and_optionally_clean(&items, cli, clean, "Electron Apps")
}

fn cmd_gamedev(cli: &Cli, clean: bool) -> Result<()> {
    use null_e::cleaners::gamedev::GameDevCleaner;

    println!("{} {}", "🎮 Game Development Cleanup".green().bold(), format!("v{}", null_e::VERSION).dimmed());
    println!();

    let cleaner = match GameDevCleaner::new() {
        Some(c) => c,
        None => {
            println!("  Could not initialize game dev cleaner.");
            return Ok(());
        }
    };

    let items = cleaner.detect()?;
    display_and_optionally_clean(&items, cli, clean, "Game Development")
}

fn cmd_cloud(cli: &Cli, clean: bool) -> Result<()> {
    use null_e::cleaners::cloud::CloudCliCleaner;

    println!("{} {}", "☁️ Cloud CLI Cleanup".green().bold(), format!("v{}", null_e::VERSION).dimmed());
    println!();

    let cleaner = match CloudCliCleaner::new() {
        Some(c) => c,
        None => {
            println!("  Could not initialize cloud CLI cleaner.");
            return Ok(());
        }
    };

    let items = cleaner.detect()?;
    display_and_optionally_clean(&items, cli, clean, "Cloud CLI")
}

#[cfg(target_os = "macos")]
fn cmd_macos(cli: &Cli, clean: bool) -> Result<()> {
    use null_e::cleaners::macos::MacOsCleaner;

    println!("{} {}", "🍎 macOS System Cleanup".green().bold(), format!("v{}", null_e::VERSION).dimmed());
    println!();

    let cleaner = match MacOsCleaner::new() {
        Some(c) => c,
        None => {
            println!("  Could not initialize macOS cleaner.");
            return Ok(());
        }
    };

    let items = cleaner.detect()?;
    display_and_optionally_clean(&items, cli, clean, "macOS System")
}

fn display_and_optionally_clean(
    items: &[null_e::cleaners::CleanableItem],
    cli: &Cli,
    clean: bool,
    category: &str,
) -> Result<()> {
    if items.is_empty() {
        println!("  No {} items found to clean.", category);
        return Ok(());
    }

    let total_size: u64 = items.iter().map(|i| i.size).sum();

    println!(
        "{} Found {} items with {} total",
        "✓".green(),
        items.len().to_string().cyan(),
        format_size(total_size).yellow().bold()
    );
    println!();

    // Display items
    println!(
        "   {:3} {:<40} {:>12}   {}",
        "".dimmed(),
        "Item".bold(),
        "Size".bold(),
        "Safety".bold()
    );
    println!("   {}", "─".repeat(70).dimmed());

    for (i, item) in items.iter().enumerate() {
        let num = format!("[{}]", i + 1);
        let size_str = format_size(item.size);

        let size_colored = if item.size > 1_000_000_000 {
            size_str.red().bold().to_string()
        } else if item.size > 100_000_000 {
            size_str.yellow().to_string()
        } else {
            size_str.normal().to_string()
        };

        let safety = match item.safe_to_delete {
            null_e::cleaners::SafetyLevel::Safe => "✓ Safe".green().to_string(),
            null_e::cleaners::SafetyLevel::SafeWithCost => "~ Rebuild".yellow().to_string(),
            null_e::cleaners::SafetyLevel::Caution => "! Caution".red().to_string(),
            null_e::cleaners::SafetyLevel::Dangerous => "⚠ Danger".magenta().to_string(),
        };

        let name = if item.name.len() > 38 {
            format!("{}...", &item.name[..35])
        } else {
            item.name.clone()
        };

        println!(
            "   {} {} {:<38} {:>12}   {}",
            num.cyan(),
            item.icon,
            name,
            size_colored,
            safety
        );
    }

    println!("   {}", "─".repeat(70).dimmed());
    println!(
        "   {:3} {:<40} {:>12}",
        "",
        "Total".bold(),
        format_size(total_size).yellow().bold()
    );
    println!();

    if clean {
        clean_items_interactive(items, cli)
    } else {
        println!(
            "{} Use {} to clean interactively",
            "💡".dimmed(),
            format!("null-e{} --clean", category.to_lowercase()).cyan()
        );
        Ok(())
    }
}

// ==================== Analysis Commands ====================

fn cmd_git_analyze(cli: &Cli, fix: bool) -> Result<()> {
    use null_e::analysis::git::GitAnalyzer;

    println!(
        "{} {}",
        "🔍 Git Repository Analysis".green().bold(),
        format!("v{}", null_e::VERSION).dimmed()
    );
    println!();

    let paths = if cli.paths.is_empty() {
        vec![std::env::current_dir()?]
    } else {
        cli.paths.clone()
    };

    let max_depth = cli.max_depth.unwrap_or(10);
    let analyzer = GitAnalyzer::new();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Scanning for git repositories...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut all_recommendations = Vec::new();

    for path in &paths {
        if let Ok(recs) = analyzer.scan(path, max_depth) {
            all_recommendations.extend(recs);
        }
    }

    // Also check for Git LFS cache
    if let Ok(lfs_recs) = analyzer.detect_lfs_cache() {
        all_recommendations.extend(lfs_recs);
    }

    pb.finish_and_clear();

    if all_recommendations.is_empty() {
        println!("  No git optimization opportunities found.");
        println!("  (Repositories under 100MB are not reported)");
        return Ok(());
    }

    // Sort by potential savings
    all_recommendations.sort_by(|a, b| b.potential_savings.cmp(&a.potential_savings));

    let total_savings: u64 = all_recommendations.iter().map(|r| r.potential_savings).sum();

    println!(
        "{} Found {} repositories with potential savings of {}",
        "✓".green(),
        all_recommendations.len().to_string().cyan(),
        format_size(total_savings).yellow().bold()
    );
    println!();

    // Display recommendations
    println!(
        "   {:3} {:<50} {:>12}   {}",
        "".dimmed(),
        "Repository".bold(),
        "Savings".bold(),
        "Action".dimmed()
    );
    println!("   {}", "─".repeat(90).dimmed());

    for (i, rec) in all_recommendations.iter().enumerate() {
        let num = format!("[{}]", i + 1);
        let savings = if rec.potential_savings > 0 {
            format_size(rec.potential_savings)
        } else {
            "-".to_string()
        };

        let risk_symbol = rec.risk.symbol();

        println!(
            "   {} {} {:<48} {:>12}   {}",
            num.cyan(),
            risk_symbol.green(),
            rec.title.chars().take(48).collect::<String>(),
            savings.yellow(),
            rec.description.chars().take(40).collect::<String>().dimmed()
        );

        if cli.verbose {
            if let Some(cmd) = &rec.fix_command {
                println!("       {} {}", "Command:".dimmed(), cmd.cyan());
            }
        }
    }

    println!("   {}", "─".repeat(90).dimmed());
    println!(
        "   {:3} {:<50} {:>12}",
        "",
        "Total Potential Savings".bold(),
        format_size(total_savings).yellow().bold()
    );
    println!();

    if fix && !all_recommendations.is_empty() {
        println!("{} Running git gc on repositories...", "🔧".yellow());
        println!();

        for rec in &all_recommendations {
            if let Some(cmd) = &rec.fix_command {
                if cmd.contains("git gc") {
                    println!("  {} {}", "→".cyan(), rec.path.display());

                    if cli.dry_run {
                        println!("    {} {}", "[DRY RUN]".yellow(), cmd);
                    } else {
                        // Run git gc
                        let output = std::process::Command::new("sh")
                            .args(["-c", cmd])
                            .output();

                        match output {
                            Ok(out) if out.status.success() => {
                                println!("    {} Optimized", "✓".green());
                            }
                            Ok(out) => {
                                let stderr = String::from_utf8_lossy(&out.stderr);
                                println!("    {} Failed: {}", "✗".red(), stderr.trim());
                            }
                            Err(e) => {
                                println!("    {} Error: {}", "✗".red(), e);
                            }
                        }
                    }
                }
            }
        }
    } else if !fix {
        println!(
            "{} Use {} to run git gc on these repositories",
            "💡".dimmed(),
            "null-egit-analyze --fix".cyan()
        );
    }

    Ok(())
}

fn cmd_stale(cli: &Cli, days: u64, clean: bool) -> Result<()> {
    use null_e::analysis::stale::StaleProjectFinder;

    println!(
        "{} {}",
        "📦 Stale Project Finder".green().bold(),
        format!("v{}", null_e::VERSION).dimmed()
    );
    println!("  Looking for projects not touched in {} days...", days);
    println!();

    let paths = if cli.paths.is_empty() {
        vec![std::env::current_dir()?]
    } else {
        cli.paths.clone()
    };

    let max_depth = cli.max_depth.unwrap_or(5);
    let finder = StaleProjectFinder {
        stale_threshold_days: days,
        min_project_size: 50_000_000, // 50MB minimum
    };

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Scanning for stale projects...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut all_recommendations = Vec::new();

    for path in &paths {
        if let Ok(recs) = finder.scan(path, max_depth) {
            all_recommendations.extend(recs);
        }
    }

    pb.finish_and_clear();

    if all_recommendations.is_empty() {
        println!("  No stale projects found (threshold: {} days, min size: 50MB)", days);
        return Ok(());
    }

    // Sort by size (largest first)
    all_recommendations.sort_by(|a, b| {
        let size_a: u64 = a.description.split(' ').filter_map(|s| parse_size(s)).next().unwrap_or(0);
        let size_b: u64 = b.description.split(' ').filter_map(|s| parse_size(s)).next().unwrap_or(0);
        size_b.cmp(&size_a)
    });

    let total_cleanable: u64 = all_recommendations.iter().map(|r| r.potential_savings).sum();

    println!(
        "{} Found {} stale projects with {} in cleanable artifacts",
        "✓".green(),
        all_recommendations.len().to_string().cyan(),
        format_size(total_cleanable).yellow().bold()
    );
    println!();

    // Display recommendations
    println!(
        "   {:3} {:<60} {:>12}",
        "".dimmed(),
        "Project".bold(),
        "Cleanable".bold()
    );
    println!("   {}", "─".repeat(80).dimmed());

    for (i, rec) in all_recommendations.iter().enumerate() {
        let num = format!("[{}]", i + 1);
        let risk_symbol = rec.risk.symbol();
        let cleanable = if rec.potential_savings > 0 {
            format_size(rec.potential_savings)
        } else {
            "archive?".to_string()
        };

        let risk_color = match rec.risk {
            null_e::analysis::RiskLevel::None | null_e::analysis::RiskLevel::Low => cleanable.green().to_string(),
            null_e::analysis::RiskLevel::Medium => cleanable.yellow().to_string(),
            null_e::analysis::RiskLevel::High => cleanable.red().to_string(),
        };

        println!(
            "   {} {} {:<58} {:>12}",
            num.cyan(),
            risk_symbol,
            rec.title.chars().take(58).collect::<String>(),
            risk_color
        );

        if cli.verbose {
            println!("       {}", rec.description.dimmed());
            if let Some(cmd) = &rec.fix_command {
                println!("       {} {}", "Clean:".dimmed(), cmd.cyan());
            }
        }
    }

    println!("   {}", "─".repeat(80).dimmed());
    println!(
        "   {:3} {:<60} {:>12}",
        "",
        "Total Cleanable Artifacts".bold(),
        format_size(total_cleanable).yellow().bold()
    );
    println!();

    if clean && total_cleanable > 0 {
        println!("{} Cleaning build artifacts from stale projects...", "🧹".yellow());
        println!();

        for rec in &all_recommendations {
            if rec.potential_savings > 0 {
                if let Some(cmd) = &rec.fix_command {
                    let project_name = rec.path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    println!("  {} {}", "→".cyan(), project_name);

                    if cli.dry_run {
                        println!("    {} {}", "[DRY RUN]".yellow(), cmd);
                    } else {
                        let output = std::process::Command::new("sh")
                            .args(["-c", cmd])
                            .output();

                        match output {
                            Ok(out) if out.status.success() => {
                                println!("    {} Cleaned {}", "✓".green(), format_size(rec.potential_savings));
                            }
                            Ok(_) | Err(_) => {
                                println!("    {} Failed to clean", "✗".red());
                            }
                        }
                    }
                }
            }
        }
    } else if !clean && total_cleanable > 0 {
        println!(
            "{} Use {} to clean build artifacts",
            "💡".dimmed(),
            format!("null-estale --days {} --clean", days).cyan()
        );
    }

    Ok(())
}

fn cmd_duplicates(cli: &Cli) -> Result<()> {
    use null_e::analysis::duplicates::DuplicateFinder;

    println!(
        "{} {}",
        "🔄 Duplicate Dependency Finder".green().bold(),
        format!("v{}", null_e::VERSION).dimmed()
    );
    println!();

    let paths = if cli.paths.is_empty() {
        vec![std::env::current_dir()?]
    } else {
        cli.paths.clone()
    };

    let max_depth = cli.max_depth.unwrap_or(8);
    let finder = DuplicateFinder::new();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Scanning for duplicate dependencies...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut all_recommendations = Vec::new();

    for path in &paths {
        if let Ok(recs) = finder.scan(path, max_depth) {
            all_recommendations.extend(recs);
        }
    }

    pb.finish_and_clear();

    if all_recommendations.is_empty() {
        println!("  No significant duplicate dependencies found.");
        println!("  (Only duplicates totaling >10MB are reported)");
        return Ok(());
    }

    // Sort by potential savings
    all_recommendations.sort_by(|a, b| b.potential_savings.cmp(&a.potential_savings));

    let total_potential: u64 = all_recommendations.iter().map(|r| r.potential_savings).sum();

    println!(
        "{} Found {} duplicate patterns with {} potential savings",
        "✓".green(),
        all_recommendations.len().to_string().cyan(),
        format_size(total_potential).yellow().bold()
    );
    println!();

    // Display recommendations
    println!(
        "   {:3} {:<55} {:>12}   {}",
        "".dimmed(),
        "Duplicate".bold(),
        "Savings".bold(),
        "Recommendation".dimmed()
    );
    println!("   {}", "─".repeat(95).dimmed());

    for (i, rec) in all_recommendations.iter().enumerate() {
        let num = format!("[{}]", i + 1);
        let savings = if rec.potential_savings > 0 {
            format_size(rec.potential_savings)
        } else {
            "-".to_string()
        };

        println!(
            "   {} {:<55} {:>12}   {}",
            num.cyan(),
            rec.title.chars().take(55).collect::<String>(),
            savings.yellow(),
            rec.fix_command.as_deref().unwrap_or("").chars().take(30).collect::<String>().dimmed()
        );

        if cli.verbose {
            println!("       {}", rec.description.dimmed());
        }
    }

    println!("   {}", "─".repeat(95).dimmed());
    println!(
        "   {:3} {:<55} {:>12}",
        "",
        "Total Potential Savings".bold(),
        format_size(total_potential).yellow().bold()
    );
    println!();

    println!(
        "{} {}",
        "💡".dimmed(),
        "Recommendations:".dimmed()
    );
    println!(
        "   {} Use {} or {} for Node.js deduplication",
        "•".dimmed(),
        "pnpm".cyan(),
        "yarn workspaces".cyan()
    );
    println!(
        "   {} Set {} for Rust shared compilation",
        "•".dimmed(),
        "CARGO_TARGET_DIR=~/.cargo/target".cyan()
    );
    println!(
        "   {} Use {} or {} for Python",
        "•".dimmed(),
        "uv".cyan(),
        "poetry".cyan()
    );

    Ok(())
}

fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();

    let (num_str, multiplier) = if s.ends_with("GB") {
        (&s[..s.len() - 2], 1_000_000_000u64)
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1_000_000u64)
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], 1_000u64)
    } else if s.ends_with('G') {
        (&s[..s.len() - 1], 1_000_000_000u64)
    } else if s.ends_with('M') {
        (&s[..s.len() - 1], 1_000_000u64)
    } else if s.ends_with('K') {
        (&s[..s.len() - 1], 1_000u64)
    } else {
        (s.as_str(), 1u64)
    };

    num_str.trim().parse::<u64>().ok().map(|n| n * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1MB"), Some(1_000_000));
        assert_eq!(parse_size("1M"), Some(1_000_000));
        assert_eq!(parse_size("500KB"), Some(500_000));
        assert_eq!(parse_size("2GB"), Some(2_000_000_000));
        assert_eq!(parse_size("1000"), Some(1000));
        assert_eq!(parse_size("invalid"), None);
    }
}
