use git2::{Repository, Status};
use ratatui::DefaultTerminal;
use std::{env, io::{Error, ErrorKind}};

use crate::{
    git_extensions::lib::commit_amend,
    screen::{add_screen::AddScreen, commit_screen::CommitScreen, lib::ScreenState},
};

mod git_extensions;
mod screen;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("add") => ratatui::run(add_app)?,
        Some("commit") => ratatui::run(commit_app)?,
        Some("cane") => ratatui::run(commit_amend_no_edit_app)?,
        Some(cmd) => eprintln!("Unknown command: {}", cmd),
        None => {
            eprintln!(
                "Usage: gity <command>\n\nCommands:\n  add      Stage changes interactively\n  commit   Commit staged changes interactively"
            )
        }
    }

    Ok(())
}

fn add_app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let repo = open_repo()?;

    let statuses = match repo.statuses(None) {
        Ok(statuses) => statuses,
        Err(e) => panic!("failed to load statuses: {}", e),
    };

    let not_ignored_statuses: Vec<(String, Status)> = statuses
        .iter()
        .filter(|entry| !entry.status().is_ignored())
        .filter_map(|entry| {
            entry
                .path()
                .map(|path| (String::from(path), entry.status()))
        })
        .collect();

    let mut screen = AddScreen::from(not_ignored_statuses);

    loop {
        terminal.draw(|frame| screen.render(frame))?;

        if screen.handle_event(crossterm::event::read()?, &repo) == ScreenState::Finished {
            break Ok(());
        };
    }
}

fn commit_app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let repo = open_repo()?;
    let mut screen = CommitScreen::new();

    loop {
        terminal.draw(|frame| screen.render(frame))?;

        if screen.handle_event(crossterm::event::read()?, &repo) == ScreenState::Finished {
            break Ok(());
        };
    }
}

fn commit_amend_no_edit_app(_: &mut DefaultTerminal) -> std::io::Result<()> {
    match commit_amend(&open_repo()?, None) {
        Ok(_) => Ok(()),
        Err(err) => Result::Err(Error::new(ErrorKind::Other, err))
    }
}

fn open_repo() -> std::io::Result<Repository> {
    Repository::open(env::current_dir()?.display().to_string())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
