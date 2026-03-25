//! Terminal User Interface
//!
//! Interactive TUI for browsing and cleaning artifacts.
//!
//! This module provides a full-screen terminal interface using Ratatui.

pub mod app;
pub mod event;
pub mod ui;

pub use app::{App, AppState, ProjectEntry};
pub use event::{Action, Event, EventHandler};

use crate::error::Result;
use crate::trash::{delete_path, DeleteMethod};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

/// Restore terminal to normal mode (used by both normal exit and panic hook)
fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
}

/// Run the TUI application
pub fn run(paths: Vec<PathBuf>) -> Result<()> {
    // Install panic hook to restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        restore_terminal();
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(paths);

    // Create event handler with faster tick rate for smooth animations
    let events = EventHandler::new(Duration::from_millis(50));

    // Main loop
    let result = run_app(&mut terminal, &mut app, &events);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Main application loop
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events: &EventHandler,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| ui::render(app, frame))?;

        // Handle events
        let event = events
            .next()
            .map_err(|e| crate::error::NullEError::Other(e.to_string()))?;
        match event {
            Event::Key(key) => {
                // Handle search mode separately
                if app.is_searching {
                    match key.code {
                        KeyCode::Esc => app.end_search(),
                        KeyCode::Enter => app.end_search(),
                        KeyCode::Backspace => app.search_pop(),
                        KeyCode::Char(c) => app.search_push(c),
                        _ => {}
                    }
                    continue;
                }

                // Handle help popup
                if app.show_help {
                    app.show_help = false;
                    continue;
                }

                // Convert key to action
                let action = Action::from_key(key);

                // Handle state-specific actions
                match app.state {
                    AppState::Scanning => {
                        // Only allow quit during scanning
                        if matches!(action, Action::Quit) {
                            app.should_quit = true;
                        }
                    }
                    AppState::Confirming => match action {
                        Action::Confirm => {
                            // Start deletion - actual delete happens on next tick
                            app.start_delete();
                        }
                        Action::TogglePermanent => {
                            app.toggle_permanent_delete();
                        }
                        Action::Cancel | Action::Quit => {
                            app.cancel_delete();
                        }
                        _ => {}
                    },
                    AppState::Ready => match action {
                        // Menu navigation
                        Action::Quit => {
                            app.should_quit = true;
                        }
                        Action::Up | Action::ScrollUp => app.menu_up(),
                        Action::Down | Action::ScrollDown => app.menu_down(),
                        Action::ToggleSelect | Action::Scan | Action::Confirm => {
                            // Enter key or 's' key starts scan
                            app.start_scan();
                        }
                        Action::Help => app.toggle_help(),
                        _ => {}
                    },
                    AppState::Results
                    | AppState::CacheResults
                    | AppState::CleanerResults
                    | AppState::Error(_) => match action {
                        Action::Quit => {
                            app.should_quit = true;
                        }
                        Action::Up => app.select_up(),
                        Action::Down => app.select_down(),
                        Action::PageUp => app.page_up(10),
                        Action::PageDown => app.page_down(10),
                        Action::Top => app.go_top(),
                        Action::Bottom => app.go_bottom(),
                        Action::ToggleSelect => app.toggle_select(),
                        Action::ToggleExpand => app.toggle_expand(),
                        Action::Expand => app.expand(),
                        Action::Collapse => app.collapse(),
                        Action::SelectAll => app.select_all(),
                        Action::DeselectAll => app.deselect_all(),
                        Action::Delete => app.request_delete(),
                        Action::Help => app.toggle_help(),
                        Action::Scan | Action::Refresh => {
                            app.start_scan();
                        }
                        Action::Search => app.start_search(),
                        Action::NextTab => app.next_tab(),
                        Action::PrevTab => app.prev_tab(),
                        Action::ScrollUp => app.scroll_up(),
                        Action::ScrollDown => app.scroll_down(),
                        Action::Back => {
                            app.go_back();
                        }
                        Action::Cancel => {
                            // Esc - go back to menu (or clear search if active)
                            if !app.search_query.is_empty() {
                                app.search_query.clear();
                                app.filter_by_tab();
                            } else {
                                app.go_back();
                            }
                        }
                        _ => {}
                    },
                    AppState::Cleaning => {
                        // Allow quit during cleaning
                        if matches!(action, Action::Quit | Action::Cancel) {
                            app.pending_delete_items.clear();
                            app.cancel_delete();
                            app.status_message = Some("Cleaning cancelled".to_string());
                            if matches!(action, Action::Quit) {
                                app.should_quit = true;
                            }
                        }
                    }
                }
            }
            Event::Tick => {
                // Always tick animation for smooth UI
                app.tick_animation();

                // Check for scan updates on every tick
                if app.state == AppState::Scanning {
                    app.check_scan_progress();
                }

                // Process pending deletions (runs after UI has rendered Cleaning state)
                if app.has_pending_delete() {
                    let items = app.take_pending_delete_items();
                    let permanent = app.permanent_delete;

                    // Wrap in catch_unwind to handle panics gracefully
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        delete_items(&items, permanent)
                    }));

                    match result {
                        Ok((success, failed, freed)) => {
                            app.deletion_complete(success, failed, freed);
                        }
                        Err(_) => {
                            // Panic during deletion - recover gracefully
                            app.deletion_complete(0, items.len(), 0);
                            app.status_message = Some("Error during deletion!".to_string());
                        }
                    }
                }
            }
            Event::Resize(_, _) => {
                // Terminal will redraw on next iteration
            }
            Event::Mouse(mouse) => {
                // Handle mouse events (scroll wheel)
                let action = Action::from_mouse(&mouse);
                match action {
                    Action::ScrollUp => {
                        app.select_up();
                        app.select_up();
                        app.select_up();
                    }
                    Action::ScrollDown => {
                        app.select_down();
                        app.select_down();
                        app.select_down();
                    }
                    _ => {}
                }
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Delete items and return (success_count, fail_count, bytes_freed)
/// Items are tuples of (path, optional clean_command)
/// If clean_command is Some, run that command instead of deleting the path
fn delete_items(items: &[(PathBuf, Option<String>)], permanent: bool) -> (usize, usize, u64) {
    let mut success = 0;
    let mut failed = 0;
    let mut freed = 0u64;

    let method = if permanent {
        DeleteMethod::Permanent
    } else {
        DeleteMethod::Trash
    };

    for (path, clean_command) in items {
        // Get size before deletion
        let size = if path.is_dir() {
            dir_size(path)
        } else {
            std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        };

        // If there's a clean_command, run it instead of deleting the path
        let result = if let Some(cmd) = clean_command {
            // Run the clean command (e.g., "docker rmi abc123")
            run_clean_command(cmd)
        } else {
            // Delete using selected method
            delete_path(path, method).map(|_| ())
        };

        match result {
            Ok(_) => {
                success += 1;
                freed += size;
            }
            Err(_) => {
                failed += 1;
            }
        }
    }

    (success, failed, freed)
}

/// Run a shell command for cleaning (Docker, etc.)
fn run_clean_command(cmd: &str) -> crate::error::Result<()> {
    use std::process::Command;

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd]).output()
    } else {
        Command::new("sh").args(["-c", cmd]).output()
    };

    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(crate::error::NullEError::Other(
            String::from_utf8_lossy(&out.stderr).to_string(),
        )),
        Err(e) => Err(crate::error::NullEError::Other(e.to_string())),
    }
}

/// Calculate directory size
fn dir_size(path: &PathBuf) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}
