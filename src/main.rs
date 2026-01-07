mod button;
mod list;

use std::{error::Error, fs, io};
use clipers::{rust_embed_text, rust_embed_image, rust_end, rust_init};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, List, ListItem, Paragraph},
};

use crate::{
    button::{BLUE, Button, State},
    list::{OptionList, OptionStatus, alternate_colors},
};

use std::collections::HashMap;

#[derive(Debug)]
pub struct App {
    search: String,
    input_mode: InputMode,
    char_index: usize,
    exit: bool,
    current_element: CurrentElement,
    button_pressed: bool,
    modesel_open: bool,
    modesel_list: OptionList,
    images_paths: Vec<String>,
    search_results: Vec<String>,
    images_embedding: HashMap<String, Vec<f32>>,
}

#[derive(Debug, PartialEq)]
enum CurrentElement {
    Search,
    Filter,
    Modesel,
}

#[derive(Debug)]
enum InputMode {
    Normal,
    Editing,
}

const SEARCH_RESULTS: u64 = 20;

fn main() -> Result<(), Box<dyn Error>> {
    rust_init("clip-vit-large-patch14_ggml-model-f16.gguf");
    ratatui::run(|terminal| App::default().run(terminal))?;
    rust_end();
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

    fn draw(&mut self, frame: &mut Frame) {
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
                            "c".bold(),
                            " to clear the search, ".into(),
                            "Enter".bold(),
                            " to start editing".into(),
                        ],
                        Style::default().add_modifier(Modifier::RAPID_BLINK),
                    )
                } else if self.current_element == CurrentElement::Filter {
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
                } else {
                    (vec![], Style::default())
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
                        Style::default().fg(BLUE.highlight)
                    } else {
                        Style::default().fg(BLUE.background)
                    }
                }
                InputMode::Editing => Style::default().fg(Color::LightCyan),
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

        let button_state = if self.current_element == CurrentElement::Filter {
            if self.button_pressed {
                State::Active
            } else {
                State::Selected
            }
        } else {
            State::Normal
        };
        self.button_pressed = false;
        let mode_selector = Button::new("Choose Mode").state(button_state).theme(BLUE);
        frame.render_widget(mode_selector, mode_area);

        // images block
        let block = Block::bordered()
            .title("Images")
            .title_alignment(Alignment::Center);
        frame.render_widget(block, img_area);

        if self.modesel_open {
            let popup_vertical = Layout::vertical([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ]);

            let [_, middle_vertical, _] = popup_vertical.areas(frame.area());

            let popup_horizontal = Layout::horizontal([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ]);

            let [_, middle, _] = popup_horizontal.areas(middle_vertical);

            let popup_block = Block::bordered()
                .title("Select Mode")
                .title_alignment(Alignment::Center);

            /*let block_area = popup_block.inner(middle);

            frame.render_widget(popup_block, middle);*/

            let items: Vec<ListItem> = self
                .modesel_list
                .items
                .iter()
                .enumerate()
                .map(|(i, todo_item)| {
                    let color = alternate_colors(i);
                    ListItem::from(todo_item).bg(color)
                })
                .collect();

            let list = List::new(items)
                .block(popup_block)
                .highlight_style(Style::new().bg(BLUE.highlight).add_modifier(Modifier::BOLD))
                .highlight_symbol(">")
                .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

            frame.render_stateful_widget(list, middle, &mut self.modesel_list.state);
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Ok(true) = event::poll(std::time::Duration::from_millis(200))
            && let Event::Key(mut key) = event::read()?
        {
            if let KeyCode::Char(c) = key.code {
                key.code = KeyCode::Char(c.to_lowercase().collect::<Vec<char>>()[0])
            }

            match self.input_mode {
                InputMode::Normal => match self.current_element {
                    CurrentElement::Search => match key.code {
                        KeyCode::Char('c') => self.clear_search(),
                        KeyCode::Char('q') => self.exit(),
                        KeyCode::Right => self.current_element = CurrentElement::Filter,
                        KeyCode::Left => self.current_element = CurrentElement::Search,
                        KeyCode::Enter => self.input_mode = InputMode::Editing,
                        _ => {}
                    },
                    CurrentElement::Filter => match key.code {
                        KeyCode::Char('q') => self.exit(),
                        KeyCode::Right => self.current_element = CurrentElement::Filter,
                        KeyCode::Left => self.current_element = CurrentElement::Search,
                        KeyCode::Enter => {
                            self.button_pressed = true;
                            self.modesel_open = !self.modesel_open;
                            self.current_element = CurrentElement::Modesel;
                            self.modesel_list.state.select(Some(0));
                        }
                        _ => {}
                    },
                    CurrentElement::Modesel => match key.code {
                        KeyCode::Char('q') => {
                            self.modesel_open = false;
                            self.current_element = CurrentElement::Filter;
                        }
                        KeyCode::Enter => {
                            self.toggle_status();
                            self.modesel_open = false;
                            self.current_element = CurrentElement::Filter;
                        }
                        KeyCode::Char(' ') => self.toggle_status(),
                        KeyCode::Down => self.select_next(),
                        KeyCode::Up => self.select_previous(),
                        _ => {}
                    },
                },
                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => self.search_results = self.search(),
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

    // gets called whenever enter is pressed.
    // is supposed to return an array of all matching image paths from best match to worst.
    // returns as many results as SEARCH_RESULTS specifies.
    fn search(&mut self) -> Vec<String> {
        vec![]
    }

    fn clear_search(&mut self) {
        self.search.clear();
        self.reset_cursor();
    }

    fn select_next(&mut self) {
        self.modesel_list.state.select_next();
    }

    fn select_previous(&mut self) {
        self.modesel_list.state.select_previous();
    }

    fn toggle_status(&mut self) {
        for item in &mut self.modesel_list.items {
            item.status = OptionStatus::Unchecked;
        }
        if let Some(i) = self.modesel_list.state.selected() {
            self.modesel_list.items[i].status = match self.modesel_list.items[i].status {
                OptionStatus::Checked => OptionStatus::Unchecked,
                OptionStatus::Unchecked => OptionStatus::Checked,
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let paths = fs::read_dir("images/").unwrap();

        let mut images_paths: Vec<String> = vec![];
        let mut image_embeddings: HashMap<String, Vec<f32>> = HashMap::new();

        for path in paths {
            images_paths.push(path.unwrap().path().display().to_string());
        }

        let mut index: usize = 0;
        for image in &images_paths{
            index += 1;
            println!("Embedded {}/{}", index, images_paths.len());
            image_embeddings.insert(image.clone(), rust_embed_image(image.clone()).unwrap());
        }


        Self {
            search: String::new(),
            exit: false,
            input_mode: InputMode::Normal,
            char_index: 0,
            current_element: CurrentElement::Search,
            button_pressed: false,
            modesel_open: false,
            modesel_list: OptionList::from_iter([(OptionStatus::Checked, "Search")]),
            search_results: Vec::new(),
            images_paths: images_paths,
            images_embedding: image_embeddings
        }
    }
}
