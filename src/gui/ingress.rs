use futures::channel::mpsc::Sender;
use ratatui::{style::{Color, Style}, text::Text};

/// TODO: check if lifetime would better be removed?
pub struct Mailbox {
    pub sender: Sender<Message>
}

impl Mailbox {
    pub fn new(sender: Sender<Message>) -> Self {
        Self { sender }
    }
}

pub enum Message {
    Draw { grid: String },
    Log { log: Log}
}

#[derive(Clone)]
pub struct Log {
    content: String,
    log_type: LogType,
}

impl Log {
    pub fn new(log_type: LogType, content: String) -> Log {
        Self {
            log_type,
            content
        }
    }
}

impl<'a> Into<Text<'a>> for Log {
    fn into(self) -> Text<'a> {
        let style = match self.log_type {
            LogType::Debug => Style::new().fg(Color::DarkGray),
            LogType::Hit => Style::new().fg(Color::Green),
            LogType::Miss => Style::new().fg(Color::Red),
            LogType::Lost => Style::new().fg(Color::Yellow),
            LogType::OpponentHit => Style::new().fg(Color::Red),
            LogType::OpponentMiss => Style::new().fg(Color::Green),
            LogType::Error => Style::new().fg(Color::Red),
            LogType::Info => Style::new().fg(Color::Yellow),
            LogType::Won => Style::new().fg(Color::Yellow),
        };

        Text::styled(self.content, style)
    }
}

#[derive(Clone, PartialEq)]
pub enum LogType {
    Debug,
    Error,
    Hit,
    Info,
    Lost,
    Miss,
    OpponentHit,
    OpponentMiss,
    Won,
}