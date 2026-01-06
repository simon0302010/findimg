use std::{error::Error, io};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph},
};

#[derive(Debug)]
pub struct App {
    search: String,
    input_mode: InputMode,
    char_index: usize,
    exit: bool,
    current_element: CurrentElement,
}

#[derive(Debug, PartialEq)]
enum CurrentElement {
    Search,
    Filter,
}

#[derive(Debug)]
enum InputMode {
    Normal,
    Editing,
}

fn main() -> Result<(), Box<dyn Error>> {
    ratatui::run(|terminal| App::default().run(terminal))?;
    Ok(())
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, img_area] = vertical.areas(frame.area());

        let interactive_bar =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [search_area, mode_area] = interactive_bar.areas(input_area);

        let (msg, style) = match self.input_mode {
            InputMode::Normal => {
                if self.current_element == CurrentElement::Search {
                    (
                        vec![
                            "Press ".into(),
                            "q".bold(),
                            " to exit, ".into(),
                            "e".bold(),
                            " to start editing".into(),
                        ],
                        Style::default().add_modifier(Modifier::RAPID_BLINK),
                    )
                } else {
                    (
                        vec![
                            "Press ".into(),
                            "q".bold(),
                            " to exit, ".into(),
                            "Enter".bold(),
                            " to select mode".into(),
                        ],
                        Style::default(),
                    )
                }
            }
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to search for images".into(),
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.search.as_str())
            .style(match self.input_mode {
                InputMode::Normal => {
                    if self.current_element == CurrentElement::Search {
                        Style::default().fg(Color::Rgb(255, 165, 0))
                    } else {
                        Style::default()
                    }
                }
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Search"));
        frame.render_widget(input, search_area);
        match self.input_mode {
            InputMode::Normal => {}
            InputMode::Editing => frame.set_cursor_position(Position::new(
                search_area.x + self.char_index as u16 + 1,
                search_area.y + 1,
            )),
        }

        let modesel = Paragraph::new("Placeholder for button")
            .style(match self.current_element {
                CurrentElement::Filter => Style::default().fg(Color::Rgb(255, 165, 0)),
                CurrentElement::Search => Style::default(),
            })
            .block(Block::bordered().title("Mode"));
        frame.render_widget(modesel, mode_area);

        let block = Block::bordered()
            .title("Images")
            .title_alignment(Alignment::Center);
        frame.render_widget(block, img_area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(mut key) = event::read()? {
            if let KeyCode::Char(c) = key.code {
                key.code = KeyCode::Char(c.to_lowercase().collect::<Vec<char>>()[0])
            }

            match self.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        if self.current_element == CurrentElement::Search {
                            self.input_mode = InputMode::Editing;
                        }
                    }
                    KeyCode::Char('q') => {
                        self.exit();
                    }
                    KeyCode::Right => {
                        // filters are on the right
                        self.current_element = CurrentElement::Filter;
                    }
                    KeyCode::Left => {
                        // search is on the left
                        self.current_element = CurrentElement::Search;
                    }
                    _ => {}
                },
                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => self.search(),
                    KeyCode::Char(to_insert) => self.enter_char(to_insert),
                    KeyCode::Backspace => self.delete_char(),
                    KeyCode::Delete => self.delete_right(),
                    KeyCode::Left => self.move_cursor_left(),
                    KeyCode::Right => self.move_cursor_right(),
                    KeyCode::Esc => self.input_mode = InputMode::Normal,
                    _ => {}
                },
                InputMode::Editing => {}
            }
        }

        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_index.saturating_sub(1);
        self.char_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_index.saturating_add(1);
        self.char_index = self.clamp_cursor(cursor_moved_right);
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.search.chars().count())
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.search.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.search
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_index)
            .unwrap_or(self.search.len())
    }

    fn delete_char(&mut self) {
        if self.char_index != 0 {
            let current_index = self.char_index;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = self.search.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.search.chars().skip(current_index);
            self.search = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    // prob better way to do this
    fn delete_right(&mut self) {
        if self.char_index < self.search.len() {
            self.search.remove(self.char_index);
        }
    }

    fn reset_cursor(&mut self) {
        self.char_index = 0;
    }

    fn search(&mut self) {
        self.search.clear();
        self.reset_cursor();
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            search: String::new(),
            exit: false,
            input_mode: InputMode::Normal,
            char_index: 0,
            current_element: CurrentElement::Search,
        }
    }
}
