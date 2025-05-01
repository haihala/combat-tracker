use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::*,
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget},
    DefaultTerminal, Frame,
};

#[derive(Debug, Clone)]
enum Mode {
    Normal,
}
impl Mode {
    fn get_instructions(&self) -> Line {
        match self {
            Mode::Normal => Line::from(vec![
                " Decrement ".into(),
                "<Left>".blue().bold(),
                " Increment ".into(),
                "<Right>".blue().bold(),
                " Quit ".into(),
                "<Q> ".blue().bold(),
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

        match self.mode {
            Mode::Normal => match ev.code {
                KeyCode::Esc => self.running = false,
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
                _ => {}
            },
        }

        Ok(())
    }
}

impl Widget for App {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Max(((self.creatures.len()) * 2 + 2) as u16),
                Constraint::Fill(1),
            ])
            .spacing(1)
            .split(area);

        // Creature list
        let list_block = Block::bordered()
            .title(Line::from(" Creatures ".bold()).centered())
            .title_bottom(self.mode.get_instructions().centered())
            .borders(Borders::ALL);

        let selected_index = self.list_state.selected();
        let items: Vec<ListItem> = self
            .creatures
            .iter()
            .enumerate()
            .map(|(i, creature)| {
                let color = if selected_index == Some(i) {
                    Color::Green
                } else {
                    Color::White
                };
                ListItem::from(creature.name.clone()).fg(color)
            })
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
                .title_bottom(Line::from(" ctrl+enter to save ".bold()).centered())
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
