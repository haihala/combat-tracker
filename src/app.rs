use std::{fmt::Display, io, str::FromStr};

use log::info;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::Rect,
    prelude::*,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget},
    DefaultTerminal,
};
use tui_textarea::{CursorMove, TextArea};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    Help,
    Normal,
    Rename(String),
    SetHealth(i32),
    SetInitiative(i32),
    HealthShift,
    EditNotes,
    Sort,
}
impl Mode {
    fn get_instructions(&self) -> Line {
        match self {
            Mode::Help => panic!("Should not ask for instructions in help mode"),
            Mode::Normal => Line::from(vec![
                " Exit: ".into(),
                "Esc".blue().bold(),
                " Help: ".into(),
                "? ".blue().bold(),
            ]),
            Mode::Rename(_) | Mode::SetHealth(_) | Mode::SetInitiative(_) | Mode::HealthShift => {
                Line::from(vec![
                    " Confirm: ".into(),
                    "Enter".blue().bold(),
                    ", Cancel: ".into(),
                    "Esc ".blue().bold(),
                ])
            }
            Mode::Sort => Line::from(vec![
                " Press letter to determine order, shift reverses: (".into(),
                "I".blue().bold(),
                ")nitiative, (".into(),
                "H".blue().bold(),
                ")ealth, (".into(),
                "N".blue().bold(),
                ")ame or ".into(),
                "Esc".blue().bold(),
                "to cancel".into(),
            ]),
            Mode::EditNotes => Line::from(vec![
                " Confirm: ".into(),
                "Enter".blue().bold(),
                " (use alt to break lines), Cancel: ".into(),
                "Esc ".blue().bold(),
            ]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct App<'a> {
    running: bool,
    mode: Mode,
    list_state: ListState,
    creatures: Vec<Creature>,
    text_area: TextArea<'a>,
}

enum HotKey {
    Divider {
        text: &'static str,
        newline: bool,
    },
    Embed {
        pre: &'static str,
        color: &'static str,
        post: &'static str,
    },
    Label {
        label: &'static str,
        keys: &'static str,
    },
}

const HELP_BLURB: &str = "\
Howdy partner, this is a combat tracker I use for my Pathfinder 2e games.
It's designed for me and since I'm a bit of a power user, so it's a modal
system that's exclusively keyboard operated.

Normal mode is the most complex. Besides that most modes have like three shoftcuts.
Most modes have a banner at the bottom with some help.

Best of luck
";
const HOTKEYS: &[HotKey] = &[
    HotKey::Divider {
        text: "In normal mode",
        newline: false,
    },
    HotKey::Label {
        label: "Open this help message",
        keys: "?",
    },
    HotKey::Label {
        label: "Quit",
        keys: "Esc",
    },
    HotKey::Label {
        label: "Move",
        keys: "JjkK",
    },
    HotKey::Embed {
        pre: "",
        color: "A",
        post: "dd a creature",
    },
    HotKey::Embed {
        pre: "",
        color: "R",
        post: "ename a creature",
    },
    HotKey::Embed {
        pre: "",
        color: "C",
        post: "opy (duplicate) a creature",
    },
    HotKey::Embed {
        pre: "",
        color: "D",
        post: "elete a creature",
    },
    HotKey::Embed {
        pre: "Set ",
        color: "i",
        post: "nitiative of a creature",
    },
    HotKey::Embed {
        pre: "Set ",
        color: "H",
        post: "health a creature",
    },
    HotKey::Label {
        label: "Subtract health",
        keys: "-",
    },
    HotKey::Label {
        label: "Add health",
        keys: "+",
    },
    HotKey::Embed {
        pre: "",
        color: "S",
        post: "ort creatures",
    },
    HotKey::Divider {
        text: "In most editing modes",
        newline: true,
    },
    HotKey::Label {
        label: "Confirm",
        keys: "Enter",
    },
    HotKey::Label {
        label: "Cancel",
        keys: "Esc",
    },
    HotKey::Divider {
        text: "In sort mode (shift inverts direction)",
        newline: true,
    },
    HotKey::Embed {
        pre: "Sort by ",
        color: "I",
        post: "nitiative",
    },
    HotKey::Embed {
        pre: "Sort by ",
        color: "H",
        post: "ealth",
    },
    HotKey::Embed {
        pre: "Sort by ",
        color: "N",
        post: "ame",
    },
    HotKey::Label {
        label: "Cancel",
        keys: "Esc",
    },
    HotKey::Divider {
        text: "In help mode",
        newline: true,
    },
    HotKey::Label {
        label: "Return to normal mode",
        keys: "Esc",
    },
];

impl App<'_> {
    pub fn new() -> App<'static> {
        App {
            running: true,
            mode: Mode::Normal,
            list_state: ListState::default(),
            creatures: vec![
                Creature {
                    name: "Goblin".into(),
                    health: 5,
                    notes: "Very gobliny".into(),
                    ..Default::default()
                },
                Creature {
                    name: "Chodlin".into(),
                    health: 4,
                    notes: "Cousin of Boblin".into(),
                    ..Default::default()
                },
                Creature {
                    name: "Boblin".into(),
                    health: 4,
                    notes: "The goblin".into(),
                    ..Default::default()
                },
            ],
            text_area: TextArea::default(),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while self.running {
            terminal
                .draw(|frame| {
                    frame.render_widget(self.clone(), frame.area());
                })
                .expect("failed to draw frame");
            self.read_events()?;
        }

        Ok(())
    }

    fn hovered_creature(&self) -> Option<&Creature> {
        self.list_state
            .selected()
            .and_then(|index| self.creatures.get(index))
    }

    fn hovered_creature_mut(&mut self) -> Option<&mut Creature> {
        self.list_state
            .selected()
            .and_then(|index| self.creatures.get_mut(index))
    }

    fn read_events(&mut self) -> io::Result<()> {
        let Event::Key(ev) = event::read()? else {
            return Ok(());
        };

        info!("Key press - {:?}", ev);

        match (&self.mode, ev.kind) {
            (Mode::Normal, KeyEventKind::Press) => {
                match ev.code {
                    // Quitting
                    KeyCode::Esc => self.running = false,

                    KeyCode::Char('?') => self.mode = Mode::Help,
                    KeyCode::Char('s') => self.mode = Mode::Sort,

                    // Navigation
                    KeyCode::Char('K') => self.select_creature(0),
                    KeyCode::Char('k') => self.select_creature({
                        let curr = self.list_state.selected().unwrap_or_default();
                        if curr == 0 {
                            self.creatures.len().saturating_sub(1)
                        } else {
                            curr - 1
                        }
                    }),
                    KeyCode::Char('j') => self.select_creature({
                        if self.creatures.is_empty() {
                            0
                        } else {
                            (self
                                .list_state
                                .selected()
                                .map(|num| num + 1)
                                .unwrap_or_default())
                                % self.creatures.len()
                        }
                    }),
                    KeyCode::Char('J') => {
                        self.select_creature(self.creatures.len().saturating_sub(1))
                    }

                    // Actions
                    KeyCode::Char('a') => {
                        self.creatures.push(Creature {
                            name: "".into(),
                            ..Creature::default()
                        });
                        self.list_state.select(Some(self.creatures.len() - 1));
                        self.mode = Mode::Rename(String::new());
                    }
                    KeyCode::Char('r') => {
                        if let Some(creat) = self.hovered_creature_mut() {
                            self.mode = Mode::Rename(creat.name.clone());
                        }
                    }
                    KeyCode::Char('n') => {
                        if self.hovered_creature().is_some() {
                            self.mode = Mode::EditNotes;
                        }
                    }
                    KeyCode::Char('c') => {
                        // TODO: Think about automatically renaming with indices or something
                        if let Some(hovered) = self.hovered_creature() {
                            let index = self.list_state.selected().unwrap();
                            let duplicate = hovered.clone();
                            self.creatures.insert(index + 1, duplicate);
                        }
                    }
                    KeyCode::Char('d') => {
                        if self.hovered_creature().is_some() {
                            let index = self.list_state.selected().unwrap();
                            self.creatures.remove(index);
                            if self.creatures.is_empty() {
                                self.list_state.select(None);
                            } else if self.creatures.len() == index {
                                // Deleted final element in a non-empty list
                                self.list_state.select(Some(self.creatures.len() - 1));
                            }
                        }
                    }
                    KeyCode::Char('h') => {
                        if let Some(creat) = self.hovered_creature() {
                            self.mode = Mode::SetHealth(creat.health);
                        }
                    }
                    KeyCode::Char('i') => {
                        if let Some(creat) = self.hovered_creature() {
                            self.mode = Mode::SetInitiative(creat.initiative);
                        }
                    }
                    KeyCode::Char('-') => {
                        if let Some(creature) = self.hovered_creature_mut() {
                            creature.health_shift = Some(HealthShift::Decrease(0));
                            self.mode = Mode::HealthShift;
                        }
                    }
                    KeyCode::Char('+') => {
                        if let Some(creature) = self.hovered_creature_mut() {
                            creature.health_shift = Some(HealthShift::Increase(0));
                            self.mode = Mode::HealthShift;
                        }
                    }
                    _ => {}
                }
            }
            (Mode::Rename(old_name), KeyEventKind::Press) => {
                let mut name = self.hovered_creature().unwrap().name.clone();
                match ev.code {
                    KeyCode::Enter => {
                        self.mode = Mode::Normal;
                    }
                    KeyCode::Esc => {
                        // Revert name
                        name = old_name.clone();
                        self.mode = Mode::Normal;
                    }
                    KeyCode::Backspace => {
                        name = name.chars().take(name.len().saturating_sub(1)).collect();
                    }
                    KeyCode::Char(ch) => {
                        name.push(ch);
                    }

                    _ => {}
                }
                self.hovered_creature_mut().unwrap().name = name;
            }
            // This accepts all key events
            (Mode::EditNotes, _) => match (ev.code, ev.kind) {
                (KeyCode::Esc, KeyEventKind::Press) => {
                    let notes = self.text_area.lines().join("\n");
                    let cursor_pos = self.text_area.cursor();
                    let creature = self.hovered_creature_mut().unwrap();
                    creature.notes = notes;
                    creature.notes_cursor_pos = cursor_pos;
                    self.mode = Mode::Normal;
                }

                _ => {
                    self.text_area.input(ev);
                }
            },
            (Mode::SetHealth(old_amount), KeyEventKind::Press) => {
                let old = *old_amount;
                self.numeric_edit(
                    |creature| creature.health,
                    |creature, value| creature.health = value,
                    |creature| creature.health = old,
                    |_| {},
                    ev,
                );
            }
            (Mode::SetInitiative(old_amount), KeyEventKind::Press) => {
                let old = *old_amount;
                self.numeric_edit(
                    |creature| creature.initiative,
                    |creature, value| creature.initiative = value,
                    |creature| creature.initiative = old,
                    |_| {},
                    ev,
                );
            }
            (Mode::HealthShift, KeyEventKind::Press) => {
                self.numeric_edit(
                    |creature| match creature.health_shift.unwrap() {
                        HealthShift::Increase(mag) | HealthShift::Decrease(mag) => mag as i32,
                    },
                    |creature, value| match creature.health_shift.as_mut().unwrap() {
                        HealthShift::Increase(ref mut mag) | HealthShift::Decrease(ref mut mag) => {
                            *mag = value as u32
                        }
                    },
                    |creature| creature.health_shift = None,
                    |creature| {
                        match creature.health_shift.unwrap() {
                            HealthShift::Increase(mag) => creature.health += mag as i32,
                            HealthShift::Decrease(mag) => creature.health -= mag as i32,
                        }
                        creature.health_shift = None;
                    },
                    ev,
                );
            }
            (Mode::Help, KeyEventKind::Press) => {
                if ev.code == KeyCode::Esc {
                    self.mode = Mode::Normal;
                }
            }
            (Mode::Sort, KeyEventKind::Press) => match ev.code {
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }

                // Initiative
                KeyCode::Char('i') => {
                    self.creatures
                        .sort_by(|a, b| a.initiative.cmp(&b.initiative));
                    self.mode = Mode::Normal;
                }
                KeyCode::Char('I') => {
                    self.creatures
                        .sort_by(|b, a| a.initiative.cmp(&b.initiative));
                    self.mode = Mode::Normal;
                }

                KeyCode::Char('h') => {
                    self.creatures.sort_by(|a, b| a.health.cmp(&b.health));
                    self.mode = Mode::Normal;
                }
                KeyCode::Char('H') => {
                    self.creatures.sort_by(|b, a| a.health.cmp(&b.health));
                    self.mode = Mode::Normal;
                }

                KeyCode::Char('n') => {
                    self.creatures.sort_by(|a, b| a.name.cmp(&b.name));
                    self.mode = Mode::Normal;
                }
                KeyCode::Char('N') => {
                    self.creatures.sort_by(|b, a| a.name.cmp(&b.name));
                    self.mode = Mode::Normal;
                }

                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    fn select_creature(&mut self, index: usize) {
        self.list_state.select(Some(index));
        if let Some(creature) = self.hovered_creature() {
            let (row, col) = creature.notes_cursor_pos;
            self.text_area = TextArea::from(creature.notes.lines());
            self.text_area
                .move_cursor(CursorMove::Jump(row as u16, col as u16));
        }
    }

    fn numeric_edit<T: Clone + Display + Default + FromStr>(
        &mut self,
        extract: impl Fn(&Creature) -> T,
        update: impl Fn(&mut Creature, T),
        revert: impl Fn(&mut Creature),
        commit: impl Fn(&mut Creature),
        ev: event::KeyEvent,
    ) {
        let Some(creature) = self
            .list_state
            .selected()
            .and_then(|index| self.creatures.get_mut(index))
        else {
            panic!("Editing an nonexistent")
        };

        let value = extract(creature);

        match ev.code {
            KeyCode::Enter => {
                commit(creature);
                self.mode = Mode::Normal;
            }
            KeyCode::Esc => {
                revert(creature);
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                let old_amount = value.to_string();
                let new_amount = old_amount
                    .chars()
                    .take(old_amount.len() - 1)
                    .collect::<String>()
                    .parse()
                    .unwrap_or_default();
                update(creature, new_amount);
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                let mut old_amount = value.to_string();
                old_amount.push(ch);

                // If number won't fit, sets to zero
                update(creature, old_amount.parse().unwrap_or_default());
            }

            _ => {}
        }
    }

    fn render_help(&mut self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(8), Constraint::Fill(1)])
            .spacing(1)
            .split(area);

        Paragraph::new(HELP_BLURB).render(main_layout[0], buf);

        let list = List::new(HOTKEYS.iter().flat_map(|hk| match hk {
            HotKey::Divider { text, newline } => {
                let div = Line::from(text.bold());

                if *newline {
                    vec![Line::default(), div]
                } else {
                    vec![div]
                }
            }
            HotKey::Embed { pre, color, post } => vec![Line::from(vec![
                format!("{pre}(").into(),
                color.blue().bold(),
                format!("){post}").into(),
            ])],
            HotKey::Label { label, keys } => {
                vec![Line::from(vec![
                    format!("{label}: ").into(),
                    keys.blue().bold(),
                ])]
            }
        }))
        .block(
            Block::bordered()
                .title(Line::from(" Hotkeys ".bold()).centered())
                .borders(Borders::TOP),
        );
        Widget::render(list, main_layout[1], buf);
    }

    fn render_normal(&mut self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length((self.creatures.len() + 2) as u16),
                Constraint::Fill(1),
            ])
            .spacing(1)
            .split(area);

        // Creature table
        let table_block = Block::bordered()
            .title(Line::from(" Creatures ".bold()).centered())
            .borders(Borders::ALL);

        let table_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(3),  // Initiative
                Constraint::Fill(1),    // Name
                Constraint::Length(10), // Health
                Constraint::Fill(2),    // Statuses
            ])
            .spacing(1)
            .split(table_block.inner(main_layout[0]));
        table_block.render(main_layout[0], buf);

        let selected_index = self.list_state.selected();
        let (initiative_list, name_list, health_list) = self
            .creatures
            .iter()
            .enumerate()
            .map(|(index, creature)| creature.render(index, selected_index))
            .collect::<(Vec<ListItem>, Vec<ListItem>, Vec<ListItem>)>();

        for (column, items) in [initiative_list, name_list, health_list]
            .into_iter()
            .enumerate()
        {
            let list = List::new(items);
            StatefulWidget::render(list, table_layout[column], buf, &mut self.list_state);
        }

        // Notes of selected creature
        let note_block = Block::bordered()
            .title(Line::from(" Notes ".bold()).centered())
            .title_bottom(self.mode.get_instructions().centered())
            .border_set(border::PLAIN);
        self.text_area.render(note_block.inner(main_layout[1]), buf);
        note_block.render(main_layout[1], buf);
    }
}

impl Widget for App<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        if self.mode == Mode::Help {
            self.render_help(area, buf)
        } else {
            self.render_normal(area, buf)
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum HealthShift {
    Increase(u32),
    Decrease(u32),
}

impl FromStr for HealthShift {
    type Err = <i32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numeric: i32 = s.parse()?;

        Ok(if numeric.is_positive() {
            HealthShift::Increase(numeric.try_into().unwrap())
        } else {
            HealthShift::Decrease((-numeric).try_into().unwrap())
        })
    }
}

impl Display for HealthShift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (sign_char, magnitude) = match self {
            HealthShift::Increase(mag) => ('+', mag),
            HealthShift::Decrease(mag) => ('-', mag),
        };
        write!(f, "{}{}", sign_char, magnitude)
    }
}

#[derive(Debug, Clone)]
struct Creature {
    name: String,
    health: i32,
    health_shift: Option<HealthShift>,
    initiative: i32,
    notes: String,
    notes_cursor_pos: (usize, usize),
}

impl Creature {
    fn render(
        &self,
        index: usize,
        selected_index: Option<usize>,
    ) -> (ListItem, ListItem, ListItem) {
        let selected = selected_index == Some(index);

        // Inverse colors when selected
        let (fg_color, bg_color) = if selected {
            (Color::Black, Color::White)
        } else {
            (Color::White, Color::Black)
        };

        let name = if self.name.is_empty() {
            "<empty>".into()
        } else {
            self.name.clone()
        };

        let health = if let Some(health_shift) = self.health_shift {
            format!("{} {}", self.health, health_shift)
        } else {
            self.health.to_string()
        };

        (
            ListItem::from(self.initiative.to_string())
                .fg(fg_color)
                .bg(bg_color),
            ListItem::from(name).fg(fg_color).bg(bg_color),
            ListItem::from(health).fg(fg_color).bg(bg_color),
        )
    }
}

impl Default for Creature {
    fn default() -> Self {
        Creature {
            name: "".into(),
            health: 0,
            health_shift: None,
            initiative: 0,
            notes: "".into(),
            notes_cursor_pos: (0, 0),
        }
    }
}
