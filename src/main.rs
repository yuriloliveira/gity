use git2::{Repository, Status};
use ratatui::{DefaultTerminal, Frame};
use std::env;

use crate::screen::{add_screen::AddScreen, lib::ScreenState};

mod git_extensions;
mod screen;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("add") => ratatui::run(app)?,
        Some(cmd) => eprintln!("Unknown command: {}", cmd),
        None => {
            eprintln!("Usage: gity <command>\n\nCommands:\n  add    Stage changes interactively")
        }
    }

    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let repo = match Repository::open(env::current_dir()?.display().to_string()) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

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
        terminal.draw(build_render(&screen))?;

        if screen.handle_event(crossterm::event::read()?, &repo) == ScreenState::Finished {
            break Ok(());
        };
    }
}

fn build_render(screen: &AddScreen) -> impl Fn(&mut Frame) {
    move |frame| screen.render(frame)
}
