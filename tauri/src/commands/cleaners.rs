use crate::dto::CleanableItemDto;
use crate::state::AppState;

#[tauri::command]
pub async fn detect_cleaners(
    _state: tauri::State<'_, AppState>,
) -> Result<Vec<CleanableItemDto>, String> {
    let result = tokio::task::spawn_blocking(|| {
        let mut items = Vec::new();

        // Collect from all cleaner modules that return Option<Self>
        macro_rules! try_cleaner {
            ($constructor:expr) => {
                if let Some(cleaner) = $constructor {
                    if let Ok(detected) = cleaner.detect() {
                        items.extend(detected.into_iter().map(CleanableItemDto::from));
                    }
                }
            };
        }

        try_cleaner!(null_e_core::cleaners::xcode::XcodeCleaner::new());
        try_cleaner!(null_e_core::cleaners::ide::IdeCleaner::new());
        try_cleaner!(null_e_core::cleaners::ml::MlCleaner::new());
        try_cleaner!(null_e_core::cleaners::android::AndroidCleaner::new());
        try_cleaner!(null_e_core::cleaners::electron::ElectronCleaner::new());
        try_cleaner!(null_e_core::cleaners::cloud::CloudCliCleaner::new());
        try_cleaner!(null_e_core::cleaners::homebrew::HomebrewCleaner::new());
        try_cleaner!(null_e_core::cleaners::gamedev::GameDevCleaner::new());
        try_cleaner!(null_e_core::cleaners::misc::MiscCleaner::new());
        try_cleaner!(null_e_core::cleaners::browsers_test::TestBrowsersCleaner::new());
        try_cleaner!(null_e_core::cleaners::system::SystemCleaner::new());
        try_cleaner!(null_e_core::cleaners::logs::LogsCleaner::new());
        try_cleaner!(null_e_core::cleaners::runtimes::RuntimesCleaner::new());
        try_cleaner!(null_e_core::cleaners::macos::MacOsCleaner::new());
        try_cleaner!(null_e_core::cleaners::ios_deps::IosDependencyCleaner::new());

        // Docker has a different constructor (always returns Self, not Option<Self>)
        let docker = null_e_core::cleaners::docker::DockerCleaner::new();
        if docker.is_available() {
            if let Ok(detected) = docker.detect() {
                items.extend(detected.into_iter().map(CleanableItemDto::from));
            }
        }

        Ok::<Vec<CleanableItemDto>, String>(items)
    })
    .await
    .map_err(|e| e.to_string())?;

    result
}
