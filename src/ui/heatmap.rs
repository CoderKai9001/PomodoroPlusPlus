use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::collections::HashMap;

use crate::app::App;

pub fn render_heatmap(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(12),    // Heatmap
            Constraint::Length(3),  // Legend
            Constraint::Length(2),  // Help
        ])
        .split(frame.area());
    
    // Title
    let title = Paragraph::new("📅 Activity Heatmap (Last 6 Months)")
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(title, chunks[0]);
    
    // Heatmap
    render_heatmap_grid(frame, app, chunks[1]);
    
    // Legend
    let legend = Paragraph::new(Line::from(vec![
        Span::raw(" Less "),
        Span::styled("░", Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled("▒", Style::default().fg(Color::Blue)),
        Span::raw(" "),
        Span::styled("▓", Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled("█", Style::default().fg(Color::Green)),
        Span::raw(" More"),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).title(" Legend "));
    frame.render_widget(legend, chunks[2]);

    // Help bar
    let help_text = " [h] Home │ [s] Stats │ [q] Quit ";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
}

fn render_heatmap_grid(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    // Get heatmap data
    let data = app.db.get_heatmap_data().unwrap_or_default();
    let data_map: HashMap<NaiveDate, i64> = data.into_iter().collect();
    
    // Calculate date range (last 6 months, ~26 weeks)
    let today = Local::now().date().naive_local();
    let start_date = today - Duration::days(180);
    
    // Find max value for intensity calculation
    let max_minutes = data_map.values().map(|v| *v / 60).max().unwrap_or(60).max(1);
    
    // Build the grid
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let mut lines: Vec<Line> = Vec::new();
    
    // Calculate weeks
    let mut weeks: Vec<Vec<Option<(NaiveDate, i64)>>> = Vec::new();
    let mut current_date = start_date;
    
    // Align to Monday
    while current_date.weekday() != Weekday::Mon {
        current_date = current_date + Duration::days(1);
    }
    
    while current_date <= today {
        let mut week = Vec::new();
        for _ in 0..7 {
            if current_date <= today {
                let minutes = data_map.get(&current_date).map(|v| *v / 60).unwrap_or(0);
                week.push(Some((current_date, minutes)));
            } else {
                week.push(None);
            }
            current_date = current_date + Duration::days(1);
        }
        weeks.push(week);
    }
    
    // Transpose to get rows by day of week
    for (day_idx, day_name) in days.iter().enumerate() {
        let mut spans: Vec<Span> = vec![
            Span::styled(format!("{} ", day_name), Style::default().fg(Color::White)),
        ];
        
        for week in &weeks {
            if let Some(Some((_, minutes))) = week.get(day_idx) {
                let (ch, color) = get_intensity_char(*minutes, max_minutes);
                spans.push(Span::styled(ch, Style::default().fg(color)));
            } else if week.get(day_idx).is_some() {
                spans.push(Span::styled("░", Style::default().fg(Color::DarkGray)));
            } else {
                spans.push(Span::raw(" "));
            }
        }
        
        lines.push(Line::from(spans));
    }
    
    // Add month labels
    let mut month_labels = vec![Span::raw("    ")]; // Align with day labels
    let mut last_month = 0u32;
    for week in &weeks {
        if let Some(Some((date, _))) = week.first() {
            if date.month() != last_month {
                let month_name = match date.month() {
                    1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
                    5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
                    9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
                    _ => "",
                };
                month_labels.push(Span::styled(month_name, Style::default().fg(Color::Yellow)));
                last_month = date.month();
            } else {
                month_labels.push(Span::raw(" "));
            }
        } else {
            month_labels.push(Span::raw(" "));
        }
    }
    lines.insert(0, Line::from(month_labels));
    
    let heatmap_text = Paragraph::new(lines)
        .alignment(Alignment::Left);
    
    frame.render_widget(heatmap_text, inner);
}

fn get_intensity_char(minutes: i64, max_minutes: i64) -> (&'static str, Color) {
    if minutes == 0 {
        ("░", Color::DarkGray)
    } else {
        let ratio = minutes as f64 / max_minutes as f64;
        if ratio < 0.25 {
            ("▒", Color::Blue)
        } else if ratio < 0.5 {
            ("▓", Color::Cyan)
        } else if ratio < 0.75 {
            ("▓", Color::LightCyan)
        } else {
            ("█", Color::Green)
        }
    }
}
