use std::{fmt::Display, io, str::FromStr};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use log::info;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::*,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget},
    DefaultTerminal,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    Help,
    Normal,
    Rename(String),
    SetHealth(i32),
    SetInitiative(i32),
    HealthShift,
    EditNotes(String),
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
            Mode::EditNotes(_) => Line::from(vec![
                " Confirm: ".into(),
                "Enter".blue().bold(),
                " (use alt to break lines), Cancel: ".into(),
                "Esc ".blue().bold(),
            ]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct App {
    running: bool,
    mode: Mode,
    list_state: ListState,
    creatures: Vec<Creature>,
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

const HELP_BLURB: &'static str = "\
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
        text: "In help mode",
        newline: true,
    },
    HotKey::Label {
        label: "Return to normal mode",
        keys: "Esc",
    },
];

impl App {
    pub fn new() -> App {
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

    fn read_events(&mut self) -> io::Result<()> {
        let Event::Key(ev) = event::read()? else {
            return Ok(());
        };

        if !ev.is_press() {
            return Ok(());
        }

        let hovered_creature = self
            .list_state
            .selected()
            .and_then(|index| self.creatures.get_mut(index));

        info!("Key press - {:?}", ev);

        match &self.mode {
            Mode::Normal => match ev.code {
                // Quitting
                KeyCode::Esc => self.running = false,

                KeyCode::Char('?') => self.mode = Mode::Help,

                // Navigation
                KeyCode::Char('K') => self.list_state.select_first(),
                KeyCode::Char('k') => self.list_state.select(Some({
                    let curr = self.list_state.selected().unwrap_or_default();
                    if curr == 0 {
                        self.creatures.len() - 1
                    } else {
                        curr - 1
                    }
                })),
                KeyCode::Char('j') => self.list_state.select(Some(
                    (self
                        .list_state
                        .selected()
                        .map(|num| num + 1)
                        .unwrap_or_default())
                        % self.creatures.len(),
                )),
                KeyCode::Char('J') => self.list_state.select(Some(self.creatures.len() - 1)),

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
                    if let Some(creat) = hovered_creature {
                        self.mode = Mode::Rename(creat.name.clone());
                    }
                }
                KeyCode::Char('n') => {
                    if let Some(creat) = hovered_creature {
                        self.mode = Mode::EditNotes(creat.notes.clone());
                    }
                }
                KeyCode::Char('c') => {
                    // TODO: Think about automatically renaming with indices or something
                    if let Some(hovered) = hovered_creature {
                        let index = self.list_state.selected().unwrap();
                        let duplicate = hovered.clone();
                        self.creatures.insert(index + 1, duplicate);
                    }
                }
                KeyCode::Char('d') => {
                    if hovered_creature.is_some() {
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
                    if let Some(creat) = hovered_creature {
                        self.mode = Mode::SetHealth(creat.health);
                    }
                }
                KeyCode::Char('i') => {
                    if let Some(creat) = hovered_creature {
                        self.mode = Mode::SetInitiative(creat.initiative);
                    }
                }
                KeyCode::Char('-') => {
                    if let Some(creature) = hovered_creature {
                        creature.health_shift = Some(HealthShift::Decrease(0));
                        self.mode = Mode::HealthShift;
                    }
                }
                KeyCode::Char('+') => {
                    if let Some(creature) = hovered_creature {
                        creature.health_shift = Some(HealthShift::Increase(0));
                        self.mode = Mode::HealthShift;
                    }
                }
                _ => {}
            },
            Mode::Rename(old_name) => {
                let selected_creature = hovered_creature.unwrap();
                match ev.code {
                    KeyCode::Enter => {
                        self.mode = Mode::Normal;
                    }
                    KeyCode::Esc => {
                        // Revert name
                        selected_creature.name = old_name.clone();
                        self.mode = Mode::Normal;
                    }
                    KeyCode::Backspace => {
                        selected_creature.name = selected_creature
                            .name
                            .chars()
                            .take(selected_creature.name.len().saturating_sub(1))
                            .collect();
                    }
                    KeyCode::Char(ch) => {
                        selected_creature.name.push(ch);
                    }

                    _ => {}
                }
            }
            Mode::EditNotes(old_content) => {
                let selected_creature = hovered_creature.unwrap();
                match ev.code {
                    KeyCode::Enter => {
                        // This doesn't work for some reason
                        if ev.modifiers.contains(KeyModifiers::ALT) {
                            selected_creature.notes.push('\n');
                        } else {
                            self.mode = Mode::Normal;
                        }
                    }
                    KeyCode::Esc => {
                        // Revert name
                        selected_creature.notes = old_content.clone();
                        self.mode = Mode::Normal;
                    }
                    KeyCode::Backspace => {
                        selected_creature.notes = selected_creature
                            .name
                            .chars()
                            .take(selected_creature.notes.len().saturating_sub(1))
                            .collect();
                    }
                    KeyCode::Char(ch) => {
                        selected_creature.notes.push(ch);
                    }

                    _ => {}
                }
            }
            Mode::SetHealth(old_amount) => {
                let old = old_amount.clone();
                self.numeric_edit(
                    |creature| creature.health,
                    |creature, value| creature.health = value,
                    |creature| creature.health = old,
                    |_| {},
                    ev,
                );
            }
            Mode::SetInitiative(old_amount) => {
                let old = old_amount.clone();
                self.numeric_edit(
                    |creature| creature.initiative,
                    |creature, value| creature.initiative = value,
                    |creature| creature.initiative = old,
                    |_| {},
                    ev,
                );
            }
            Mode::HealthShift => {
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
            Mode::Help => {
                if ev.code == KeyCode::Esc {
                    self.mode = Mode::Normal;
                }
            }
        }

        Ok(())
    }

    fn numeric_edit<T: Clone + Display + Default + FromStr>(
        &mut self,
        extract: impl Fn(&Creature) -> T,
        update: impl Fn(&mut Creature, T) -> (),
        revert: impl Fn(&mut Creature) -> (),
        commit: impl Fn(&mut Creature) -> (),
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

        let list = List::new(HOTKEYS.into_iter().flat_map(|hk| match hk {
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
        Paragraph::new(
            selected_index
                .and_then(|index| self.creatures.get(index))
                .map(|creature| creature.notes.clone())
                .unwrap_or_default(),
        )
        .block(
            Block::bordered()
                .title(Line::from(" Notes ".bold()).centered())
                .title_bottom(self.mode.get_instructions().centered())
                .border_set(border::PLAIN),
        )
        .render(main_layout[1], buf);
    }
}

impl Widget for App {
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
            format!("{} {}", self.health, health_shift.to_string())
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
        }
    }
}
