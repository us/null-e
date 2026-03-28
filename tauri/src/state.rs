use null_e_core::core::{CleanProgress, ScanProgress};
use null_e_core::plugins::PluginRegistry;
use std::sync::{Arc, Mutex};

use crate::dto::ScanResultDto;

pub struct AppState {
    pub scan_progress: Mutex<Option<Arc<ScanProgress>>>,
    pub clean_progress: Mutex<Option<Arc<CleanProgress>>>,
    pub last_scan_result: Mutex<Option<ScanResultDto>>,
    pub registry: Arc<PluginRegistry>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            scan_progress: Mutex::new(None),
            clean_progress: Mutex::new(None),
            last_scan_result: Mutex::new(None),
            registry: Arc::new(PluginRegistry::with_builtins()),
        }
    }
}
