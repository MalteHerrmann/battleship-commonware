use std::io;

use commonware_runtime::{ContextCell, Metrics, Spawner, spawn_cell};
use futures::{StreamExt, channel::mpsc::{self, Receiver}};
use rand::Rng;
use ratatui::{Frame, Terminal, backend::CrosstermBackend, crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}}, layout::{Constraint, Layout, Rect}, widgets::{Block, Borders, List, ListItem, Paragraph}};

use super::ingress::{Log, LogType, Mailbox, Message};

pub struct GuiActor<R: Rng + Spawner + Metrics> {
    context: ContextCell<R>,
    mailbox: Receiver<Message>
}

impl<R: Rng + Spawner + Metrics> GuiActor<R> {
    pub fn new(context: R) -> (Self, Mailbox) {
        // TODO: use other size here?
        let (tx, rx) = mpsc::channel(1);

        (
            Self {
                context: ContextCell::new(context),
                mailbox: rx,
            },
            Mailbox::new(tx)
        )

    }

    pub fn start(
        mut self,
    ) {
        spawn_cell!(self.context, self.run().await);
    }

    async fn run(
        mut self,
    ) {
        // enable_raw_mode().expect("failed to set raw mode");
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).expect("failed to execute gui macro");
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("failed to create terminal gui");

        // Before receiving any messages we will draw an empty frame.
        terminal.draw(|frame| self.draw_empty(frame)).expect("failed to draw");

        let mut grid_string: String = "".into();
        let mut logs: Vec<Log> = vec![];

        while let Some(message) = self.mailbox.next().await {
            terminal.draw(|frame| {
                let [left, right] = create_layout(frame);

                match message {
                    Message::Draw { grid: g } => {
                        grid_string = g.to_owned();
                    },
                    Message::Log { log } => {
                        logs.push(log);
                    }
                };

                let grid = self.draw_grid(&grid_string);
                frame.render_widget(grid, left);

                let list = self.put_logs(&logs);
                frame.render_widget(list, right);
            }).expect("failed to draw");
        } 

        // disable_raw_mode().expect("failed to disable raw mode");
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen
        ).expect("failed to leave alternate screen");
        terminal.show_cursor().expect("failed to show cursor");
    }

    pub fn draw_empty(&self, frame: &mut Frame) {
        let [left, right] = create_layout(frame);

        let empty_grid = self.draw_grid("");
        frame.render_widget(empty_grid, left);

        let empty = Vec::new();
        let empty_logs = self.put_logs(&empty);
        frame.render_widget(empty_logs, right);
    }

    pub fn draw_grid<'a>(&self, grid: &'a str) -> Paragraph<'a> {
        let block = Block::default()
            .title("Game State")
            .borders(Borders::ALL);

        Paragraph::new(grid).block(block)
    }

    pub fn put_logs<'a>(&self, logs: &'a [Log]) -> List<'a> {
        let block = Block::default()
            .title("Logs")
            .borders(Borders::ALL);

        let items = logs.iter().rev().map(|log| ListItem::new(log.to_owned()));
        List::new(items).block(block)
    }
}

fn create_layout(frame: &mut Frame) -> [Rect;2] {
    Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .areas::<2>(frame.area())
}