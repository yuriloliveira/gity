use std::path::Path;

use git2::{Repository, Status};

pub fn has_staged_change(status: &Status) -> bool {
    status.intersects(
        Status::INDEX_NEW
            | Status::INDEX_DELETED
            | Status::INDEX_MODIFIED
            | Status::INDEX_RENAMED
            | Status::INDEX_TYPECHANGE,
    )
}

pub fn has_unstaged_changes(status: &Status) -> bool {
    status.intersects(
        Status::WT_DELETED
            | Status::WT_MODIFIED
            | Status::WT_NEW
            | Status::WT_RENAMED
            | Status::WT_TYPECHANGE
            | Status::WT_UNREADABLE,
    )
}

pub fn index_label_of(status: &Status) -> Option<&'static str> {
    if status.contains(Status::INDEX_NEW) {
        Some("New")
    } else if status.contains(Status::INDEX_MODIFIED) {
        Some("Modified")
    } else if status.contains(Status::INDEX_DELETED) {
        Some("Deleted")
    } else if status.contains(Status::INDEX_RENAMED) {
        Some("Renamed")
    } else if status.contains(Status::INDEX_TYPECHANGE) {
        Some("Type changed")
    } else {
        None
    }
}

pub fn stage_paths(repo: &Repository, paths: Vec<(String, bool)>) -> Result<(), git2::Error> {
    let mut index = repo.index()?;

    for (path, should_stage) in &paths {
        if *should_stage {
            index.add_path(Path::new(path))?;
        }
    }
    index.write()
}
