//! TUI rendering with Ratatui
//!
//! This module handles all the UI rendering for the TUI.

use super::app::{format_size, App, AppState, ScanMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Tabs,
    },
    Frame,
};

/// Robot banner for the header
const ROBOT_SMALL: &str = "🤖";

/// Main UI rendering function
pub fn render(app: &mut App, frame: &mut Frame) {
    let size = frame.area();

    // Create main layout - hide tabs on ready screen
    let show_tabs = !matches!(app.state, AppState::Ready);

    let chunks = if show_tabs {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Tabs
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Status bar
            ])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Content (no tabs)
                Constraint::Length(3), // Status bar
            ])
            .split(size)
    };

    // Render header
    render_header(app, frame, chunks[0]);

    // Render main content based on state
    if show_tabs {
        // Render tabs for results screens
        render_tabs(app, frame, chunks[1]);

        match &app.state {
            AppState::Scanning => render_scanning_screen(app, frame, chunks[2]),
            AppState::Results => render_results(app, frame, chunks[2]),
            AppState::CacheResults => render_cache_results(app, frame, chunks[2]),
            AppState::CleanerResults => render_cleaner_results(app, frame, chunks[2]),
            AppState::Confirming => {
                // Show appropriate results behind dialog
                if !app.projects.is_empty() {
                    render_results(app, frame, chunks[2]);
                } else if !app.caches.is_empty() {
                    render_cache_results(app, frame, chunks[2]);
                } else {
                    render_cleaner_results(app, frame, chunks[2]);
                }
                render_confirm_dialog(app, frame, size);
            }
            AppState::Cleaning => render_cleaning_screen(app, frame, chunks[2]),
            AppState::Error(msg) => render_error_screen(msg, frame, chunks[2]),
            _ => {}
        }

        // Render status bar
        render_status_bar(app, frame, chunks[3]);
    } else {
        // Ready screen - no tabs
        render_ready_screen(app, frame, chunks[1]);
        render_status_bar(app, frame, chunks[2]);
    }

    // Render help popup if active
    if app.show_help {
        render_help_popup(frame, size);
    }
}

/// Render the header
fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let title = format!(" {} null-e v{} ", ROBOT_SMALL, env!("CARGO_PKG_VERSION"));

    let total_info = if app.total_size > 0 {
        format!("Total: {} ", format_size(app.total_size))
    } else {
        String::new()
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            title,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - The friendly disk cleanup robot"),
        Span::raw(" ".repeat(area.width.saturating_sub(60) as usize)),
        Span::styled(total_info, Style::default().fg(Color::Yellow)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );

    frame.render_widget(header, area);
}

/// Render the tabs
fn render_tabs(app: &App, frame: &mut Frame, area: Rect) {
    let titles: Vec<Line> = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let style = if i == app.current_tab {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Line::from(Span::styled(format!(" {} ", t), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Categories (Tab/Shift+Tab) "),
        )
        .select(app.current_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow));

    frame.render_widget(tabs, area);
}

/// Render the ready/welcome screen with scan mode menu
fn render_ready_screen(app: &App, frame: &mut Frame, area: Rect) {
    // Split into robot area and menu area
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Robot
            Constraint::Percentage(60), // Menu
        ])
        .split(area);

    // Robot ASCII art - each line separate
    let robot_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "       .---.      ",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "      |o   o|     ",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "      |  ^  |     ",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "      | === |     ",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "      `-----'     ",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "       /| |\\      ",
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Welcome to null-e!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Send your dev cruft",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "    to /dev/null",
            Style::default().fg(Color::Gray),
        )),
    ];

    let robot = Paragraph::new(robot_lines)
        .block(Block::default().borders(Borders::ALL).title(" 🤖 "))
        .alignment(Alignment::Center);

    frame.render_widget(robot, chunks[0]);

    // Menu items
    let modes = ScanMode::all_modes();
    let menu_items: Vec<ListItem> = modes
        .iter()
        .enumerate()
        .map(|(i, mode)| {
            let is_selected = i == app.menu_index;

            let marker = if is_selected { "▸ " } else { "  " };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let desc_style = if is_selected {
                Style::default().fg(Color::Gray)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let lines = vec![
                Line::from(vec![
                    Span::styled(marker, style),
                    Span::styled(mode.icon(), Style::default()),
                    Span::raw(" "),
                    Span::styled(mode.name(), style),
                ]),
                Line::from(vec![
                    Span::raw("     "),
                    Span::styled(mode.description(), desc_style),
                ]),
            ];

            ListItem::new(lines)
        })
        .collect();

    let menu = List::new(menu_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Scan Mode (j/k + Enter) ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(menu, chunks[1]);
}

/// Render the scanning screen
fn render_scanning_screen(app: &App, frame: &mut Frame, area: Rect) {
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner_idx = app.anim_frame % spinner_frames.len();

    // Progress bar animation
    let bar_width = 30;
    let progress_pos = (app.anim_frame / 2) % (bar_width * 2);
    let progress_bar: String = (0..bar_width)
        .map(|i| {
            let pos = if progress_pos < bar_width {
                progress_pos
            } else {
                bar_width * 2 - progress_pos - 1
            };
            if i == pos || i == pos.saturating_sub(1) || i == pos + 1 {
                '█'
            } else {
                '░'
            }
        })
        .collect();

    let text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "       .---.      ",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "      |o   o|     ",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "      |  ^  |     ",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "      | === |     ",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "      `-----'     ",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "       /| |\\      ",
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {} {}  ", spinner_frames[spinner_idx], app.scan_message),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  [{}]  ", progress_bar),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  Scanning: {}  ", app.scan_mode.name()),
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'q' to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🔍 Scanning "),
        )
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render the project results list
fn render_results(app: &mut App, frame: &mut Frame, area: Rect) {
    // Calculate viewport height for scrolling
    let viewport_height = area.height.saturating_sub(2) as usize;
    app.ensure_visible_with_height(viewport_height);

    let visible_projects = app.visible_projects();

    if visible_projects.is_empty() {
        let text = if app.projects.is_empty() {
            "No projects found. Press 'b' to go back."
        } else {
            "No projects match the current filter."
        };

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Projects "))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
        return;
    }

    // Build list items
    let items: Vec<ListItem> = visible_projects
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(viewport_height)
        .map(|(i, entry)| {
            let is_selected = i == app.selected;
            let is_expanded = app.is_expanded(i);

            // Checkbox
            let checkbox = if entry.selected { "☑" } else { "☐" };

            // Expand indicator
            let expand = if is_expanded { "▾" } else { "▸" };

            // Project icon based on kind
            let icon = entry.project.kind.icon();

            // Project name and size
            let name = &entry.project.name;
            let size = format_size(entry.project.cleanable_size);

            // Age info
            let age = format_age(&entry.project);

            // Build the line
            let line_style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let checkbox_style = if entry.selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            };

            let spans = vec![
                Span::styled(format!(" {} ", checkbox), checkbox_style),
                Span::styled(format!("{} ", expand), Style::default().fg(Color::Gray)),
                Span::styled(format!("{} ", icon), Style::default()),
                Span::styled(
                    format!("{:<30}", truncate(name, 30)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(format!("{:>10}", size), Style::default().fg(Color::Yellow)),
                Span::styled(format!("  {}", age), Style::default().fg(Color::Gray)),
            ];

            let mut lines = vec![Line::from(spans).style(line_style)];

            // Add expanded details
            if is_expanded {
                // Show path
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled("Path: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        entry.project.root.to_string_lossy().to_string(),
                        Style::default().fg(Color::Blue),
                    ),
                ]));

                // Show artifacts
                for artifact in &entry.project.artifacts {
                    lines.push(Line::from(vec![
                        Span::raw("      └── "),
                        Span::styled(
                            artifact.name().to_string(),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::raw(" "),
                        Span::styled(format_size(artifact.size), Style::default().fg(Color::Gray)),
                    ]));
                }
            }

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Projects ({}/{}) Esc=back ",
            app.visible_count(),
            app.projects.len()
        )))
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(list, area);

    // Render scrollbar if needed
    if visible_projects.len() > viewport_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state =
            ScrollbarState::new(visible_projects.len()).position(app.scroll_offset);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

/// Render cache results
fn render_cache_results(app: &mut App, frame: &mut Frame, area: Rect) {
    let viewport_height = area.height.saturating_sub(2) as usize;
    app.ensure_visible_with_height(viewport_height);

    if app.caches.is_empty() {
        let paragraph = Paragraph::new("No caches found. Press 'b' to go back.")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Global Caches "),
            )
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .caches
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(viewport_height)
        .map(|(i, cache)| {
            let is_selected = i == app.selected;
            let checkbox = if cache.selected { "☑" } else { "☐" };

            let line_style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let checkbox_style = if cache.selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            };

            let lines = vec![
                Line::from(vec![
                    Span::styled(format!(" {} ", checkbox), checkbox_style),
                    Span::styled(&cache.icon, Style::default()),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:<25}", truncate(&cache.name, 25)),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("{:>12}", format_size(cache.size)),
                        Style::default().fg(Color::Yellow),
                    ),
                ])
                .style(line_style),
                Line::from(vec![
                    Span::raw("      "),
                    Span::styled(&cache.description, Style::default().fg(Color::DarkGray)),
                ]),
            ];

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Global Caches ({}) Esc=back ", app.caches.len())),
    );

    frame.render_widget(list, area);
}

/// Render cleaner results (Xcode, Docker, IDE, ML)
fn render_cleaner_results(app: &mut App, frame: &mut Frame, area: Rect) {
    let viewport_height = area.height.saturating_sub(2) as usize;
    app.ensure_visible_with_height(viewport_height);

    if app.cleaners.is_empty() {
        let paragraph = Paragraph::new("No items found. Press 'b' to go back.")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", app.scan_mode.name())),
            )
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .cleaners
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(viewport_height)
        .map(|(i, item)| {
            let is_selected = i == app.selected;
            let checkbox = if item.selected { "☑" } else { "☐" };

            let line_style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let checkbox_style = if item.selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            };

            let lines = vec![
                Line::from(vec![
                    Span::styled(format!(" {} ", checkbox), checkbox_style),
                    Span::styled(&item.icon, Style::default()),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:<30}", truncate(&item.name, 30)),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("{:>12}", format_size(item.size)),
                        Style::default().fg(Color::Yellow),
                    ),
                ])
                .style(line_style),
                Line::from(vec![
                    Span::raw("      "),
                    Span::styled(&item.category, Style::default().fg(Color::Cyan)),
                    Span::raw(" - "),
                    Span::styled(
                        truncate(&item.path.to_string_lossy(), 50),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
            ];

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(format!(
        " {} ({}) Esc=back ",
        app.scan_mode.name(),
        app.cleaners.len()
    )));

    frame.render_widget(list, area);
}

/// Render error screen
fn render_error_screen(message: &str, frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "⚠ Error",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(message),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'b' to go back or 'q' to quit",
            Style::default().fg(Color::Gray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Error "))
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render cleaning screen with progress bar
fn render_cleaning_screen(app: &App, frame: &mut Frame, area: Rect) {
    let (icon, title, message, color) = if app.permanent_delete {
        (
            "🔥",
            " PERMANENTLY DELETING ",
            "Removing files permanently (cannot be recovered)...",
            Color::Red,
        )
    } else {
        (
            "🗑️",
            " Cleaning ",
            "Moving items to trash...",
            Color::Yellow,
        )
    };

    // Create layout with space for progress bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(2), // Spacer
            Constraint::Length(3), // Progress bar
            Constraint::Length(2), // Spacer
            Constraint::Length(2), // Item count
            Constraint::Min(0),    // Rest
        ])
        .split(area);

    // Title text
    let title_text = Paragraph::new(Line::from(vec![
        Span::styled(format!("{} ", icon), Style::default().fg(color)),
        Span::styled(
            "Cleaning...",
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title_text, chunks[0]);

    // Animated progress bar (pulses since we don't track individual file progress)
    let progress = ((app.anim_frame % 20) as f64 / 20.0 * 0.3) + 0.7; // Pulse between 70-100%
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color)),
        )
        .gauge_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .percent((progress * 100.0) as u16)
        .label(Span::styled(
            message,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(gauge, chunks[2]);

    // Item count
    let item_text = Paragraph::new(Line::from(Span::styled(
        format!("Processing {} items...", app.pending_delete_items.len()),
        Style::default().fg(Color::Cyan),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(item_text, chunks[4]);

    // Border around the whole area
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(color));
    frame.render_widget(block, area);
}

/// Render status bar
fn render_status_bar(app: &App, frame: &mut Frame, area: Rect) {
    let selected_info = if app.selected_count() > 0 {
        format!(
            "Selected: {} ({}) | ",
            app.selected_count(),
            format_size(app.selected_size())
        )
    } else {
        String::new()
    };

    let search_info = if app.is_searching {
        format!("Search: {} | ", app.search_query)
    } else if !app.search_query.is_empty() {
        format!("Filter: {} | ", app.search_query)
    } else {
        String::new()
    };

    let status = app
        .status_message
        .clone()
        .unwrap_or_else(|| "Ready".to_string());

    let help_hint = " [?] Help  [q] Quit ";

    let status_bar = Paragraph::new(Line::from(vec![
        Span::styled(&selected_info, Style::default().fg(Color::Green)),
        Span::styled(&search_info, Style::default().fg(Color::Cyan)),
        Span::styled(&status, Style::default().fg(Color::White)),
        Span::raw(" ".repeat(area.width.saturating_sub(
            (selected_info.chars().count()
                + search_info.chars().count()
                + status.chars().count()
                + help_hint.chars().count()) as u16,
        ) as usize)),
        Span::styled(help_hint, Style::default().fg(Color::Gray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(status_bar, area);
}

/// Render confirmation dialog
fn render_confirm_dialog(app: &App, frame: &mut Frame, area: Rect) {
    let dialog_width = 55;
    let dialog_height = 11;

    let dialog_area = centered_rect(dialog_width, dialog_height, area);

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    // Show delete mode
    let mode_text = if app.permanent_delete {
        Span::styled(
            "🔥 PERMANENT (rm -rf)",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("🗑️  Move to Trash", Style::default().fg(Color::Green))
    };

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "⚠ Confirm Deletion",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(
            "Delete {} items ({})?",
            app.selected_count(),
            format_size(app.selected_size())
        )),
        Line::from(""),
        Line::from(vec![Span::raw("Mode: "), mode_text]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [y] ", Style::default().fg(Color::Green)),
            Span::raw("Yes  "),
            Span::styled(" [n] ", Style::default().fg(Color::Red)),
            Span::raw("No  "),
            Span::styled(" [p] ", Style::default().fg(Color::Magenta)),
            Span::raw("Toggle Permanent"),
        ]),
    ];

    let border_color = if app.permanent_delete {
        Color::Red
    } else {
        Color::Yellow
    };

    let dialog = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(" Confirm "),
        )
        .alignment(Alignment::Center);

    frame.render_widget(dialog, dialog_area);
}

/// Render help popup
fn render_help_popup(frame: &mut Frame, area: Rect) {
    let dialog_width = 65;
    let dialog_height = 24;

    let dialog_area = centered_rect(dialog_width, dialog_height, area);

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    let help_text = vec![
        Line::from(Span::styled(
            "🤖 null-e Keyboard Shortcuts",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " Navigation",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(vec![
            Span::styled("  j/↓      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move down"),
        ]),
        Line::from(vec![
            Span::styled("  k/↑      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move up"),
        ]),
        Line::from(vec![
            Span::styled("  →/l      ", Style::default().fg(Color::Yellow)),
            Span::raw("Expand item details"),
        ]),
        Line::from(vec![
            Span::styled("  ←/h      ", Style::default().fg(Color::Yellow)),
            Span::raw("Collapse item details"),
        ]),
        Line::from(vec![
            Span::styled("  g/Home   ", Style::default().fg(Color::Yellow)),
            Span::raw("Go to top"),
        ]),
        Line::from(vec![
            Span::styled("  G/End    ", Style::default().fg(Color::Yellow)),
            Span::raw("Go to bottom"),
        ]),
        Line::from(""),
        Line::from(Span::styled(" Selection", Style::default().fg(Color::Cyan))),
        Line::from(vec![
            Span::styled("  Space    ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle selection"),
        ]),
        Line::from(vec![
            Span::styled("  Enter    ", Style::default().fg(Color::Yellow)),
            Span::raw("Start scan / Toggle expand"),
        ]),
        Line::from(vec![
            Span::styled("  a        ", Style::default().fg(Color::Yellow)),
            Span::raw("Select all"),
        ]),
        Line::from(vec![
            Span::styled("  u/A      ", Style::default().fg(Color::Yellow)),
            Span::raw("Deselect all"),
        ]),
        Line::from(""),
        Line::from(Span::styled(" Actions", Style::default().fg(Color::Cyan))),
        Line::from(vec![
            Span::styled("  d        ", Style::default().fg(Color::Yellow)),
            Span::raw("Delete selected"),
        ]),
        Line::from(vec![
            Span::styled("  Esc/⌫   ", Style::default().fg(Color::Yellow)),
            Span::raw("Go back to menu"),
        ]),
        Line::from(vec![
            Span::styled("  /        ", Style::default().fg(Color::Yellow)),
            Span::raw("Search/filter"),
        ]),
        Line::from(vec![
            Span::styled("  q        ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Mouse: Scroll wheel to navigate | Press any key to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Help "),
        )
        .alignment(Alignment::Left);

    frame.render_widget(help, dialog_area);
}

/// Create a centered rect
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Truncate string to max length
fn truncate(s: &str, max: usize) -> String {
    let char_count = s.chars().count();
    if char_count > max {
        let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

/// Format age of a project
fn format_age(project: &null_e_core::core::Project) -> String {
    if let Some(modified) = project.last_modified {
        if let Ok(age) = modified.elapsed() {
            let days = age.as_secs() / 86400;
            if days == 0 {
                return "today".to_string();
            } else if days == 1 {
                return "1 day".to_string();
            } else if days < 30 {
                return format!("{} days", days);
            } else if days < 365 {
                return format!("{} months", days / 30);
            } else {
                return format!("{} years", days / 365);
            }
        }
    }
    "unknown".to_string()
}
