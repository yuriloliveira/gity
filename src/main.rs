use std::env;
use git2::Repository;

fn main() -> std::io::Result<()> {
    let repo = match Repository::open(env::current_dir()?.display().to_string()) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let statuses = match repo.statuses(None) {
        Ok(statuses) => statuses,
        Err(e) => panic!("failed to load statuses: {}", e)
    };

    println!("Git repo {}", repo.path().display());

    let not_ignored_statuses_iter = statuses.iter().filter(|entry| !entry.status().is_ignored());


    for entry in not_ignored_statuses_iter {
        match entry.path() {
            Some(path) => println!("{}", path),
            None => panic!("Status is none"),
        }
    }

    Ok(())
}