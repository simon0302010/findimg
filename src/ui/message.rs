use std::time::{Duration, Instant};

use ratatui::{
    Frame,
    layout::{HorizontalAlignment, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Paragraph},
};

impl Messages {
    /// Creates a new message and displays it
    ///
    /// # Example
    ///
    /// ```rust
    /// messages.add(Message::default());
    /// ```
    pub fn add(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    /// Draws all unexpired messages to the screen. Multiple messages will be stacked from oldest to newest.
    ///
    /// # Example
    ///
    /// ```rust
    /// messages.draw(frame);
    /// ```
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
                .title_alignment(HorizontalAlignment::Center)
                .title_bottom(format!(
                    "{:.1}s",
                    message
                        .duration
                        .saturating_sub(message.time.elapsed())
                        .as_millis() as f64
                        / 1000.0
                ))
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
    /// Creates a new message. Use `Messages::add()` to display it.
    ///
    /// # Arguments
    ///
    /// - text: The text content of the Message.
    /// - severity: The severity of the message. This controls the color and title of the message.
    /// - duration: This option sets the duration the message will be displayed before disappearing.
    ///
    /// # Example
    ///
    /// ```rust
    /// let msg = Message::new(
    ///     "Message text",
    ///     MessageSeverity::Warning,
    ///     Duration::from_secs(5)
    /// );
    /// ```
    pub fn new(text: impl Into<String>, severity: MessageSeverity, duration: Duration) -> Self {
        Self {
            text: text.into(),
            severity: severity,
            duration,
            time: Instant::now(),
        }
    }
}

#[allow(unused)]
/// Severity of a message. This controls the background color and title.
pub enum MessageSeverity {
    /// Info, the Background is white and the title is "Info"
    Info,
    /// Warning, the Background is yellow and the title is "Warning"
    Warning,
    /// Error, the Background is red and the title is "Error"
    Error,
}

/// Structure containing messages. Implementation handles adding and displaying them.
///
/// # Example
///
/// ```rust
/// let messages = Messages::default()
/// ```
pub struct Messages {
    /// Vector containing the messages
    messages: Vec<Message>,
}

/// Message struct. This includes text, severity, duration, and creation time.
pub struct Message {
    /// Text content of the messages
    pub text: String,
    /// Severity of the message
    pub severity: MessageSeverity,
    /// Duration the message will be displayed for
    pub duration: Duration,
    /// Time the message has been created
    pub time: Instant,
}

impl Default for Messages {
    /// Default Messages struct containing an empty Vector
    fn default() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}

impl Default for Message {
    /// Default Message struct. Creates an empty info message that will be displayed for 3 seconds.
    fn default() -> Self {
        Self {
            text: String::new(),
            severity: MessageSeverity::Info,
            duration: Duration::from_secs(3),
            time: Instant::now(),
        }
    }
}
