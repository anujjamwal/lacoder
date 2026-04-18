//! GitHub backend — git2 for local ops; PR creation/submit deferred.
//!
//! Phase 4.0 ships local read ops (`status`, `diff`, `current_change`) over
//! `git2`. PR creation and submit are stubbed with `ScmError::NotImplemented`
//! until the auth surface (`gh` CLI vs. PAT vs. OAuth) is decided in a
//! follow-up.

use std::path::{Path, PathBuf};

use git2::{Repository, StatusOptions};
use url::Url;

use crate::{
    ScmBackend,
    types::{
        ChangeDraft, ChangeRef, DiffScope, FileStatus, FileStatusKind,
        PipelineStatus, ScmError, ScmKind, SubmitOpts, SubmitOutcome,
        UnifiedDiff,
    },
};

pub struct GitHubBackend {
    root: PathBuf,
}

impl GitHubBackend {
    pub fn open(root: &Path) -> Result<Self, ScmError> {
        // Verify the repository can be opened — surface the error early
        // instead of letting it bubble out of every method.
        let _ = Repository::discover(root)?;
        Ok(Self {
            root: root.to_owned(),
        })
    }

    fn repo(&self) -> Result<Repository, ScmError> {
        Repository::discover(&self.root).map_err(Into::into)
    }
}

impl ScmBackend for GitHubBackend {
    fn id(&self) -> ScmKind {
        ScmKind::GitHub
    }

    fn status(&self) -> Result<Vec<FileStatus>, ScmError> {
        let repo = self.repo()?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(true).recurse_untracked_dirs(true);
        let statuses = repo.statuses(Some(&mut opts))?;

        Ok(statuses
            .iter()
            .filter_map(|entry| {
                let path = entry.path()?.to_string();
                let kind = map_status(entry.status());
                Some(FileStatus {
                    path: PathBuf::from(path),
                    kind,
                })
            })
            .collect())
    }

    fn diff(&self, scope: DiffScope) -> Result<UnifiedDiff, ScmError> {
        let repo = self.repo()?;
        let diff = match scope {
            DiffScope::WorkingTree => {
                let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
                repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), None)?
            }
            DiffScope::BranchSince { against } => {
                let base_oid = repo
                    .revparse_single(&against)?
                    .peel_to_commit()?
                    .tree_id();
                let base_tree = repo.find_tree(base_oid)?;
                let head_tree = repo.head()?.peel_to_tree()?;
                repo.diff_tree_to_tree(Some(&base_tree), Some(&head_tree), None)?
            }
            DiffScope::AgentSession(_) => {
                // Mapping a session id to a branch namespace is out of scope
                // for Phase 4.0; agent sessions land their work on a branch
                // that callers should pass via BranchSince.
                return Err(ScmError::NotImplemented {
                    backend: ScmKind::GitHub,
                    operation: "diff(AgentSession)",
                });
            }
        };

        let stats = diff.stats()?;
        let mut text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            if matches!(
                line.origin(),
                ' ' | '+' | '-' | 'F' | 'H' | '=' | '<' | '>'
            ) {
                text.push(line.origin());
            }
            if let Ok(s) = std::str::from_utf8(line.content()) {
                text.push_str(s);
            }
            true
        })?;

        Ok(UnifiedDiff {
            text,
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        })
    }

    fn current_change(&self) -> Result<Option<ChangeRef>, ScmError> {
        let repo = self.repo()?;
        let head = repo.head()?;
        let branch = head
            .shorthand()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "HEAD".to_string());
        Ok(Some(ChangeRef {
            backend: ScmKind::GitHub,
            id: branch.clone(),
            title: branch,
        }))
    }

    fn create_change(&self, _draft: ChangeDraft) -> Result<ChangeRef, ScmError> {
        Err(ScmError::NotImplemented {
            backend: ScmKind::GitHub,
            operation: "create_change (PR creation requires gh CLI / PAT — wired in follow-up)",
        })
    }

    fn update_change(
        &self,
        _change: &ChangeRef,
        _draft: ChangeDraft,
    ) -> Result<(), ScmError> {
        Err(ScmError::NotImplemented {
            backend: ScmKind::GitHub,
            operation: "update_change",
        })
    }

    fn submit(
        &self,
        _change: &ChangeRef,
        _opts: SubmitOpts,
    ) -> Result<SubmitOutcome, ScmError> {
        Err(ScmError::NotImplemented {
            backend: ScmKind::GitHub,
            operation: "submit",
        })
    }

    fn open_url(&self, _change: &ChangeRef) -> Option<Url> {
        // Without remote-URL parsing here, leave `None`; consumers can derive
        // from `git remote get-url origin` when needed.
        None
    }

    fn fetch_pipeline(
        &self,
        _change: &ChangeRef,
    ) -> Result<PipelineStatus, ScmError> {
        // Pipeline status requires the GitHub Actions API; deferred with the
        // PR creation flow.
        Ok(PipelineStatus::Unknown)
    }
}

fn map_status(s: git2::Status) -> FileStatusKind {
    use git2::Status;
    if s.contains(Status::CONFLICTED) {
        FileStatusKind::Conflicted
    } else if s.contains(Status::WT_NEW) || s.contains(Status::INDEX_NEW) {
        FileStatusKind::Added
    } else if s.contains(Status::WT_DELETED) || s.contains(Status::INDEX_DELETED)
    {
        FileStatusKind::Deleted
    } else if s.contains(Status::WT_RENAMED) || s.contains(Status::INDEX_RENAMED)
    {
        FileStatusKind::Renamed
    } else if s.contains(Status::WT_MODIFIED) || s.contains(Status::INDEX_MODIFIED)
    {
        FileStatusKind::Modified
    } else {
        FileStatusKind::Untracked
    }
}
