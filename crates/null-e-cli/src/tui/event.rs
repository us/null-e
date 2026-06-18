//! Event handling for TUI
//!
//! Handles keyboard and terminal events in a separate thread.

use crossterm::event::{
    self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseEventKind,
};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Terminal events
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Event {
    /// Terminal tick (for animations/updates)
    Tick,
    /// Key press
    Key(KeyEvent),
    /// Mouse event
    Mouse(crossterm::event::MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
}

/// Event handler that polls for terminal events
pub struct EventHandler {
    /// Event receiver
    rx: mpsc::Receiver<Event>,
    /// Event sender (kept for potential future use)
    _tx: mpsc::Sender<Event>,
}

impl EventHandler {
    /// Create a new event handler with specified tick rate
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        let _tx = tx.clone();

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                // Calculate timeout for next tick
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(Duration::ZERO);

                // Poll for events
                if event::poll(timeout).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            // Filter out release events on some terminals
                            if key.kind == crossterm::event::KeyEventKind::Press
                                && tx.send(Event::Key(key)).is_err()
                            {
                                return;
                            }
                        }
                        Ok(CrosstermEvent::Mouse(mouse)) => {
                            if tx.send(Event::Mouse(mouse)).is_err() {
                                return;
                            }
                        }
                        Ok(CrosstermEvent::Resize(w, h))
                            if tx.send(Event::Resize(w, h)).is_err() =>
                        {
                            return;
                        }
                        _ => {}
                    }
                }

                // Send tick event if enough time has passed
                if last_tick.elapsed() >= tick_rate {
                    if tx.send(Event::Tick).is_err() {
                        return;
                    }
                    last_tick = Instant::now();
                }
            }
        });

        Self { rx, _tx }
    }

    /// Get the next event, blocking until one is available
    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.rx.recv()
    }
}

/// Key bindings for the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Quit the application
    Quit,
    /// Move selection up
    Up,
    /// Move selection down
    Down,
    /// Toggle selection of current item
    ToggleSelect,
    /// Expand/collapse current item
    ToggleExpand,
    /// Expand current item
    Expand,
    /// Collapse current item
    Collapse,
    /// Select all items
    SelectAll,
    /// Deselect all items
    DeselectAll,
    /// Delete selected items
    Delete,
    /// Confirm action
    Confirm,
    /// Cancel action
    Cancel,
    /// Show help
    Help,
    /// Start scanning
    Scan,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Go to top
    Top,
    /// Go to bottom
    Bottom,
    /// Search/filter
    Search,
    /// Tab to next category
    NextTab,
    /// Tab to previous category
    PrevTab,
    /// Refresh
    Refresh,
    /// Scroll up (mouse)
    ScrollUp,
    /// Scroll down (mouse)
    ScrollDown,
    /// Go back to menu
    Back,
    /// Toggle permanent delete mode
    TogglePermanent,
    /// No action
    None,
}

impl Action {
    /// Convert a key event to an action
    pub fn from_key(key: KeyEvent) -> Self {
        match key.code {
            // Quit
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Esc => Action::Cancel,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,

            // Navigation
            KeyCode::Up | KeyCode::Char('k') => Action::Up,
            KeyCode::Down | KeyCode::Char('j') => Action::Down,
            KeyCode::PageUp | KeyCode::Char('u')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Action::PageUp
            }
            KeyCode::PageDown | KeyCode::Char('d')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Action::PageDown
            }
            KeyCode::Home | KeyCode::Char('g') if key.modifiers == KeyModifiers::NONE => {
                Action::Top
            }
            KeyCode::End | KeyCode::Char('G') => Action::Bottom,

            // Expand/Collapse with arrow keys
            KeyCode::Right | KeyCode::Char('l') => Action::Expand,
            KeyCode::Left | KeyCode::Char('h') => Action::Collapse,

            // Selection - Enter to select, Space to expand
            KeyCode::Enter => Action::ToggleSelect,
            KeyCode::Char(' ') => Action::ToggleExpand,
            KeyCode::Char('a') => Action::SelectAll,
            KeyCode::Char('A') => Action::DeselectAll,
            KeyCode::Char('u') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::DeselectAll
            }

            // Actions
            KeyCode::Char('d') if !key.modifiers.contains(KeyModifiers::CONTROL) => Action::Delete,
            KeyCode::Delete => Action::Delete,
            KeyCode::Char('y') => Action::Confirm,
            KeyCode::Char('n') => Action::Cancel,
            KeyCode::Char('p') => Action::TogglePermanent,
            KeyCode::Char('?') => Action::Help,
            KeyCode::Char('s') => Action::Scan,
            KeyCode::Char('/') => Action::Search,
            KeyCode::Char('r') | KeyCode::F(5) => Action::Refresh,
            KeyCode::Char('b') | KeyCode::Backspace => Action::Back,

            // Tabs
            KeyCode::Tab => Action::NextTab,
            KeyCode::BackTab => Action::PrevTab,
            KeyCode::Char('1') => Action::None, // Could map to specific tabs
            KeyCode::Char('2') => Action::None,

            _ => Action::None,
        }
    }

    /// Convert a mouse event to an action
    pub fn from_mouse(mouse: &crossterm::event::MouseEvent) -> Self {
        match mouse.kind {
            MouseEventKind::ScrollUp => Action::ScrollUp,
            MouseEventKind::ScrollDown => Action::ScrollDown,
            _ => Action::None,
        }
    }
}
