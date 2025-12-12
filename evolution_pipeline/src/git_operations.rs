//! Git operations helpers for the GitHub-first creation enforcement pipeline.
//!
//! Notes:
//! - These functions use `git2` (libgit2) so they work without invoking the `git` CLI.
//! - Authentication is performed using a GitHub Personal Access Token (PAT) as the password
//!   with username `x-access-token`.

use git2::{Cred, IndexAddOption, PushOptions, RemoteCallbacks, Repository};
use std::path::Path;

use crate::github_enforcement::CreationError;

fn remote_callbacks_with_pat(pat: &str) -> RemoteCallbacks<'static> {
    let token = pat.to_string();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_, _, _| Cred::userpass_plaintext("x-access-token", &token));
    callbacks
}

fn open_repo(code_path: &Path) -> Result<Repository, CreationError> {
    Ok(Repository::open(code_path)?)
}

/// Stage all changes and commit.
///
/// If there is no HEAD yet (fresh repo), this will create an initial commit.
pub fn commit_all(code_path: &Path, message: &str) -> Result<(), CreationError> {
    let repo = open_repo(code_path)?;

    let mut index = repo.index()?;
    index.add_all(["*"], IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let sig = repo
        .signature()
        .or_else(|_| git2::Signature::now("Phoenix 2.0", "phoenix@eternal.agi"))?;

    // Determine parent commit(s) if HEAD exists.
    let parents = match repo.head() {
        Ok(h) => {
            let parent = h.peel_to_commit()?;
            vec![parent]
        }
        Err(_) => vec![],
    };
    let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        message,
        &tree,
        &parent_refs,
    )?;

    Ok(())
}

/// Create a local branch at HEAD, checkout it, and push it to the remote `origin`.
pub fn create_and_push_branch(code_path: &Path, branch: &str, pat: &str) -> Result<(), CreationError> {
    let repo = open_repo(code_path)?;

    let head_commit = repo
        .head()?
        .peel_to_commit()
        .map_err(|_| CreationError::Git(git2::Error::from_str("HEAD is not a commit")))?;

    // Create/update local branch.
    repo.branch(branch, &head_commit, true)?;
    repo.set_head(&format!("refs/heads/{branch}"))?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::new()
            .force()
            .remove_untracked(true),
    ))?;

    let mut remote = repo.find_remote("origin")?;
    let callbacks = remote_callbacks_with_pat(pat);
    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(callbacks);

    remote.push(
        &[&format!(
            "refs/heads/{branch}:refs/heads/{branch}",
            branch = branch
        )],
        Some(&mut push_opts),
    )?;

    Ok(())
}

/// Checkout `main` and hard-reset it to `origin/main` (best-effort pull).
pub fn checkout_and_pull_main(code_path: &Path) -> Result<(), CreationError> {
    let repo = open_repo(code_path)?;
    let main_ref = "refs/heads/main";

    // Checkout main locally (create if needed).
    if repo.find_reference(main_ref).is_err() {
        // Try to create from origin/main.
        let origin_main = repo
            .find_reference("refs/remotes/origin/main")
            .or_else(|_| repo.find_reference("refs/remotes/origin/master"))?;
        let oid = origin_main
            .target()
            .ok_or_else(|| CreationError::Other("origin/main has no target".to_string()))?;
        let commit = repo.find_commit(oid)?;
        repo.branch("main", &commit, true)?;
    }
    repo.set_head(main_ref)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;

    // Fetch origin.
    let mut remote = repo.find_remote("origin")?;
    remote.fetch(&["main"], None, None)?;

    // Reset to origin/main.
    let origin_main = repo
        .find_reference("refs/remotes/origin/main")
        .or_else(|_| repo.find_reference("refs/remotes/origin/master"))?;
    let oid = origin_main
        .target()
        .ok_or_else(|| CreationError::Other("origin/main has no target".to_string()))?;
    let obj = repo.find_object(oid, None)?;
    repo.reset(&obj, git2::ResetType::Hard, None)?;

    Ok(())
}

