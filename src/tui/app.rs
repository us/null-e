//! TUI Application state and logic

use crate::core::Project;
use std::collections::HashSet;

/// Main TUI application state
pub struct App {
    /// Current state
    pub state: AppState,
    /// Projects to display
    pub projects: Vec<ProjectEntry>,
    /// Currently selected index
    pub selected: Option<usize>,
    /// Expanded project indices
    pub expanded: HashSet<usize>,
    /// Status message
    pub status_message: Option<String>,
    /// Should quit
    pub should_quit: bool,
}

/// Application state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    /// Ready to scan
    Ready,
    /// Currently scanning
    Scanning,
    /// Showing results
    Results,
    /// Confirming deletion
    Confirming,
    /// Cleaning in progress
    Cleaning,
    /// Error state
    Error(String),
}

/// A project entry in the list
#[derive(Debug, Clone)]
pub struct ProjectEntry {
    pub project: Project,
    pub selected: bool,
}

impl App {
    /// Create a new app
    pub fn new() -> Self {
        Self {
            state: AppState::Ready,
            projects: Vec::new(),
            selected: Some(0),
            expanded: HashSet::new(),
            status_message: None,
            should_quit: false,
        }
    }

    /// Handle key input
    pub fn on_key(&mut self, key: char) {
        match key {
            'q' => self.should_quit = true,
            'j' | 'n' => self.select_next(),
            'k' | 'p' => self.select_prev(),
            ' ' => self.toggle_selected(),
            '\n' => self.toggle_expanded(),
            'a' => self.select_all(),
            'd' => self.state = AppState::Confirming,
            _ => {}
        }
    }

    fn select_next(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        let len = self.projects.len();
        self.selected = Some(self.selected.map_or(0, |i| (i + 1) % len));
    }

    fn select_prev(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        let len = self.projects.len();
        self.selected = Some(self.selected.map_or(0, |i| (i + len - 1) % len));
    }

    fn toggle_selected(&mut self) {
        if let Some(idx) = self.selected {
            if let Some(entry) = self.projects.get_mut(idx) {
                entry.selected = !entry.selected;
            }
        }
    }

    fn toggle_expanded(&mut self) {
        if let Some(idx) = self.selected {
            if self.expanded.contains(&idx) {
                self.expanded.remove(&idx);
            } else {
                self.expanded.insert(idx);
            }
        }
    }

    fn select_all(&mut self) {
        let all_selected = self.projects.iter().all(|p| p.selected);
        for entry in &mut self.projects {
            entry.selected = !all_selected;
        }
    }

    /// Get total selected size
    pub fn selected_size(&self) -> u64 {
        self.projects
            .iter()
            .filter(|p| p.selected)
            .map(|p| p.project.cleanable_size)
            .sum()
    }

    /// Get number of selected items
    pub fn selected_count(&self) -> usize {
        self.projects.iter().filter(|p| p.selected).count()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
