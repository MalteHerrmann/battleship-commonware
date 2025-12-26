use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        execute,
    },
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut game_counter: i16 = 0;
    let mut logs = vec!["App started".to_string()];

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(f.area());

            // Top pane: Game UI
            let game_block = Block::default()
                .title("Game State")
                .borders(Borders::ALL);
            let game_text = Paragraph::new(format!("Counter: {}", game_counter))
                .block(game_block);
            f.render_widget(game_text, chunks[0]);

            // Bottom pane: Logs
            let log_items: Vec<ListItem> = logs
                .iter()
                .rev()
                .map(|log| ListItem::new(log.clone()))
                .collect();
            let log_block = Block::default()
                .title("Logs")
                .borders(Borders::ALL);
            let log_list = List::new(log_items).block(log_block);
            f.render_widget(log_list, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => {
                        game_counter += 1;
                        logs.push(format!("Counter incremented to {}", game_counter));
                    }
                    KeyCode::Down => {
                        game_counter = game_counter.saturating_sub(1);
                        logs.push(format!("Counter decremented to {}", game_counter));
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}