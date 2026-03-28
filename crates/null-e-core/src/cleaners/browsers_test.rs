//! Testing browser cleanup module
//!
//! Handles cleanup of browser binaries used for testing:
//! - Playwright browsers
//! - Cypress browser cache
//! - Puppeteer browsers
//! - Selenium WebDriver cache

use super::{calculate_dir_size, get_mtime, CleanableItem, SafetyLevel};
use crate::error::Result;
use std::borrow::Cow;
use std::path::PathBuf;

/// Testing browsers cleaner
pub struct TestBrowsersCleaner {
    home: PathBuf,
}

impl TestBrowsersCleaner {
    /// Create a new test browsers cleaner
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        Some(Self { home })
    }

    /// Detect all cleanable items
    pub fn detect(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Playwright
        items.extend(self.detect_playwright()?);

        // Cypress
        items.extend(self.detect_cypress()?);

        // Puppeteer
        items.extend(self.detect_puppeteer()?);

        // Selenium
        items.extend(self.detect_selenium()?);

        // Chrome for Testing
        items.extend(self.detect_chrome_testing()?);

        Ok(items)
    }

    /// Detect Playwright browsers
    fn detect_playwright(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Playwright stores browsers in different locations per OS
        let playwright_paths = [
            // macOS
            self.home.join("Library/Caches/ms-playwright"),
            // Linux
            self.home.join(".cache/ms-playwright"),
            // Windows
            self.home.join("AppData/Local/ms-playwright"),
            // Alternative Linux
            self.home.join(".local/share/ms-playwright"),
        ];

        for pw_path in playwright_paths {
            if !pw_path.exists() {
                continue;
            }

            // Check for browser directories
            if let Ok(entries) = std::fs::read_dir(&pw_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    // Determine browser type
                    let (browser_name, icon) = if name.contains("chromium") {
                        ("Chromium", "🌐")
                    } else if name.contains("firefox") {
                        ("Firefox", "🦊")
                    } else if name.contains("webkit") {
                        ("WebKit", "🧭")
                    } else {
                        continue;
                    };

                    let (size, file_count) = calculate_dir_size(&path)?;
                    if size < 50_000_000 {
                        // Skip small
                        continue;
                    }

                    items.push(CleanableItem {
                        name: format!("Playwright {}", browser_name),
                        category: "Testing".to_string(),
                        subcategory: "Playwright".to_string(),
                        icon,
                        path,
                        size,
                        file_count: Some(file_count),
                        last_modified: get_mtime(&entry.path()),
                        description: Cow::Borrowed(
                            "Browser binary for Playwright testing. Will be re-downloaded.",
                        ),
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: Some("npx playwright install --clean".to_string()),
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect Cypress cache
    fn detect_cypress(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Cypress cache locations
        let cypress_paths = [
            // macOS/Linux
            self.home.join(".cache/Cypress"),
            // Windows
            self.home.join("AppData/Local/Cypress/Cache"),
            // Alternative
            self.home.join("Library/Caches/Cypress"),
        ];

        for cypress_path in cypress_paths {
            if !cypress_path.exists() {
                continue;
            }

            // Check for version directories
            if let Ok(entries) = std::fs::read_dir(&cypress_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let version = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let (size, file_count) = calculate_dir_size(&path)?;
                    if size < 100_000_000 {
                        // Cypress is usually 400MB+
                        continue;
                    }

                    items.push(CleanableItem {
                        name: format!("Cypress v{}", version),
                        category: "Testing".to_string(),
                        subcategory: "Cypress".to_string(),
                        icon: "🌲",
                        path,
                        size,
                        file_count: Some(file_count),
                        last_modified: get_mtime(&entry.path()),
                        description: Cow::Borrowed(
                            "Cypress test runner. Will be re-downloaded when needed.",
                        ),
                        safe_to_delete: SafetyLevel::SafeWithCost,
                        clean_command: Some("npx cypress cache clear".to_string()),
                    });
                }
            }
        }

        Ok(items)
    }

    /// Detect Puppeteer browsers
    fn detect_puppeteer(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Puppeteer cache locations
        let puppeteer_paths = [
            // Default location
            self.home.join(".cache/puppeteer"),
            // Windows
            self.home.join("AppData/Local/puppeteer"),
            // macOS
            self.home.join("Library/Caches/puppeteer"),
        ];

        for pup_path in puppeteer_paths {
            if !pup_path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&pup_path)?;
            if size < 100_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: "Puppeteer Browsers".to_string(),
                category: "Testing".to_string(),
                subcategory: "Puppeteer".to_string(),
                icon: "🎭",
                path: pup_path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed(
                    "Puppeteer Chrome/Chromium binary. Will be re-downloaded.",
                ),
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect Selenium WebDriver cache
    fn detect_selenium(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Selenium WebDriver cache locations
        let selenium_paths = [
            // selenium-manager cache
            self.home.join(".cache/selenium"),
            // WebDriver Manager (Python)
            self.home.join(".wdm"),
            // Windows
            self.home.join("AppData/Local/selenium"),
        ];

        for sel_path in selenium_paths {
            if !sel_path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&sel_path)?;
            if size < 50_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: "Selenium WebDriver".to_string(),
                category: "Testing".to_string(),
                subcategory: "Selenium".to_string(),
                icon: "🔧",
                path: sel_path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed("Selenium browser drivers. Will be re-downloaded."),
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        Ok(items)
    }

    /// Detect Chrome for Testing
    fn detect_chrome_testing(&self) -> Result<Vec<CleanableItem>> {
        let mut items = Vec::new();

        // Chrome for Testing (new official test binaries)
        let chrome_paths = [
            self.home.join(".cache/chrome-for-testing"),
            self.home.join("AppData/Local/chrome-for-testing"),
        ];

        for chrome_path in chrome_paths {
            if !chrome_path.exists() {
                continue;
            }

            let (size, file_count) = calculate_dir_size(&chrome_path)?;
            if size < 100_000_000 {
                continue;
            }

            items.push(CleanableItem {
                name: "Chrome for Testing".to_string(),
                category: "Testing".to_string(),
                subcategory: "Chrome".to_string(),
                icon: "🌐",
                path: chrome_path,
                size,
                file_count: Some(file_count),
                last_modified: None,
                description: Cow::Borrowed("Chrome for Testing binaries. Will be re-downloaded."),
                safe_to_delete: SafetyLevel::SafeWithCost,
                clean_command: None,
            });
        }

        Ok(items)
    }
}
