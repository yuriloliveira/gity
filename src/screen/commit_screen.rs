use crossterm::event::KeyCode;
use git2::Repository;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::{git_extensions::lib::commit, screen::lib::ScreenState};

#[derive(PartialEq, Eq, Clone, Debug)]
enum FocusedField {
    Subject,
    Body,
}

#[derive(Clone, Debug)]
pub struct CommitScreen {
    subject: String,
    body: String,
    focused: FocusedField,
}

impl CommitScreen {
    pub fn new() -> CommitScreen {
        CommitScreen {
            subject: String::new(),
            body: String::new(),
            focused: FocusedField::Subject,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Min(5)])
            .split(frame.area());

        let subject_len = self.subject.len();
        let subject_color = if subject_len > 75 {
            Color::Red
        } else if subject_len > 50 {
            Color::Yellow
        } else {
            Color::Reset
        };

        let subject_focused = self.focused == FocusedField::Subject;
        let subject_block = Block::bordered()
            .border_set(if subject_focused {
                border::THICK
            } else {
                border::PLAIN
            })
            .title(Line::from(" Commit message "))
            .title_bottom(
                Line::from(format!(
                    " [Tab] Detailed message [Enter] Commit [Esc] Quit ({} characters) ",
                    subject_len
                ))
                .right_aligned(),
            );

        let subject_paragraph = Paragraph::new(self.subject.as_str())
            .style(Style::default().fg(subject_color))
            .block(subject_block);

        let body_focused = self.focused == FocusedField::Body;
        let body_block = Block::bordered()
            .border_set(if body_focused {
                border::THICK
            } else {
                border::PLAIN
            })
            .title(Line::from(" Detailed message "))
            .title_bottom(Line::from(" [Tab] Commit message [Esc] Quit ").right_aligned());

        let body_paragraph = Paragraph::new(self.body.as_str()).block(body_block);

        frame.render_widget(subject_paragraph, layout[0]);
        frame.render_widget(body_paragraph, layout[1]);
    }

    pub fn handle_event(
        &mut self,
        event: crossterm::event::Event,
        repo: &Repository,
    ) -> ScreenState {
        if !event.is_key_press() {
            return ScreenState::Active;
        }

        let Some(key) = event.as_key_press_event() else {
            return ScreenState::Active;
        };

        match key.code {
            KeyCode::Esc => ScreenState::Finished,
            KeyCode::Tab => {
                self.focused = match self.focused {
                    FocusedField::Subject => FocusedField::Body,
                    FocusedField::Body => FocusedField::Subject,
                };
                ScreenState::Active
            }
            KeyCode::Enter => match self.focused {
                FocusedField::Subject => self.try_commit(repo),
                FocusedField::Body => {
                    self.body.push('\n');
                    ScreenState::Active
                }
            },
            KeyCode::Backspace => {
                match self.focused {
                    FocusedField::Subject => {
                        self.subject.pop();
                    }
                    FocusedField::Body => {
                        self.body.pop();
                    }
                }
                ScreenState::Active
            }
            KeyCode::Char(c) => {
                match self.focused {
                    FocusedField::Subject => self.subject.push(c),
                    FocusedField::Body => self.body.push(c),
                }
                ScreenState::Active
            }
            _ => ScreenState::Active,
        }
    }

    fn try_commit(&self, repo: &Repository) -> ScreenState {
        if self.subject.is_empty() {
            return ScreenState::Active;
        }

        let message = if self.body.is_empty() {
            self.subject.clone()
        } else {
            format!("{}\n\n{}", self.subject, self.body)
        };

        match commit(repo, &message) {
            Ok(_) => ScreenState::Finished,
            Err(_) => panic!("Couldn't create commit"),
        }
    }
}
