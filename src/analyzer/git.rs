// src/git_inspector.rs
use git2::{BlameOptions, Commit, Oid, Repository, StatusOptions, Time};

fn extract_path_from_src(path: &Path) -> Option<String> {
    let delimiter = "src/".to_string();
    path.to_str()
        .unwrap_or("")
        .split_once(&delimiter)
        .map(|(_, rest)| format!("src/{}", rest)) // return owned String
}

fn extract_repository_path(path: &Path) -> Option<String> {
    let delimiter = "/src".to_string();
    path.to_str()
        .unwrap_or("")
        .split_once(&delimiter)
        .map(|(repo, _)| format!("{}", repo))
}

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct GitInspector {
    pub(crate) repo: Repository,
}

impl GitInspector {
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Result<Self, git2::Error> {
        let repo_path_str = extract_repository_path(&repo_path.as_ref())
            .ok_or_else(|| git2::Error::from_str("Invalid repository path"))?;
        let repo: Repository = Repository::discover(Path::new(&repo_path_str))?;
        Ok(GitInspector { repo })
    }

    pub fn list_changed_files(&self) -> Result<Vec<PathBuf>, git2::Error> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true).recurse_untracked_dirs(true);

        let statuses = self.repo.statuses(Some(&mut opts))?;
        let changed_files = statuses
            .iter()
            .filter_map(|entry| entry.path().map(PathBuf::from))
            .collect();

        Ok(changed_files)
    }

    pub fn is_file_changed<P: AsRef<Path>>(&self, path: P) -> Result<bool, git2::Error> {
        let changed_files = self.list_changed_files()?;
        Ok(changed_files.contains(&path.as_ref().to_path_buf()))
    }

    pub fn is_file_stale<P: AsRef<Path>>(
        &self,
        path: P,
        stale_days: u64,
    ) -> Result<bool, git2::Error> {
        let relative_path = extract_path_from_src(path.as_ref())
            .unwrap_or_else(|| path.as_ref().to_str().unwrap_or("").to_string());
        let blame = self
            .repo
            .blame_file(relative_path.as_ref(), Some(&mut BlameOptions::new()))?;
        let mut latest_time = 0;

        for hunk in blame.iter() {
            let commit = self.repo.find_commit(hunk.final_commit_id())?;
            let time = commit.time().seconds();
            if time > latest_time {
                latest_time = time;
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let age_days = (now - latest_time) / 86400;
        Ok(age_days as u64 > stale_days)
    }

    pub fn file_commit_authors<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<HashMap<String, u32>, git2::Error> {
        let relative_path = extract_path_from_src(path.as_ref())
            .unwrap_or_else(|| path.as_ref().to_string_lossy().to_string());
        let blame = self
            .repo
            .blame_file(relative_path.as_ref(), Some(&mut BlameOptions::new()))?;
        let mut authors = HashMap::new();

        for hunk in blame.iter() {
            let commit = self.repo.find_commit(hunk.final_commit_id())?;
            let author = commit.author().name().unwrap_or("Unknown").to_string();
            *authors.entry(author).or_insert(0) += 1;
        }

        Ok(authors)
    }

    pub fn most_frequent_author<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Option<String>, git2::Error> {
        let authors = self.file_commit_authors(path)?;
        let most = authors.into_iter().max_by_key(|(_, count)| *count);
        Ok(most.map(|(author, _)| author))
    }
}

// --- Integration Example ---
// if git_inspector.is_file_changed(file)? {
//     warn!("⚠️ This file has uncommitted changes.");
// }
// if git_inspector.is_file_stale(file, 30)? {
//     warn!("🕰️ This file hasn't been touched in over 30 days.");
// }
// if let Some(author) = git_inspector.most_frequent_author(file)? {
//     info!("🧙 Most edits on this file were made by: {}", author);
// }
