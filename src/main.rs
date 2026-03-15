use std::{collections::HashSet, env, io::Error, path::Path};
use crossterm::event::KeyCode;
use git2::{Repository, Status};
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Direction, Layout}, style::Stylize, symbols::border, text::Line, widgets::{Block, Paragraph}};

fn main() -> color_eyre::Result<()> {

    color_eyre::install()?;
    ratatui::run(app)?;

    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {

    let repo = match Repository::open(env::current_dir()?.display().to_string()) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let statuses = match repo.statuses(None) {
        Ok(statuses) => statuses,
        Err(e) => panic!("failed to load statuses: {}", e)
    };

    let not_ignored_statuses: Vec<(String, Status)> = statuses
        .iter()
        .filter(|entry| !entry.status().is_ignored())
        .filter_map(|entry| entry.path().map(|path| (String::from(path), entry.status())))
        .collect();
    
    let mut screen  = AddScreen::init(not_ignored_statuses);

    loop {
        terminal.draw(build_render(&screen))?;

        let event = crossterm::event::read()?;

        if event.is_key_press() {
            match event.as_key_press_event() {
                Some(key_press_event) => match key_press_event.code {
                    KeyCode::Esc => break Ok(()),
                    KeyCode::Down => screen.jump_down(),
                    KeyCode::Up => screen.jump_up(),
                    KeyCode::Char(' ') => screen.toggle_current_line_selection(),
                    KeyCode::Enter => match stage_paths(&repo, eval_paths(&screen)) {
                        Ok(_) => break Ok(()),
                        Err(_) => break Err(Error::other("Couldn't stage all files")),
                    },
                    _ => (),
                },
                None => todo!(),
            }
        }
    }
}

fn build_render(screen: &AddScreen) -> impl Fn(&mut Frame) {
    move |frame| { render(screen, frame) }
}

fn render(screen: &AddScreen, frame: &mut Frame) {
    let title = Line::from("Changes not staged for commit:");

    let unstaged_block = Block::bordered()
        .title(title)
        .border_set(border::EMPTY)
        .title_bottom(Line::from("[Down|Up] Move [Space] Select [Enter] Stage selected [Esc] Quit"));

    let unstaged_lines: Vec<Line> = screen
        .unstaged_changes
        .iter()
        .enumerate()
        .map(|(index, (path, _))| (index, Line::from(vec![if screen.is_line_to_be_staged(&index) { "[+] " } else { "[ ] " }.into(), path.into()])))
        .map(|(index, line)| if index == screen.current_line { line.underlined().bold() } else { line } )
        .map(|line| line.red().into())
        .collect();

    let unstaged_paragraph = Paragraph::new(unstaged_lines)
        .left_aligned()
        .block(unstaged_block);

    let already_staged_changes_lines: Vec<Line> = screen
        .changes
        .iter()
        .filter(|(_, status)| has_staged_change(status))
        .filter_map(|(path, status)| index_label_of(status).map(|status_label| Line::from(vec![path.as_str().into(), " (".into(), status_label.into(), ")".into()])))
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

fn eval_paths(screen: &AddScreen) -> Vec<(String, bool)> {
    screen
        .unstaged_changes
        .iter()
        .enumerate()
        .map(|(line, (path,_))| if screen.is_line_to_be_staged(&line) { (path.clone(), true) } else { (path.clone(), false) })
        .collect()
}

fn stage_paths(repo: &Repository, paths: Vec<(String, bool)>) -> Result<(), git2::Error> {
    let mut index = repo.index()?;
    
    for (path, should_stage) in &paths {
        if *should_stage {
            index.add_path(Path::new(path))?;
        }
    }
    index.write()
}

#[derive(Clone, Debug)]
struct AddScreen {
    changes: Vec<(String, Status)>,
    current_line: usize,
    lines_to_be_staged: HashSet<usize>,
    unstaged_changes: Vec<(String, Status)>,
}

impl AddScreen {
    fn init(changes: Vec<(String, Status)>) -> AddScreen {
        AddScreen {
            lines_to_be_staged: HashSet::new(),
            unstaged_changes: changes.clone().iter().filter(|(_, status)| has_unstaged_changes(status)).map(|change| change.clone()).collect(),
            changes: changes,
            current_line: 0,
        }
    }

    fn jump_up(&mut self) {
        if self.current_line > 0 {
            self.current_line -= 1;
        }
    }

    fn jump_down(&mut self) {
        if self.current_line < self.unstaged_changes.len() - 1 {
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
}

fn has_staged_change(status: &Status) -> bool {
    status.intersects(
        Status::INDEX_NEW
        | Status::INDEX_DELETED
        | Status::INDEX_MODIFIED
        | Status::INDEX_RENAMED
        | Status::INDEX_TYPECHANGE
    )
}

fn has_unstaged_changes(status: &Status) -> bool {
    status.intersects(
        Status::WT_DELETED
        | Status::WT_MODIFIED
        | Status::WT_NEW
        | Status::WT_RENAMED
        | Status::WT_TYPECHANGE
        | Status::WT_UNREADABLE
    )
}

fn index_label_of(status: &Status) -> Option<&'static str> {
    if status.contains(Status::INDEX_NEW) { Some("New") }
    else if status.contains(Status::INDEX_MODIFIED) { Some("Modified") }
    else if status.contains(Status::INDEX_DELETED) { Some("Deleted") }
    else if status.contains(Status::INDEX_RENAMED) { Some("Renamed") }
    else if status.contains(Status::INDEX_TYPECHANGE) { Some("Type changed") }
    else { None }
}