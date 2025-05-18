use std::io;

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

#[derive(Debug, Clone)]
enum Mode {
    Normal,
    Rename(String),
    SetHealth(i32),
    SetInitiative(i32),
    DamageCreature,
    HealCreature,
    EditNotes(String),
}
impl Mode {
    fn get_instructions(&self) -> Line {
        match self {
            Mode::Normal => Line::from(vec![
                " Move: ".into(),
                "JjkK".blue().bold(),
                ", Exit: ".into(),
                "Esc".blue().bold(),
                ", Actions: (".into(),
                "A".blue().bold(),
                ")dd, (".into(),
                "R".blue().bold(),
                ")ename, (".into(),
                "C".blue().bold(),
                ")opy, (".into(),
                "D".blue().bold(),
                ")estroy (".into(),
                "H".blue().bold(),
                ")health (".into(),
                "I".blue().bold(),
                ")nitiative ".into(),
            ]),
            Mode::Rename(_) | Mode::SetHealth(_) | Mode::SetInitiative(_) => Line::from(vec![
                " Confirm: ".into(),
                "Enter".blue().bold(),
                ", Cancel: ".into(),
                "Esc ".blue().bold(),
            ]),
            Mode::EditNotes(_) => Line::from(vec![
                " Confirm: ".into(),
                "Enter".blue().bold(),
                " (use alt to break lines), Cancel: ".into(),
                "Esc ".blue().bold(),
            ]),
            _ => Line::from(vec![]),
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
                self.numeric_edit(|creature| &mut creature.health, ev, *old_amount);
            }

            Mode::SetInitiative(old_amount) => {
                self.numeric_edit(|creature| &mut creature.initiative, ev, *old_amount);
            }

            _ => {}
        }

        Ok(())
    }

    fn numeric_edit(
        &mut self,
        extract: impl Fn(&mut Creature) -> &mut i32,
        ev: event::KeyEvent,
        old_amount: i32,
    ) {
        let Some(creature) = self
            .list_state
            .selected()
            .and_then(|index| self.creatures.get_mut(index))
        else {
            panic!("Editing an nonexistent")
        };

        let attribute = extract(creature);

        match ev.code {
            KeyCode::Enter => {
                self.mode = Mode::Normal;
            }
            KeyCode::Esc => {
                // Revert name
                *attribute = old_amount.clone();
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                let old_amount = attribute.to_string();
                *attribute = old_amount
                    .chars()
                    .take(old_amount.len() - 1)
                    .collect::<String>()
                    .parse()
                    .unwrap_or_default();
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                let mut old_amount = attribute.to_string();
                old_amount.push(ch);

                // If number won't fit, sets to zero
                *attribute = old_amount.parse().unwrap_or_default();
            }

            _ => {}
        }
    }
}

impl Widget for App {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Max((self.creatures.len() + 2) as u16),
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
                Constraint::Max(5),  // Initiative
                Constraint::Fill(1), // Name
                Constraint::Max(5),  // Health
                Constraint::Fill(2), // Statuses
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

#[derive(Debug, Clone)]
struct Creature {
    name: String,
    health: i32,
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

        (
            ListItem::from(self.initiative.to_string())
                .fg(fg_color)
                .bg(bg_color),
            ListItem::from(name).fg(fg_color).bg(bg_color),
            ListItem::from(self.health.to_string())
                .fg(fg_color)
                .bg(bg_color),
        )
    }
}

impl Default for Creature {
    fn default() -> Self {
        Creature {
            name: "".into(),
            health: 0,
            initiative: 0,
            notes: "".into(),
        }
    }
}
