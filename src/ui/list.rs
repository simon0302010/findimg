//! # [Ratatui] List example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

use ratatui::{
    style::{
        Color,
        palette::tailwind::{GREEN, SLATE},
    },
    text::Line,
    widgets::{ListItem, ListState},
};

const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const TEXT_FG_COLOR: Color = SLATE.c200;
const CHECKED_TEXT_FG_COLOR: Color = GREEN.c500;

/// This struct holds the current state of the app. In particular, it has the `todo_list` field
/// which is a wrapper around `ListState`. Keeping track of the state lets us render the
/// associated widget with its state and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events. Check
/// the drawing logic for items on how to specify the highlighting style for selected items.

#[derive(Debug, Default)]
pub struct OptionList {
    pub items: Vec<OptionItem>,
    pub state: ListState,
}

#[derive(Debug)]
pub enum SearchEnum {
    Search,
    NegativePrompt,
    Ranking,
    Image2Image,
}

#[derive(Debug)]
pub struct OptionItem {
    pub option: String,
    pub status: OptionStatus,
    pub search_type: SearchEnum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OptionStatus {
    Unchecked,
    Checked,
}

impl FromIterator<(OptionStatus, &'static str, SearchEnum)> for OptionList {
    fn from_iter<I: IntoIterator<Item = (OptionStatus, &'static str, SearchEnum)>>(
        iter: I,
    ) -> Self {
        let items = iter
            .into_iter()
            .map(|(status, option, search_type)| OptionItem::new(status, option, search_type))
            .collect();
        let state = ListState::default();
        Self { items, state }
    }
}

impl OptionItem {
    fn new(status: OptionStatus, todo: &str, search_type: SearchEnum) -> Self {
        Self {
            status,
            option: todo.to_string(),
            search_type,
        }
    }
}

pub const fn alternate_colors(i: usize) -> Color {
    if i.is_multiple_of(2) {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

impl From<&OptionItem> for ListItem<'_> {
    fn from(value: &OptionItem) -> Self {
        let line = match value.status {
            OptionStatus::Unchecked => Line::styled(format!(" ☐ {}", value.option), TEXT_FG_COLOR),
            OptionStatus::Checked => {
                Line::styled(format!(" ✓ {}", value.option), CHECKED_TEXT_FG_COLOR)
            }
        };
        ListItem::new(line)
    }
}
