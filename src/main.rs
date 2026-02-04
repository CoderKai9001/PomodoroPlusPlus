mod app;
mod ascii_art;
mod db;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, InputMode, Screen};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?;
    
    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();
    let mut second_tracker = Instant::now();

    loop {
        // Draw UI
        terminal.draw(|f| {
            match app.current_screen {
                Screen::Home | Screen::TagInput | Screen::DeleteConfirm => ui::render_home(f, app),
                Screen::Stats => ui::render_stats(f, app),
                Screen::Heatmap => ui::render_heatmap(f, app),
                Screen::Settings => ui::render_home(f, app),
            }
        })?;

        // Handle input with timeout
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_event(app, key.code);
                }
            }
        }

        // Timer tick (every second)
        if second_tracker.elapsed() >= Duration::from_secs(1) {
            app.tick();
            second_tracker = Instant::now();
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_key_event(app: &mut App, key: KeyCode) {
    // Handle input mode separately
    if app.current_screen == Screen::TagInput {
        match app.input_mode {
            InputMode::Editing => match key {
                KeyCode::Enter => {
                    let tag = app.input_buffer.trim().to_string();
                    if !tag.is_empty() {
                        app.add_tag(tag);
                    }
                    app.input_buffer.clear();
                    app.input_mode = InputMode::Normal;
                    app.current_screen = Screen::Home;
                }
                KeyCode::Esc => {
                    app.input_buffer.clear();
                    app.input_mode = InputMode::Normal;
                    app.current_screen = Screen::Home;
                }
                KeyCode::Backspace => {
                    app.input_buffer.pop();
                }
                KeyCode::Char(c) => {
                    app.input_buffer.push(c);
                }
                _ => {}
            },
            InputMode::Normal => {
                app.input_mode = InputMode::Editing;
            }
        }
        return;
    }
    
    // Handle delete confirmation
    if app.current_screen == Screen::DeleteConfirm {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.delete_selected_tag();
                app.current_screen = Screen::Home;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.current_screen = Screen::Home;
            }
            _ => {}
        }
        return;
    }

    match app.current_screen {
        Screen::Home | Screen::Settings => match key {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char(' ') => app.toggle_timer(),
            KeyCode::Char('r') => app.reset_timer(),
            KeyCode::Char('t') | KeyCode::Tab => app.next_tag(),
            KeyCode::Char('T') | KeyCode::BackTab => app.prev_tag(),
            KeyCode::Char('+') | KeyCode::Char('n') => {
                app.navigate_to(Screen::TagInput);
                app.input_mode = InputMode::Editing;
            }
            KeyCode::Char('-') => {
                if !app.tags.is_empty() {
                    app.navigate_to(Screen::DeleteConfirm);
                }
            }
            KeyCode::Char('s') => app.navigate_to(Screen::Stats),
            KeyCode::Char('m') => app.navigate_to(Screen::Heatmap),
            KeyCode::Char('w') => app.adjust_work_duration(60),   // +1 min
            KeyCode::Char('W') => app.adjust_work_duration(-60),  // -1 min
            KeyCode::Char('b') => app.adjust_break_duration(60),  // +1 min
            KeyCode::Char('B') => app.adjust_break_duration(-60), // -1 min
            KeyCode::Up => app.prev_tag(),
            KeyCode::Down => app.next_tag(),
            _ => {}
        },
        Screen::Stats => match key {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('h') => app.navigate_to(Screen::Home),
            KeyCode::Char('m') => app.navigate_to(Screen::Heatmap),
            KeyCode::Tab => app.toggle_stats_view(),
            KeyCode::Left => app.prev_stats_tag(),
            KeyCode::Right => app.next_stats_tag(),
            _ => {}
        },
        Screen::Heatmap => match key {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('h') => app.navigate_to(Screen::Home),
            KeyCode::Char('s') => app.navigate_to(Screen::Stats),
            _ => {}
        },
        _ => {}
    }
}
