//! Piper-style backend stub.
//!
//! Returns `ScmError::NotImplemented` for every operation — the trait is
//! satisfied so Lacoder's code paths (CL request, submit, pipeline polling)
//! can be exercised end-to-end against a workspace that thinks it lives in
//! an internal monorepo, without the IDE pretending to support what it
//! actually doesn't. Real Piper / similar CLIs need corp auth (Kerberos,
//! LOAS) which is intentionally out of scope here.

use std::path::PathBuf;

use url::Url;

use crate::{
    ScmBackend,
    types::{
        ChangeDraft, ChangeRef, DiffScope, FileStatus, PipelineStatus,
        ScmError, ScmKind, SubmitOpts, SubmitOutcome, UnifiedDiff,
    },
};

pub struct PiperBackend {
    #[allow(dead_code)]
    root: PathBuf,
}

impl PiperBackend {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

const STUB_OP: &str = "piper backend (stub)";

impl ScmBackend for PiperBackend {
    fn id(&self) -> ScmKind {
        ScmKind::Piper
    }

    fn status(&self) -> Result<Vec<FileStatus>, ScmError> {
        Err(ScmError::NotImplemented {
            backend: ScmKind::Piper,
            operation: STUB_OP,
        })
    }

    fn diff(&self, _scope: DiffScope) -> Result<UnifiedDiff, ScmError> {
        Err(ScmError::NotImplemented {
            backend: ScmKind::Piper,
            operation: STUB_OP,
        })
    }

    fn current_change(&self) -> Result<Option<ChangeRef>, ScmError> {
        Ok(None)
    }

    fn create_change(&self, _draft: ChangeDraft) -> Result<ChangeRef, ScmError> {
        Err(ScmError::NotImplemented {
            backend: ScmKind::Piper,
            operation: STUB_OP,
        })
    }

    fn update_change(
        &self,
        _change: &ChangeRef,
        _draft: ChangeDraft,
    ) -> Result<(), ScmError> {
        Err(ScmError::NotImplemented {
            backend: ScmKind::Piper,
            operation: STUB_OP,
        })
    }

    fn submit(
        &self,
        _change: &ChangeRef,
        _opts: SubmitOpts,
    ) -> Result<SubmitOutcome, ScmError> {
        Err(ScmError::NotImplemented {
            backend: ScmKind::Piper,
            operation: STUB_OP,
        })
    }

    fn open_url(&self, _change: &ChangeRef) -> Option<Url> {
        None
    }

    fn fetch_pipeline(
        &self,
        _change: &ChangeRef,
    ) -> Result<PipelineStatus, ScmError> {
        Ok(PipelineStatus::Unknown)
    }
}
