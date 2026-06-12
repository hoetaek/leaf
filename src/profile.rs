use anyhow::{Context, Result, bail};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

const GLOBAL_PROFILE_FILE: &str = "profile.md";
const GLOBAL_PROFILE_TEMPLATE: &str = "# Global Profile\n\n이 기기에서 모든 repo의 LEAF 작업에 적용할 내용만 적는다. 한 repo에만\n해당하는 내용은 그 repo의 .leaf/PROFILE.md에 둔다. 충돌하면 repo-local\nPROFILE이 이긴다.\n\n## User Language\n\n- 미정\n\n## Settled\n\n## Provisional\n";

/// One profile file: where it should live and what it contains, if anything.
#[derive(Debug, Clone)]
pub(crate) struct ProfileSource {
    pub(crate) path: PathBuf,
    pub(crate) content: Option<String>,
}

pub(crate) fn effective_profile(cwd: impl AsRef<Path>) -> Result<String> {
    let global = match global_config_dir() {
        Some(dir) => Some(load_source(global_profile_path(&dir))?),
        None => None,
    };
    let local = match crate::git::repo_paths(cwd) {
        Ok(paths) => Some(load_source(paths.root.join(".leaf").join("PROFILE.md"))?),
        Err(_) => None,
    };
    Ok(render_effective(global.as_ref(), local.as_ref()))
}

fn load_source(path: PathBuf) -> Result<ProfileSource> {
    let content = match fs::read_to_string(&path) {
        Ok(content) => Some(content),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => {
            return Err(err).with_context(|| format!("failed to read {}", path.display()));
        }
    };
    Ok(ProfileSource { path, content })
}

pub(crate) fn global_config_dir() -> Option<PathBuf> {
    config_dir_from(
        std::env::var_os("LEAF_CONFIG_DIR"),
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

fn config_dir_from(
    leaf_config_dir: Option<OsString>,
    xdg_config_home: Option<OsString>,
    home: Option<OsString>,
) -> Option<PathBuf> {
    let non_empty = |value: Option<OsString>| value.filter(|value| !value.is_empty());
    if let Some(dir) = non_empty(leaf_config_dir) {
        return Some(PathBuf::from(dir));
    }
    if let Some(xdg) = non_empty(xdg_config_home) {
        return Some(PathBuf::from(xdg).join("leaf"));
    }
    non_empty(home).map(|home| PathBuf::from(home).join(".config").join("leaf"))
}

pub(crate) fn global_profile_path(config_dir: &Path) -> PathBuf {
    config_dir.join(GLOBAL_PROFILE_FILE)
}

pub(crate) fn ensure_global_profile(config_dir: &Path) -> Result<bool> {
    let path = global_profile_path(config_dir);
    match fs::symlink_metadata(&path) {
        Ok(_) => {
            let metadata = fs::metadata(&path)
                .with_context(|| format!("failed to inspect {}", path.display()))?;
            if metadata.is_dir() {
                bail!("path exists but is a directory: {}", path.display());
            }
            Ok(false)
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir_all(config_dir)
                .with_context(|| format!("failed to create directory {}", config_dir.display()))?;
            fs::write(&path, GLOBAL_PROFILE_TEMPLATE)
                .with_context(|| format!("failed to create profile {}", path.display()))?;
            Ok(true)
        }
        Err(err) => Err(err).with_context(|| format!("failed to inspect {}", path.display())),
    }
}

fn render_effective(global: Option<&ProfileSource>, local: Option<&ProfileSource>) -> String {
    let mut text = String::from(
        "# Effective Profile\n\n충돌 시 local(.leaf/PROFILE.md)이 global보다 우선한다. PROFILE은\nleaf-soul을 부정하지 않는다.\n",
    );
    push_layer(&mut text, "global", global, "(unavailable)");
    push_layer(&mut text, "local", local, "(not in a git repository)");
    text
}

fn push_layer(text: &mut String, name: &str, source: Option<&ProfileSource>, absent: &str) {
    text.push('\n');
    match source {
        Some(ProfileSource {
            path,
            content: Some(content),
        }) => {
            text.push_str(&format!("<!-- {name}: {} -->\n", path.display()));
            text.push_str(content.trim_end());
            text.push('\n');
        }
        Some(ProfileSource {
            path,
            content: None,
        }) => {
            text.push_str(&format!(
                "<!-- {name}: {} (missing; run `leaf init`) -->\n",
                path.display()
            ));
        }
        None => {
            text.push_str(&format!("<!-- {name}: {absent} -->\n"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(value: &str) -> Option<OsString> {
        Some(OsString::from(value))
    }

    #[test]
    fn config_dir_prefers_leaf_config_dir() {
        let dir = config_dir_from(os("/custom"), os("/xdg"), os("/home/u"));
        assert_eq!(dir, Some(PathBuf::from("/custom")));
    }

    #[test]
    fn config_dir_falls_back_to_xdg_config_home() {
        let dir = config_dir_from(None, os("/xdg"), os("/home/u"));
        assert_eq!(dir, Some(PathBuf::from("/xdg/leaf")));
    }

    #[test]
    fn config_dir_falls_back_to_home_dot_config() {
        let dir = config_dir_from(None, None, os("/home/u"));
        assert_eq!(dir, Some(PathBuf::from("/home/u/.config/leaf")));
    }

    #[test]
    fn config_dir_treats_empty_values_as_unset() {
        let dir = config_dir_from(os(""), os(""), os("/home/u"));
        assert_eq!(dir, Some(PathBuf::from("/home/u/.config/leaf")));
        assert_eq!(config_dir_from(os(""), os(""), None), None);
    }

    #[test]
    fn ensure_global_profile_creates_template_and_is_idempotent() {
        let dir = assert_fs::TempDir::new().unwrap();
        let config_dir = dir.path().join("nested/leaf");

        let created = ensure_global_profile(&config_dir).unwrap();
        assert!(created, "first call must create the file");
        let path = config_dir.join("profile.md");
        let template = std::fs::read_to_string(&path).unwrap();
        assert!(
            template.contains("## User Language"),
            "template should prompt for user language: {template}"
        );

        std::fs::write(&path, "# Mine\n").unwrap();
        let created = ensure_global_profile(&config_dir).unwrap();
        assert!(!created, "second call must not report a change");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "# Mine\n",
            "existing content must be preserved"
        );
    }

    #[test]
    fn ensure_global_profile_rejects_directory_at_profile_path() {
        let dir = assert_fs::TempDir::new().unwrap();
        let config_dir = dir.path().to_path_buf();
        std::fs::create_dir_all(config_dir.join("profile.md")).unwrap();

        let err = ensure_global_profile(&config_dir).unwrap_err();
        assert!(
            err.to_string().contains("directory"),
            "error should mention the directory conflict: {err}"
        );
    }

    fn source(path: &str, content: Option<&str>) -> ProfileSource {
        ProfileSource {
            path: PathBuf::from(path),
            content: content.map(str::to_string),
        }
    }

    #[test]
    fn render_layers_global_then_local_with_source_markers() {
        let global = source(
            "/cfg/leaf/profile.md",
            Some("## User Language\n\n- 한국어\n"),
        );
        let local = source(
            "/repo/.leaf/PROFILE.md",
            Some("## Settled\n\n- repo fact\n"),
        );
        let text = render_effective(Some(&global), Some(&local));

        let global_marker = text.find("<!-- global: /cfg/leaf/profile.md -->").unwrap();
        let local_marker = text.find("<!-- local: /repo/.leaf/PROFILE.md -->").unwrap();
        assert!(
            global_marker < local_marker,
            "global must come first:\n{text}"
        );
        assert!(text.contains("- 한국어"));
        assert!(text.contains("- repo fact"));
        assert!(
            text.contains("local") && text.contains("global"),
            "precedence note should mention both layers:\n{text}"
        );
    }

    #[test]
    fn render_marks_missing_global_file() {
        let global = source("/cfg/leaf/profile.md", None);
        let local = source("/repo/.leaf/PROFILE.md", Some("body\n"));
        let text = render_effective(Some(&global), Some(&local));

        assert!(
            text.contains("<!-- global: /cfg/leaf/profile.md (missing; run `leaf init`) -->"),
            "missing global must keep its marker:\n{text}"
        );
        assert!(text.contains("body"));
    }

    #[test]
    fn render_outside_git_repo_shows_global_only() {
        let global = source("/cfg/leaf/profile.md", Some("global body\n"));
        let text = render_effective(Some(&global), None);

        assert!(text.contains("global body"));
        assert!(
            text.contains("<!-- local: (not in a git repository) -->"),
            "local layer must explain why it is absent:\n{text}"
        );
    }

    #[test]
    fn render_with_nothing_available_still_emits_markers() {
        let text = render_effective(None, None);
        assert!(
            text.contains("<!-- global: (unavailable) -->"),
            "unknown global location must be visible:\n{text}"
        );
        assert!(text.contains("<!-- local: (not in a git repository) -->"));
    }
}
