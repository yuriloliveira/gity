use std::{cmp::max, collections::HashSet};

use crossterm::event::KeyCode;
use git2::{Repository, Status};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::{
    git_extensions::lib::{has_staged_change, has_unstaged_changes, index_label_of, stage_paths},
    screen::lib::ScreenState,
};

#[derive(Clone, Debug)]
pub struct AddScreen {
    changes: Vec<(String, Status)>,
    current_line: usize,
    lines_to_be_staged: HashSet<usize>,
    unstaged_changes: Vec<(String, Status)>,
}

impl AddScreen {
    pub fn from(changes: Vec<(String, Status)>) -> AddScreen {
        AddScreen {
            lines_to_be_staged: HashSet::new(),
            unstaged_changes: changes
                .clone()
                .iter()
                .filter(|(_, status)| has_unstaged_changes(status))
                .map(|change| change.clone())
                .collect(),
            changes: changes,
            current_line: 0,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let title = Line::from("Changes not staged for commit:");

        let unstaged_block = Block::bordered()
            .title(title)
            .border_set(border::EMPTY)
            .title_bottom(Line::from(
                "[Down|Up] Move [Space] Select [Enter] Stage selected [Esc] Quit",
            ));

        let unstaged_lines: Vec<Line> = self
            .unstaged_changes
            .iter()
            .enumerate()
            .map(|(index, (path, _))| {
                (
                    index,
                    Line::from(vec![
                        if self.is_line_to_be_staged(&index) {
                            "[+] "
                        } else {
                            "[ ] "
                        }
                        .into(),
                        path.into(),
                    ]),
                )
            })
            .map(|(index, line)| {
                if index == self.current_line {
                    line.underlined().bold()
                } else {
                    line
                }
            })
            .map(|line| line.red().into())
            .collect();

        let unstaged_paragraph = Paragraph::new(unstaged_lines)
            .left_aligned()
            .block(unstaged_block);

        let already_staged_changes_lines: Vec<Line> = self
            .changes
            .iter()
            .filter(|(_, status)| has_staged_change(status))
            .filter_map(|(path, status)| {
                index_label_of(status).map(|status_label| {
                    Line::from(vec![
                        path.as_str().into(),
                        " (".into(),
                        status_label.into(),
                        ")".into(),
                    ])
                })
            })
            .map(|line| line.green())
            .collect();

        let already_staged_changes_block = Block::bordered()
            .title(Line::from("Changes to be committed:"))
            .border_set(border::EMPTY);

        let already_staged_changes_paragraph = Paragraph::new(already_staged_changes_lines)
            .left_aligned()
            .block(already_staged_changes_block);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());

        frame.render_widget(unstaged_paragraph, layout[0]);
        frame.render_widget(already_staged_changes_paragraph, layout[1]);
    }

    fn jump_up(&mut self) {
        if self.current_line > 0 {
            self.current_line -= 1;
        }
    }

    fn jump_down(&mut self) {
        if self.current_line < max(self.unstaged_changes.len(), 1) - 1 {
            self.current_line += 1;
        }
    }

    fn toggle_current_line_selection(&mut self) {
        if !self.lines_to_be_staged.remove(&self.current_line) {
            self.lines_to_be_staged.insert(self.current_line);
        }
    }

    fn is_line_to_be_staged(&self, line: &usize) -> bool {
        self.lines_to_be_staged.contains(line)
    }

    pub fn handle_event(
        &mut self,
        event: crossterm::event::Event,
        repo: &Repository,
    ) -> ScreenState {
        if event.is_key_press() {
            match event.as_key_press_event() {
                Some(key_press_event) => match key_press_event.code {
                    KeyCode::Esc => ScreenState::Finished,
                    KeyCode::Down => {
                        self.jump_down();
                        ScreenState::Active
                    }
                    KeyCode::Up => {
                        self.jump_up();
                        ScreenState::Active
                    }
                    KeyCode::Char(' ') => {
                        self.toggle_current_line_selection();
                        ScreenState::Active
                    }
                    KeyCode::Enter => match stage_paths(repo, self.eval_paths()) {
                        Ok(_) => ScreenState::Finished,
                        Err(_) => panic!("Couldn't stage all files"),
                    },
                    _ => ScreenState::Active,
                },
                None => ScreenState::Active,
            }
        } else {
            ScreenState::Active
        }
    }

    fn eval_paths(&self) -> Vec<(String, bool)> {
        self.unstaged_changes
            .iter()
            .enumerate()
            .map(|(line, (path, _))| {
                if self.is_line_to_be_staged(&line) {
                    (path.clone(), true)
                } else {
                    (path.clone(), false)
                }
            })
            .collect()
    }
}
