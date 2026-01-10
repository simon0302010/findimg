use std::time::{Duration, Instant};

use ratatui::{
    Frame,
    layout::{HorizontalAlignment, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Paragraph},
};

impl Messages {
    pub fn add(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    /// draws all messages to the screen. also handles disposing of expired ones.
    pub fn draw(&mut self, frame: &mut Frame) {
        // remove expired messages
        self.messages
            .retain(|msg| msg.time.elapsed() < msg.duration);

        for (index, message) in self.messages.iter().enumerate() {
            let width = message.text.len() as u16 + 4;
            let height = 3;
            let (size_x, _) = crossterm::terminal::size().unwrap_or((80, 24));
            let x = size_x - width - 1; // 1 is the distance from the right edge
            let y = index as u16 * (height + 1) + 1;

            let msg_title = match message.severity {
                MessageSeverity::Error => "Error",
                MessageSeverity::Warning => "Warning",
                MessageSeverity::Info => "Info",
            };
            let msg_color = match message.severity {
                MessageSeverity::Error => Color::Red,
                MessageSeverity::Warning => Color::Yellow,
                MessageSeverity::Info => Color::White,
            };
            let msg_block = Block::bordered()
                .title(msg_title)
                .title_alignment(HorizontalAlignment::Left)
                .bg(msg_color)
                .fg(Color::Black);

            let area = Rect::new(x, y, width, height);
            let block_area = msg_block.inner(area);

            let msg = Paragraph::new(Line::from(format!(" {}", message.text)))
                .bg(msg_color)
                .fg(Color::Black);

            frame.render_widget(msg_block, area);
            frame.render_widget(msg, block_area);
        }
    }
}

impl Message {
    pub fn new(
        text: impl Into<String>,
        severity: MessageSeverity,
        title: impl Into<String>,
        duration: Duration,
    ) -> Self {
        Self {
            text: text.into(),
            severity: severity,
            title: title.into(),
            duration,
            time: Instant::now(),
        }
    }
}

pub enum MessageSeverity {
    Info,
    Warning,
    Error,
}

pub struct Messages {
    messages: Vec<Message>,
}

pub struct Message {
    text: String,
    severity: MessageSeverity,
    title: String,
    duration: Duration,
    time: Instant,
}

impl Default for Messages {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}
