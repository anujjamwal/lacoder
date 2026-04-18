//! Smoke test: backend detection + status against the lacoder repo.

use std::path::PathBuf;

use lapce_scm::{ScmKind, detect};

#[test]
fn detects_github_for_lacoder_repo() {
    // Tests run from the workspace root or the crate dir; walk up either way.
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if !root.join(".git").exists() {
        root = root.parent().unwrap().to_owned();
    }

    let backend = detect(&root).expect("detect should find a backend");
    assert_eq!(backend.id(), ScmKind::GitHub);

    let _ = backend.status().expect("status read should not error");
    let head = backend
        .current_change()
        .expect("current_change read")
        .expect("a HEAD ref should exist");
    assert_eq!(head.backend, ScmKind::GitHub);
    assert!(!head.id.is_empty());
}
