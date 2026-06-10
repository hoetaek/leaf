use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodexSettings {
    pub(crate) codex_home: Option<PathBuf>,
    pub(crate) tui: CodexTuiSettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodexTuiSettings {
    pub(crate) theme: Option<String>,
    pub(crate) status_line_use_colors: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodexMarkdownRenderSettings {
    pub(crate) theme: Option<String>,
    pub(crate) use_theme_colors: bool,
}

impl CodexSettings {
    pub(crate) fn load() -> Self {
        let codex_home = codex_home();
        Self::from_codex_home(codex_home)
    }

    pub(crate) fn from_codex_home(codex_home: Option<PathBuf>) -> Self {
        let tui = codex_home
            .as_deref()
            .map(|home| Self::from_config_path(&home.join("config.toml")).tui)
            .unwrap_or_default();
        Self { codex_home, tui }
    }

    pub(crate) fn from_config_path(path: &Path) -> Self {
        let tui = std::fs::read_to_string(path)
            .ok()
            .and_then(|config| toml::from_str::<CodexConfigToml>(&config).ok())
            .and_then(|config| config.tui)
            .map(CodexTuiSettings::from)
            .unwrap_or_default();

        Self {
            codex_home: path.parent().map(Path::to_path_buf),
            tui,
        }
    }

    pub(crate) fn markdown_render_settings(&self) -> CodexMarkdownRenderSettings {
        CodexMarkdownRenderSettings {
            theme: self.tui.theme.clone(),
            use_theme_colors: self.tui.status_line_use_colors,
        }
    }
}

fn codex_home() -> Option<PathBuf> {
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".codex")))
}

#[derive(Debug, Deserialize)]
struct CodexConfigToml {
    tui: Option<CodexTuiToml>,
}

#[derive(Debug, Deserialize)]
struct CodexTuiToml {
    theme: Option<String>,
    status_line_use_colors: Option<bool>,
}

impl From<CodexTuiToml> for CodexTuiSettings {
    fn from(tui: CodexTuiToml) -> Self {
        Self {
            theme: tui.theme.and_then(non_empty_trimmed),
            status_line_use_colors: tui.status_line_use_colors.unwrap_or(true),
        }
    }
}

impl Default for CodexTuiSettings {
    fn default() -> Self {
        Self {
            theme: None,
            status_line_use_colors: true,
        }
    }
}

fn non_empty_trimmed(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_settings_read_tui_theme_from_config_path() {
        let temp = assert_fs::TempDir::new().expect("temp codex home");
        let config = temp.path().join("config.toml");
        std::fs::write(
            &config,
            "[tui]\ntheme = \" gruvbox-dark \"\nstatus_line_use_colors = false\n",
        )
        .expect("config");

        let settings = CodexSettings::from_config_path(&config);

        assert_eq!(settings.codex_home.as_deref(), Some(temp.path()));
        assert_eq!(settings.tui.theme.as_deref(), Some("gruvbox-dark"));
        assert!(!settings.tui.status_line_use_colors);
    }

    #[test]
    fn codex_settings_ignore_blank_tui_theme() {
        let temp = assert_fs::TempDir::new().expect("temp codex home");
        let config = temp.path().join("config.toml");
        std::fs::write(&config, "[tui]\ntheme = \"   \"\n").expect("config");

        let settings = CodexSettings::from_config_path(&config);

        assert_eq!(settings.tui.theme, None);
        assert!(settings.tui.status_line_use_colors);
    }

    #[test]
    fn codex_settings_default_tui_colors_to_enabled_like_codex() {
        let temp = assert_fs::TempDir::new().expect("temp codex home");
        let config = temp.path().join("config.toml");
        std::fs::write(&config, "[tui]\ntheme = \"github\"\n").expect("config");

        let settings = CodexSettings::from_config_path(&config);

        assert!(settings.tui.status_line_use_colors);
    }

    #[test]
    fn codex_settings_expose_markdown_renderer_settings_boundary() {
        let temp = assert_fs::TempDir::new().expect("temp codex home");
        let config = temp.path().join("config.toml");
        std::fs::write(
            &config,
            "[tui]\ntheme = \"dracula\"\nstatus_line_use_colors = false\n",
        )
        .expect("config");

        let settings = CodexSettings::from_config_path(&config).markdown_render_settings();

        assert_eq!(settings.theme.as_deref(), Some("dracula"));
        assert!(!settings.use_theme_colors);
    }
}
