// skill_system/src/folder_loader.rs
// Load skills from JSON files in organized folder structure

use std::fs;
use std::path::{Path, PathBuf};
use serde_json;
use uuid::Uuid;

use crate::{SkillDefinition, SkillLibrary};

/// Load skills from a directory structure.
///
/// Expected structure:
/// ```text
/// skills/
///   intimate/
///     skill1.json
///     skill2.json
///   passion/
///     skill1.json
///   fantasy/
///     skill1.json
/// ```
pub fn load_skills_from_folder(
    lib: &mut SkillLibrary,
    base_path: &str,
) -> Result<LoadResult, String> {
    let base = Path::new(base_path);
    if !base.exists() {
        return Ok(LoadResult {
            loaded: 0,
            failed: 0,
            errors: vec![format!("Skills folder does not exist: {}", base_path)],
        });
    }

    let mut result = LoadResult {
        loaded: 0,
        failed: 0,
        errors: Vec::new(),
    };

    // Load from root directory (any .json files)
    if let Err(e) = load_skills_from_directory(lib, base, &mut result) {
        result.errors.push(format!("Error loading from root: {}", e));
        result.failed += 1;
    }

    // Load from subdirectories
    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                if let Err(e) = load_skills_from_directory(lib, &path, &mut result) {
                    result.errors.push(format!(
                        "Error loading from {}: {}",
                        path.display(),
                        e
                    ));
                    result.failed += 1;
                }
            }
        }
    }

    Ok(result)
}

/// Load all JSON skill files from a specific directory.
fn load_skills_from_directory(
    lib: &mut SkillLibrary,
    dir: &Path,
    result: &mut LoadResult,
) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    match load_skill_from_file(&path) {
                        Ok(skill) => {
                            match lib.add_skill(skill) {
                                Ok(_) => {
                                    result.loaded += 1;
                                }
                                Err(e) => {
                                    result.failed += 1;
                                    result.errors.push(format!(
                                        "Failed to add skill from {}: {}",
                                        path.display(),
                                        e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            result.failed += 1;
                            result.errors.push(format!(
                                "Failed to load skill from {}: {}",
                                path.display(),
                                e
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Load a single skill from a JSON file.
fn load_skill_from_file(path: &Path) -> Result<SkillDefinition, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

    // Try to parse as a single skill
    match serde_json::from_str::<SkillDefinition>(&content) {
        Ok(mut skill) => {
            // Ensure ID is set (generate if missing or invalid)
            if skill.id == Uuid::nil() {
                skill.id = Uuid::new_v4();
            }
            Ok(skill)
        }
        Err(_) => {
            // Try to parse as a skill array (multiple skills in one file)
            match serde_json::from_str::<Vec<SkillDefinition>>(&content) {
                Ok(skills) => {
                    if skills.is_empty() {
                        return Err("Skill array is empty".to_string());
                    }
                    // Return first skill (could be extended to load all)
                    let mut first = skills[0].clone();
                    if first.id == Uuid::nil() {
                        first.id = Uuid::new_v4();
                    }
                    Ok(first)
                }
                Err(e) => Err(format!(
                    "Failed to parse JSON from {}: {}",
                    path.display(),
                    e
                )),
            }
        }
    }
}

/// Result of loading skills from folder.
#[derive(Debug, Clone)]
pub struct LoadResult {
    pub loaded: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

impl LoadResult {
    pub fn is_success(&self) -> bool {
        self.failed == 0 && self.loaded > 0
    }

    pub fn summary(&self) -> String {
        format!(
            "Loaded {} skills, {} failed. Errors: {}",
            self.loaded,
            self.failed,
            self.errors.len()
        )
    }
}

/// Find the skills directory relative to the project root.
pub fn find_skills_directory() -> Option<PathBuf> {
    // Try common locations
    let mut candidates: Vec<PathBuf> = vec![
        PathBuf::from("skills"),
        PathBuf::from("./skills"),
        PathBuf::from("../skills"),
    ];

    // Try to find from current executable location.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("skills"));
        }
    }

    for candidate in candidates {
        if candidate.exists() && candidate.is_dir() {
            return Some(candidate);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    /// Global lock to avoid tests changing the process CWD concurrently.
    static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct CwdGuard {
        original: PathBuf,
    }

    impl CwdGuard {
        fn new() -> Self {
            Self {
                original: std::env::current_dir().expect("current_dir should succeed in tests"),
            }
        }
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    #[test]
    fn test_find_skills_directory_finds_skills_in_cwd() {
        let _guard = CWD_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let _cwd_guard = CwdGuard::new();

        let tmp_root = std::env::temp_dir().join(format!("phoenix-skill-system-test-{}", Uuid::new_v4()));
        let skills_dir = tmp_root.join("skills");
        fs::create_dir_all(&skills_dir).expect("create_dir_all skills_dir");

        std::env::set_current_dir(&tmp_root).expect("set_current_dir tmp_root");

        let found = find_skills_directory().expect("expected to find skills dir");
        assert!(found.ends_with("skills"), "expected path ending in skills, got: {}", found.display());

        // Cleanup: best-effort.
        let _ = fs::remove_dir_all(&tmp_root);
    }
}
