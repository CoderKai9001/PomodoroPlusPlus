use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear, Gauge},
};

use crate::app::{App, InputMode, PomodoroMode, Screen};

pub fn render_home(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Settings bar
            Constraint::Length(3),  // Help bar
        ])
        .split(frame.area());
    
    // Title
    let title = Paragraph::new("🍅 Pomodoro++")
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(title, chunks[0]);
    
    // Main content - split into timer and tags
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);
    
    render_timer(frame, app, main_chunks[0]);
    render_tags(frame, app, main_chunks[1]);
    
    // Settings bar
    let work_mins = app.work_duration / 60;
    let break_mins = app.break_duration / 60;
    let settings_text = format!(
        " ⏱  Work: {} min  │  Break: {} min  │  [w/W] adjust work  │  [b/B] adjust break ",
        work_mins, break_mins
    );
    let settings = Paragraph::new(settings_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(settings, chunks[2]);
    
    // Help bar
    let help_text = " [Space] Start/Pause │ [r] Reset │ [t] Tag │ [+] Add │ [-] Delete │ [s] Stats │ [m] Map │ [q] Quit ";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
    
    // Render tag input popup if in TagInput screen
    if app.current_screen == Screen::TagInput {
        render_tag_input_popup(frame, app);
    }
    
    // Render delete confirmation popup if in DeleteConfirm screen
    if app.current_screen == Screen::DeleteConfirm {
        render_delete_confirm_popup(frame, app);
    }
}

fn render_timer(frame: &mut Frame, app: &App, area: Rect) {
    let timer_block = Block::default()
        .title(" Timer ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    
    let inner = timer_block.inner(area);
    frame.render_widget(timer_block, area);
    
    let timer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),  // Mode
            Constraint::Min(3),     // Timer display
            Constraint::Length(3),  // Progress bar
            Constraint::Length(2),  // Status
        ])
        .split(inner);
    
    // Mode indicator
    let mode_color = match app.mode {
        PomodoroMode::Work => Color::Red,
        PomodoroMode::Break => Color::Green,
    };
    let mode_text = match app.mode {
        PomodoroMode::Work => "📚 WORK SESSION",
        PomodoroMode::Break => "☕ BREAK TIME",
    };
    let mode = Paragraph::new(mode_text)
        .style(Style::default().fg(mode_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(mode, timer_chunks[0]);
    
    // Timer display
    let time_str = app.format_time();
    let timer_color = if app.timer_running { Color::Yellow } else { Color::White };
    
    // Create large ASCII-style numbers
    let timer_display = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}  ", time_str),
            Style::default()
                .fg(timer_color)
                .add_modifier(Modifier::BOLD)
        )),
    ])
    .alignment(Alignment::Center)
    .style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(timer_display, timer_chunks[1]);
    
    // Progress bar
    let total_duration = match app.mode {
        PomodoroMode::Work => app.work_duration,
        PomodoroMode::Break => app.break_duration,
    };
    let elapsed = total_duration.saturating_sub(app.remaining_seconds);
    let progress_ratio = if total_duration > 0 {
        elapsed as f64 / total_duration as f64
    } else {
        0.0
    };
    
    let progress_color = match app.mode {
        PomodoroMode::Work => Color::Red,
        PomodoroMode::Break => Color::Green,
    };
    
    let progress_label = format!("{}%", (progress_ratio * 100.0) as u16);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(progress_color).bg(Color::DarkGray))
        .ratio(progress_ratio)
        .label(progress_label);
    frame.render_widget(gauge, timer_chunks[2]);
    
    // Status
    let status_text = if app.timer_running {
        "▶ Running"
    } else if app.remaining_seconds < match app.mode {
        PomodoroMode::Work => app.work_duration,
        PomodoroMode::Break => app.break_duration,
    } {
        "⏸ Paused"
    } else {
        "⏹ Ready"
    };
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(status, timer_chunks[3]);
}

fn render_tags(frame: &mut Frame, app: &App, area: Rect) {
    let tags_block = Block::default()
        .title(" Tags ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    
    let items: Vec<ListItem> = app.tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            let style = if i == app.selected_tag_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == app.selected_tag_index { "▶ " } else { "  " };
            ListItem::new(format!("{}{}", prefix, tag)).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .block(tags_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    
    frame.render_widget(list, area);
}

fn render_tag_input_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, frame.area());
    
    let popup_block = Block::default()
        .title(" New Tag ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    frame.render_widget(Clear, area);
    frame.render_widget(popup_block.clone(), area);
    
    let inner = popup_block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(inner);
    
    let label = Paragraph::new("Enter tag name:")
        .style(Style::default().fg(Color::White));
    frame.render_widget(label, chunks[0]);
    
    let input_style = match app.input_mode {
        InputMode::Editing => Style::default().fg(Color::Yellow),
        InputMode::Normal => Style::default().fg(Color::White),
    };
    
    let input = Paragraph::new(format!("{}_", app.input_buffer))
        .style(input_style)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(input, chunks[1]);
    
    let help = Paragraph::new("[Enter] Save │ [Esc] Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

fn render_delete_confirm_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 25, frame.area());
    
    let popup_block = Block::default()
        .title(" Delete Tag ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    
    frame.render_widget(Clear, area);
    frame.render_widget(popup_block.clone(), area);
    
    let inner = popup_block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(inner);
    
    let tag_name = app.get_tag_to_delete().unwrap_or("Unknown");
    let label = Paragraph::new(format!("Are you sure you want to delete\nthe tag \"{}\"?", tag_name))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    frame.render_widget(label, chunks[0]);
    
    let warning = Paragraph::new("This action cannot be undone!")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center);
    frame.render_widget(warning, chunks[1]);
    
    let help = Paragraph::new("[y] Yes, delete │ [n/Esc] Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
