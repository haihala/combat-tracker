use std::io;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
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
    DamageCreature,
    HealCreature,
    EditNotes(String),
}
impl Mode {
    fn get_instructions(&self) -> Line {
        match self {
            Mode::Normal => Line::from(vec![
                " Move: ".into(),
                "hjkl".blue().bold(),
                ", Exit: ".into(),
                "Esc".blue().bold(),
                ", Actions: (".into(),
                "A".blue().bold(),
                ")dd, (".into(),
                "R".blue().bold(),
                ")ename, (".into(),
                "D".blue().bold(),
                ")estroy ".into(),
            ]),
            Mode::Rename(_) => Line::from(vec![
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
                    damage: 0,
                    health: Some(5),
                    quantity: 2,
                    notes: "Very gobliny".into(),
                },
                Creature {
                    name: "Chodlin".into(),
                    damage: 0,
                    health: Some(4),
                    quantity: 1,
                    notes: "Cousin of Boblin".into(),
                },
                Creature {
                    name: "Boblin".into(),
                    damage: 0,
                    health: Some(4),
                    quantity: 1,
                    notes: "The goblin".into(),
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

        match &self.mode {
            Mode::Normal => match ev.code {
                // Quitting
                KeyCode::Esc => self.running = false,

                // Navigation
                KeyCode::Char('h') => self.list_state.select_first(),
                KeyCode::Char('j') => self.list_state.select(Some(
                    (self
                        .list_state
                        .selected()
                        .map(|num| num + 1)
                        .unwrap_or_default())
                        % self.creatures.len(),
                )),
                KeyCode::Char('k') => self.list_state.select(Some({
                    let curr = self.list_state.selected().unwrap_or_default();
                    if curr == 0 {
                        self.creatures.len() - 1
                    } else {
                        curr - 1
                    }
                })),
                KeyCode::Char('l') => self.list_state.select(Some(self.creatures.len() - 1)),

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

            _ => {}
        }

        Ok(())
    }
}

impl Widget for App {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Max((self.creatures.len() + 2) as u16),
                Constraint::Fill(1),
            ])
            .spacing(1)
            .split(area);

        // Creature list
        let list_block = Block::bordered()
            .title(Line::from(" Creatures ".bold()).centered())
            .borders(Borders::ALL);

        let selected_index = self.list_state.selected();
        let items: Vec<ListItem> = self
            .creatures
            .iter()
            .enumerate()
            .map(|(i, creature)| creature.render(i, selected_index))
            .collect();
        let list = List::new(items).block(list_block);
        StatefulWidget::render(list, layout[0], buf, &mut self.list_state);

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
        .render(layout[1], buf);
    }
}

#[derive(Debug, Clone)]
struct Creature {
    name: String,
    quantity: usize,
    damage: usize,
    health: Option<usize>,
    notes: String,
}

impl Creature {
    fn render(&self, index: usize, selected_index: Option<usize>) -> ListItem {
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

        ListItem::from(name).fg(fg_color).bg(bg_color)
    }
}

impl Default for Creature {
    fn default() -> Self {
        Creature {
            name: "".into(),
            quantity: 1,
            damage: 0,
            health: None,
            notes: "".into(),
        }
    }
}
