use crate::dto::CleanableItemDto;
use crate::state::AppState;
use null_e_core::cleaners::CleanableItem;

/// Collect cleanable system items from every cleaner module.
///
/// Returns `(items, skipped)` where `skipped` names any cleaner whose `detect()` errored — these
/// were previously dropped **silently** (`if let Ok(..)`), hiding scan gaps. Surfacing them is the
/// fix for "the scan misses things and I can't tell why".
///
/// Shared by [`detect_cleaners`] (UI listing) and [`run_system_action`] (server-side re-resolution
/// of an item so we never trust a command/path supplied by the frontend).
fn collect_cleanable_items() -> (Vec<CleanableItem>, Vec<String>) {
    use null_e_core::cleaners::rayon::prelude::*;

    // Each entry runs one cleaner and returns `(detected items, optional skip reason)`. They are
    // boxed into a uniform closure type so they can be fanned out across rayon's thread pool. This
    // is safe inside `spawn_blocking`: STEP-1 dedup means the heavy cleaners (System vs IDE/ML) walk
    // DISJOINT subtrees, so parallel detection no longer races on shared `~/Library/Caches` dirs.
    type CleanerJob = Box<dyn Fn() -> (Vec<CleanableItem>, Option<String>) + Send + Sync>;

    // Helper: build a job for an `Option<Self>`-constructed cleaner, preserving the per-cleaner
    // error handling (errors become a `skipped` entry; absent cleaners contribute nothing).
    macro_rules! cleaner_job {
        ($name:expr, $constructor:expr) => {
            Box::new(move || match $constructor {
                Some(cleaner) => match cleaner.detect() {
                    Ok(detected) => (detected, None),
                    Err(e) => (Vec::new(), Some(format!("{}: {}", $name, e))),
                },
                None => (Vec::new(), None),
            }) as CleanerJob
        };
    }

    let jobs: Vec<CleanerJob> = vec![
        cleaner_job!("Xcode", null_e_core::cleaners::xcode::XcodeCleaner::new()),
        cleaner_job!("IDE", null_e_core::cleaners::ide::IdeCleaner::new()),
        cleaner_job!("ML", null_e_core::cleaners::ml::MlCleaner::new()),
        cleaner_job!(
            "Android",
            null_e_core::cleaners::android::AndroidCleaner::new()
        ),
        cleaner_job!(
            "Electron",
            null_e_core::cleaners::electron::ElectronCleaner::new()
        ),
        cleaner_job!(
            "Cloud CLI",
            null_e_core::cleaners::cloud::CloudCliCleaner::new()
        ),
        cleaner_job!(
            "Homebrew",
            null_e_core::cleaners::homebrew::HomebrewCleaner::new()
        ),
        cleaner_job!(
            "Game Dev",
            null_e_core::cleaners::gamedev::GameDevCleaner::new()
        ),
        cleaner_job!("Misc", null_e_core::cleaners::misc::MiscCleaner::new()),
        cleaner_job!(
            "Browsers (test)",
            null_e_core::cleaners::browsers_test::TestBrowsersCleaner::new()
        ),
        cleaner_job!(
            "System",
            null_e_core::cleaners::system::SystemCleaner::new()
        ),
        cleaner_job!("Logs", null_e_core::cleaners::logs::LogsCleaner::new()),
        cleaner_job!(
            "Runtimes",
            null_e_core::cleaners::runtimes::RuntimesCleaner::new()
        ),
        cleaner_job!("macOS", null_e_core::cleaners::macos::MacOsCleaner::new()),
        cleaner_job!(
            "iOS deps",
            null_e_core::cleaners::ios_deps::IosDependencyCleaner::new()
        ),
        // Docker has a different constructor (always returns Self) and an availability guard that
        // must be kept: skip detection entirely when the Docker daemon/CLI isn't available.
        Box::new(|| {
            let docker = null_e_core::cleaners::docker::DockerCleaner::new();
            if !docker.is_available() {
                return (Vec::new(), None);
            }
            match docker.detect() {
                Ok(detected) => (detected, None),
                Err(e) => (Vec::new(), Some(format!("Docker: {}", e))),
            }
        }) as CleanerJob,
    ];

    // Run all cleaners in parallel and merge results. No item is lost or duplicated: each job owns
    // a disjoint set of paths and we concatenate every job's output.
    let (mut items, skipped): (Vec<CleanableItem>, Vec<String>) = jobs
        .into_par_iter()
        .map(|job| job())
        .collect::<Vec<_>>()
        .into_iter()
        .fold(
            (Vec::new(), Vec::new()),
            |(mut items, mut skipped), (detected, skip)| {
                items.extend(detected);
                if let Some(reason) = skip {
                    skipped.push(reason);
                }
                (items, skipped)
            },
        );

    // Deterministic order regardless of parallel completion order: biggest first, with stable
    // tiebreakers so repeated scans (and the server-side re-resolution in `run_system_action`)
    // produce identical orderings. The UI re-sorts by size anyway, so this never changes behaviour.
    items.sort_by(|a, b| {
        b.size
            .cmp(&a.size)
            .then_with(|| a.category.cmp(&b.category))
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.name.cmp(&b.name))
    });

    (items, skipped)
}

#[tauri::command]
pub async fn detect_cleaners(
    _state: tauri::State<'_, AppState>,
) -> Result<crate::dto::DetectCleanersResultDto, String> {
    tokio::task::spawn_blocking(|| {
        let (items, skipped) = collect_cleanable_items();
        Ok::<crate::dto::DetectCleanersResultDto, String>(crate::dto::DetectCleanersResultDto {
            items: items.into_iter().map(CleanableItemDto::from).collect(),
            skipped,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Allowed command binaries for system "official command" actions. Defense-in-depth: even though
/// we run via `Command` (no shell, so no injection), the command string is re-resolved server-side
/// from our own detection — never taken from the frontend — and the binary must be on this list.
const ALLOWED_ACTION_BINARIES: &[&str] = &[
    "tmutil",
    "npm",
    "yarn",
    "pnpm",
    "bun",
    "brew",
    "docker",
    "go",
    "cargo",
    "pip",
    "pip3",
    "pod",
    "gem",
    "conda",
    "gradle",
    "xcrun",
    "dscacheutil",
];

fn command_is_allowed(cmd: &str) -> bool {
    cmd.split_whitespace()
        .next()
        .map(|bin| ALLOWED_ACTION_BINARIES.contains(&bin))
        .unwrap_or(false)
}

/// Result of running a typed system action.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemActionResultDto {
    pub success: bool,
    pub bytes_freed: u64,
    pub message: String,
}

/// Run a system cleanup action, resolved and validated **entirely server-side**.
///
/// The UI sends only `{path, name}` identifying which detected item to act on. The backend
/// re-detects the item, reads *its own* `clean_command`, validates it against an allow-list, and
/// either runs that command (measuring the real free-space delta) or — if the item has no command —
/// performs a guarded permanent delete (refusing protected aggregate locations). This is the
/// backend-owned action model: no argv or command string is ever trusted from the frontend.
#[tauri::command]
pub async fn run_system_action(
    _state: tauri::State<'_, AppState>,
    path: String,
    name: String,
) -> Result<SystemActionResultDto, String> {
    tokio::task::spawn_blocking(move || {
        let (items, _skipped) = collect_cleanable_items();
        let item = items
            .into_iter()
            .find(|i| i.path.to_string_lossy() == path && i.name == name)
            .ok_or_else(|| format!("System item not found: {} ({})", name, path))?;

        let target = item.path.clone();

        // Branch 1: item has an official clean command → validate + run it, measure real delta.
        if let Some(cmd) = item.clean_command.clone() {
            if !command_is_allowed(&cmd) {
                return Err(format!("Refused: command not allowed ({})", cmd));
            }
            let root = std::path::Path::new("/");
            let before = null_e_core::fsutil::available_bytes(root).unwrap_or(0);
            run_allowed_command(&cmd)?;
            let after = null_e_core::fsutil::available_bytes(root).unwrap_or(before);
            let bytes_freed = after.saturating_sub(before);
            return Ok(SystemActionResultDto {
                success: true,
                bytes_freed,
                message: format!("Ran: {}", cmd),
            });
        }

        // Branch 2: plain path delete — refuse protected aggregates, then permanently delete.
        if null_e_core::fsutil::is_protected_aggregate(&target) {
            return Err(format!(
                "Refused: {} is a protected location",
                target.display()
            ));
        }
        match null_e_core::trash::delete_path_detailed(
            &target,
            null_e_core::trash::DeleteMethod::Permanent,
        ) {
            Ok(outcome) => Ok(SystemActionResultDto {
                success: true,
                bytes_freed: outcome.freed,
                message: "Deleted".to_string(),
            }),
            Err(e) => Err(e.to_string()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Run a validated command with no shell (so no injection is possible).
fn run_allowed_command(cmd: &str) -> Result<(), String> {
    use std::process::Command;
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }
    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
