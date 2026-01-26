use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph},
};

use crate::app::{App, StatsView};

pub fn render_stats(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Controls
            Constraint::Min(10),    // Chart
            Constraint::Length(2),  // Help
        ])
        .split(frame.area());
    
    // Title
    let title = Paragraph::new("📊 Statistics")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(title, chunks[0]);
    
    // Controls bar
    render_controls(frame, app, chunks[1]);
    
    // Chart
    render_chart(frame, app, chunks[2]);
    
    // Help bar
    let help_text = " [Tab] Toggle View │ [←/→] Change Tag │ [h] Home │ [m] Heatmap │ [q] Quit ";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
}

fn render_controls(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    // View toggle
    let weekly_style = if app.stats_view == StatsView::Weekly {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let monthly_style = if app.stats_view == StatsView::Monthly {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    
    let view_line = Line::from(vec![
        Span::raw(" View: "),
        Span::styled("[ Weekly ]", weekly_style),
        Span::raw("  "),
        Span::styled("[ Monthly ]", monthly_style),
    ]);
    let view = Paragraph::new(view_line)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(view, chunks[0]);
    
    // Tag filter
    let tag_name = match app.get_stats_tag() {
        Some(tag) => tag.to_string(),
        None => "All Tags".to_string(),
    };
    let tag_line = Line::from(vec![
        Span::raw(" Tag: "),
        Span::styled(format!("◀ {} ▶", tag_name), Style::default().fg(Color::Magenta)),
    ]);
    let tag = Paragraph::new(tag_line)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(tag, chunks[1]);
}

fn render_chart(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chart_block = Block::default()
        .title(match app.stats_view {
            StatsView::Weekly => " Weekly Activity (minutes) ",
            StatsView::Monthly => " Monthly Activity (minutes) ",
        })
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    
    // Get data based on view type
    let data = match app.stats_view {
        StatsView::Weekly => app.db.get_weekly_stats(app.get_stats_tag()).unwrap_or_default(),
        StatsView::Monthly => app.db.get_monthly_stats(app.get_stats_tag()).unwrap_or_default(),
    };
    
    if data.is_empty() {
        let no_data = Paragraph::new("\n\n  No data available yet. Complete some Pomodoro sessions to see statistics!")
            .style(Style::default().fg(Color::DarkGray))
            .block(chart_block);
        frame.render_widget(no_data, area);
        return;
    }
    
    // Convert data to bar chart format
    let bars: Vec<Bar> = data
        .iter()
        .map(|(label, value)| {
            let short_label = if app.stats_view == StatsView::Weekly {
                // Show day of week
                label.chars().skip(5).collect::<String>() // Skip year, show MM-DD
            } else {
                label.clone()
            };
            Bar::default()
                .value((*value as u64) / 60) // Convert to minutes
                .label(Line::from(short_label))
                .style(Style::default().fg(Color::Cyan))
        })
        .collect();
    
    let bar_group = BarGroup::default().bars(&bars);
    
    let bar_chart = BarChart::default()
        .block(chart_block)
        .data(bar_group)
        .bar_width(5)
        .bar_gap(2)
        .bar_style(Style::default().fg(Color::Cyan))
        .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    
    frame.render_widget(bar_chart, area);
}
