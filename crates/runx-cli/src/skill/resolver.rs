use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn resolve_skill_ref(skill_ref: &Path, cwd: &Path) -> Result<PathBuf, String> {
    if skill_ref.exists() {
        return resolve_exported_skill_shim(skill_ref);
    }

    if is_bare_skill_ref(skill_ref) {
        let local_skill = cwd.join("skills").join(skill_ref);
        if local_skill.exists() {
            return resolve_exported_skill_shim(&local_skill);
        }
        return Err(format!(
            "could not resolve skill ref '{}'; tried {}",
            skill_ref.display(),
            local_skill.display()
        ));
    }

    Ok(skill_ref.to_path_buf())
}

fn is_bare_skill_ref(skill_ref: &Path) -> bool {
    skill_ref.components().count() == 1
}

fn resolve_exported_skill_shim(skill_ref: &Path) -> Result<PathBuf, String> {
    let skill_dir = if skill_ref.is_file() {
        skill_ref.parent().unwrap_or(skill_ref)
    } else {
        skill_ref
    };
    if skill_dir.join("X.yaml").exists() {
        return Ok(skill_ref.to_path_buf());
    }

    let skill_md = if skill_ref.is_file() {
        skill_ref.to_path_buf()
    } else {
        skill_dir.join("SKILL.md")
    };
    if !skill_md.exists() {
        return Ok(skill_ref.to_path_buf());
    }

    let source = fs::read_to_string(&skill_md)
        .map_err(|error| format!("failed to read {}: {error}", skill_md.display()))?;
    let Some(source_path) = exported_source_path(&source) else {
        return Ok(skill_ref.to_path_buf());
    };
    if source_path.join("X.yaml").exists() {
        return Ok(source_path);
    }
    Err(format!(
        "exported skill shim {} points at missing or invalid source {}; rerun `runx export`",
        skill_md.display(),
        source_path.display()
    ))
}

fn exported_source_path(source: &str) -> Option<PathBuf> {
    source
        .lines()
        .find(|line| line.contains("runx-export:") && line.contains(" source="))
        .and_then(|line| line.split_once(" source=").map(|(_prefix, value)| value))
        .map(|value| {
            let raw = value.trim().trim_end_matches("-->").trim();
            let raw = raw
                .strip_suffix("- generated, do not edit")
                .unwrap_or(raw)
                .trim();
            PathBuf::from(raw)
        })
        .filter(|path| !path.as_os_str().is_empty())
}
