use std::env;
use git2::Repository;
use ratatui::{DefaultTerminal, Frame, symbols::border, text::Line, widgets::{Block, Paragraph}};

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

    let not_ignored_statuses: Vec<String> = statuses
        .iter()
        .filter(|entry| !entry.status().is_ignored())
        .filter_map(|entry| entry.path().map(|path| path.to_string()))
        .collect();
    
    loop {
        terminal.draw(build_render(&not_ignored_statuses))?;

        
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn build_render(statuses: &Vec<String>) -> impl Fn(&mut Frame) {
    move |frame| { render(statuses.clone(), frame) }
}

fn render(statuses: Vec<String>, frame: &mut Frame) {
    let title = Line::from("Changes:");

    let block = Block::bordered()
        .title(title)
        .border_set(border::THICK);

    let lines: Vec<Line> = statuses.iter().map(|path| Line::from(vec!["[ ] ".into(), path.as_str().into()])).collect();

    let paragraph = Paragraph::new(lines)
        .left_aligned()
        .block(block);

    frame.render_widget(paragraph, frame.area());
}