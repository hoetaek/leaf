use crate::preview::{PreviewColor, PreviewStyle, PreviewTextStyle};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{
    Color as SyntectColor, FontStyle, Highlighter, Style as SyntectStyle, Theme, ThemeSet,
};
use syntect::parsing::{Scope, SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;
use two_face::theme::EmbeddedThemeName;

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME: OnceLock<Theme> = OnceLock::new();

const MAX_HIGHLIGHT_BYTES: usize = 512 * 1024;
const MAX_HIGHLIGHT_LINES: usize = 10_000;
const ANSI_ALPHA_INDEX: u8 = 0x00;
const ANSI_ALPHA_DEFAULT: u8 = 0x01;
const OPAQUE_ALPHA: u8 = 0xFF;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HighlightedSpan {
    pub(crate) text: String,
    pub(crate) style: PreviewStyle,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ThemeAccentColors {
    pub(crate) primary: Option<PreviewColor>,
    pub(crate) secondary: Option<PreviewColor>,
    pub(crate) muted: Option<PreviewColor>,
}

pub(crate) fn language_from_info_string(info: &str) -> Option<String> {
    info.split([',', ' ', '\t'])
        .next()
        .filter(|language| !language.is_empty())
        .map(str::to_string)
}

pub(crate) fn highlight_code(code: &str, language: &str) -> Option<Vec<Vec<HighlightedSpan>>> {
    if code.is_empty()
        || code.len() > MAX_HIGHLIGHT_BYTES
        || code.lines().count() > MAX_HIGHLIGHT_LINES
    {
        return None;
    }

    let syntax = find_syntax(language)?;
    let mut highlighter = HighlightLines::new(syntax, theme());
    let mut lines = Vec::new();

    for line in LinesWithEndings::from(code) {
        let ranges = highlighter.highlight_line(line, syntax_set()).ok()?;
        let mut spans = Vec::new();
        for (style, text) in ranges {
            let text = text.trim_end_matches(['\n', '\r']);
            if text.is_empty() {
                continue;
            }
            spans.push(HighlightedSpan {
                text: text.to_string(),
                style: preview_style(style),
            });
        }
        if spans.is_empty() {
            spans.push(HighlightedSpan {
                text: String::new(),
                style: PreviewStyle::default(),
            });
        }
        lines.push(spans);
    }

    Some(lines)
}

pub(crate) fn theme_accent_colors() -> ThemeAccentColors {
    theme_accent_colors_for_theme(theme())
}

fn syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(two_face::syntax::extra_newlines)
}

fn theme() -> &'static Theme {
    THEME.get_or_init(theme_from_codex_config)
}

fn theme_from_codex_config() -> Theme {
    let settings = CodexSyntaxSettings::load();
    let _theme_warning = validate_theme_name(
        settings.theme_name.as_deref(),
        settings.codex_home.as_deref(),
    );
    resolve_theme_with_settings(&settings)
}

fn default_theme() -> Theme {
    two_face::theme::extra()
        .get(EmbeddedThemeName::CatppuccinMocha)
        .clone()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodexSyntaxSettings {
    codex_home: Option<PathBuf>,
    theme_name: Option<String>,
}

impl CodexSyntaxSettings {
    fn load() -> Self {
        let settings = crate::codex_config::CodexSettings::load();
        Self {
            codex_home: settings.codex_home,
            theme_name: settings.tui.theme,
        }
    }
}

fn resolve_theme_by_name(name: &str, codex_home: Option<&Path>) -> Option<Theme> {
    let themes = two_face::theme::extra();
    if let Some(embedded) = parse_theme_name(name) {
        return Some(themes.get(embedded).clone());
    }
    codex_home.and_then(|home| load_custom_theme(name, home))
}

fn resolve_theme_with_settings(settings: &CodexSyntaxSettings) -> Theme {
    settings
        .theme_name
        .as_deref()
        .and_then(|name| resolve_theme_by_name(name, settings.codex_home.as_deref()))
        .unwrap_or_else(default_theme)
}

fn theme_accent_colors_for_theme(theme: &Theme) -> ThemeAccentColors {
    ThemeAccentColors {
        primary: foreground_color_for_scopes(
            theme,
            &[
                "keyword",
                "storage.type",
                "entity.name.function",
                "support.function",
            ],
        ),
        secondary: foreground_color_for_scopes(
            theme,
            &["string", "constant", "constant.numeric", "variable.other"],
        ),
        muted: foreground_color_for_scopes(theme, &["comment", "punctuation", "meta"]),
    }
}

fn foreground_color_for_scopes(theme: &Theme, scope_names: &[&str]) -> Option<PreviewColor> {
    let highlighter = Highlighter::new(theme);
    scope_names.iter().find_map(|scope_name| {
        let scope = Scope::new(scope_name).ok()?;
        let foreground = highlighter.style_mod_for_stack(&[scope]).foreground?;
        preview_color(foreground)
    })
}

fn validate_theme_name(name: Option<&str>, codex_home: Option<&Path>) -> Option<String> {
    let name = name?;
    let custom_theme_path_display = codex_home
        .map(|home| custom_theme_path(name, home).display().to_string())
        .unwrap_or_else(|| format!("$CODEX_HOME/themes/{name}.tmTheme"));

    if parse_theme_name(name).is_some() {
        return None;
    }
    if let Some(home) = codex_home {
        let custom_path = custom_theme_path(name, home);
        if custom_path.is_file() {
            if load_custom_theme(name, home).is_some() {
                return None;
            }
            return Some(format!(
                "Custom theme \"{name}\" at {custom_theme_path_display} could not be loaded. \
                 Falling back to the default theme."
            ));
        }
    }
    Some(format!(
        "Theme \"{name}\" not found. Using the default theme. \
         To use a custom theme, place a .tmTheme file at {custom_theme_path_display}."
    ))
}

fn custom_theme_path(name: &str, codex_home: &Path) -> PathBuf {
    codex_home.join("themes").join(format!("{name}.tmTheme"))
}

fn load_custom_theme(name: &str, codex_home: &Path) -> Option<Theme> {
    ThemeSet::get_theme(custom_theme_path(name, codex_home)).ok()
}

fn parse_theme_name(name: &str) -> Option<EmbeddedThemeName> {
    let normalized = normalized_builtin_theme_name(name);
    match normalized.as_str() {
        "ansi" => Some(EmbeddedThemeName::Ansi),
        "base16" => Some(EmbeddedThemeName::Base16),
        "base16-eighties-dark" => Some(EmbeddedThemeName::Base16EightiesDark),
        "base16-mocha-dark" => Some(EmbeddedThemeName::Base16MochaDark),
        "base16-ocean-dark" => Some(EmbeddedThemeName::Base16OceanDark),
        "base16-ocean-light" => Some(EmbeddedThemeName::Base16OceanLight),
        "base16-256" => Some(EmbeddedThemeName::Base16_256),
        "catppuccin-frappe" => Some(EmbeddedThemeName::CatppuccinFrappe),
        "catppuccin-latte" => Some(EmbeddedThemeName::CatppuccinLatte),
        "catppuccin-macchiato" => Some(EmbeddedThemeName::CatppuccinMacchiato),
        "catppuccin-mocha" => Some(EmbeddedThemeName::CatppuccinMocha),
        "coldark-cold" => Some(EmbeddedThemeName::ColdarkCold),
        "coldark-dark" => Some(EmbeddedThemeName::ColdarkDark),
        "dark-neon" => Some(EmbeddedThemeName::DarkNeon),
        "dracula" => Some(EmbeddedThemeName::Dracula),
        "github" => Some(EmbeddedThemeName::Github),
        "gruvbox-dark" => Some(EmbeddedThemeName::GruvboxDark),
        "gruvbox-light" => Some(EmbeddedThemeName::GruvboxLight),
        "inspired-github" => Some(EmbeddedThemeName::InspiredGithub),
        "1337" => Some(EmbeddedThemeName::Leet),
        "monokai-extended" => Some(EmbeddedThemeName::MonokaiExtended),
        "monokai-extended-bright" => Some(EmbeddedThemeName::MonokaiExtendedBright),
        "monokai-extended-light" => Some(EmbeddedThemeName::MonokaiExtendedLight),
        "monokai-extended-origin" => Some(EmbeddedThemeName::MonokaiExtendedOrigin),
        "nord" => Some(EmbeddedThemeName::Nord),
        "one-half-dark" => Some(EmbeddedThemeName::OneHalfDark),
        "one-half-light" => Some(EmbeddedThemeName::OneHalfLight),
        "solarized-dark" => Some(EmbeddedThemeName::SolarizedDark),
        "solarized-light" => Some(EmbeddedThemeName::SolarizedLight),
        "sublime-snazzy" => Some(EmbeddedThemeName::SublimeSnazzy),
        "two-dark" => Some(EmbeddedThemeName::TwoDark),
        "zenburn" => Some(EmbeddedThemeName::Zenburn),
        _ => None,
    }
}

fn normalized_builtin_theme_name(name: &str) -> String {
    let mut normalized = String::new();
    for ch in name.trim().chars() {
        match ch {
            ' ' | '_' => normalized.push('-'),
            _ => normalized.extend(ch.to_lowercase()),
        }
    }
    normalized
}

fn find_syntax(language: &str) -> Option<&'static SyntaxReference> {
    let syntax_set = syntax_set();
    let patched = match language {
        "csharp" | "c-sharp" => "c#",
        "golang" => "go",
        "python3" => "python",
        "shell" => "bash",
        _ => language,
    };

    syntax_set
        .find_syntax_by_token(patched)
        .or_else(|| syntax_set.find_syntax_by_name(patched))
        .or_else(|| {
            let lower = patched.to_ascii_lowercase();
            syntax_set
                .syntaxes()
                .iter()
                .find(|syntax| syntax.name.to_ascii_lowercase() == lower)
        })
        .or_else(|| syntax_set.find_syntax_by_extension(language))
}

fn preview_style(style: SyntectStyle) -> PreviewStyle {
    let mut preview = PreviewStyle {
        fg: preview_color(style.foreground),
        bg: preview_color(style.background),
        text_style: PreviewTextStyle::default(),
    };
    if style.font_style.contains(FontStyle::BOLD) {
        preview.text_style.bold = true;
    }
    if style.font_style.contains(FontStyle::ITALIC) {
        preview.text_style.italic = true;
    }
    if style.font_style.contains(FontStyle::UNDERLINE) {
        preview.text_style.underline = true;
    }
    preview
}

fn preview_color(color: SyntectColor) -> Option<PreviewColor> {
    match color.a {
        ANSI_ALPHA_INDEX => Some(PreviewColor::Ansi(ansi_palette_index(color.r))),
        ANSI_ALPHA_DEFAULT => None,
        OPAQUE_ALPHA => Some(PreviewColor::Rgb(color.r, color.g, color.b)),
        _ => Some(PreviewColor::Rgb(color.r, color.g, color.b)),
    }
}

fn ansi_palette_index(index: u8) -> u8 {
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_from_info_string_takes_first_metadata_token() {
        assert_eq!(
            language_from_info_string("rust,no_run").as_deref(),
            Some("rust")
        );
        assert_eq!(
            language_from_info_string("rust title=demo").as_deref(),
            Some("rust")
        );
        assert_eq!(
            language_from_info_string("rust\tignore").as_deref(),
            Some("rust")
        );
        assert_eq!(language_from_info_string("").as_deref(), None);
    }

    #[test]
    fn highlight_code_known_language_returns_styled_spans() {
        let lines = highlight_code("fn main() {}\n", "rust").expect("rust highlighting");

        assert_eq!(
            lines
                .iter()
                .flat_map(|line| line.iter())
                .map(|span| span.text.as_str())
                .collect::<String>(),
            "fn main() {}"
        );
        assert!(
            lines
                .iter()
                .flat_map(|line| line.iter())
                .any(|span| span.style.fg.is_some() || span.style.text_style.bold)
        );
    }

    #[test]
    fn highlight_code_unknown_language_returns_none() {
        assert!(highlight_code("hello\n", "xyzlang").is_none());
    }

    #[test]
    fn preview_style_preserves_syntect_text_modifiers() {
        let style = SyntectStyle {
            foreground: SyntectColor {
                r: 1,
                g: 2,
                b: 3,
                a: OPAQUE_ALPHA,
            },
            background: SyntectColor {
                r: 0,
                g: 0,
                b: 0,
                a: OPAQUE_ALPHA,
            },
            font_style: FontStyle::BOLD | FontStyle::ITALIC | FontStyle::UNDERLINE,
        };

        let preview = preview_style(style);

        assert_eq!(preview.fg, Some(PreviewColor::Rgb(1, 2, 3)));
        assert_eq!(preview.bg, Some(PreviewColor::Rgb(0, 0, 0)));
        assert!(preview.text_style.bold);
        assert!(preview.text_style.italic);
        assert!(preview.text_style.underline);
        assert!(!preview.text_style.strikethrough);
    }

    #[test]
    fn resolve_theme_by_name_accepts_codex_builtin_theme_names() {
        assert!(resolve_theme_by_name("gruvbox-dark", None).is_some());
        assert!(resolve_theme_by_name(" Gruvbox Dark ", None).is_some());
        assert!(resolve_theme_by_name("catppuccin_mocha", None).is_some());
        assert!(resolve_theme_by_name("not-a-real-theme", None).is_none());
    }

    #[test]
    fn validate_theme_name_matches_codex_custom_theme_rules() {
        let temp = assert_fs::TempDir::new().expect("temp codex home");
        let themes_dir = temp.path().join("themes");
        std::fs::create_dir(&themes_dir).expect("themes dir");

        assert!(validate_theme_name(Some("dracula"), None).is_none());
        let missing = validate_theme_name(Some("my-custom"), Some(temp.path()))
            .expect("missing custom theme warning");
        assert!(missing.contains("my-custom"));
        assert!(missing.contains("themes/my-custom.tmTheme"));

        std::fs::write(themes_dir.join("broken.tmTheme"), "not a plist").expect("broken theme");
        let broken = validate_theme_name(Some("broken"), Some(temp.path()))
            .expect("invalid custom theme warning");
        assert!(broken.contains("could not be loaded"));

        write_minimal_tmtheme(&themes_dir.join("valid.tmTheme"));
        assert!(validate_theme_name(Some("valid"), Some(temp.path())).is_none());
        assert!(resolve_theme_by_name("valid", Some(temp.path())).is_some());
    }

    #[test]
    fn codex_syntax_settings_resolve_configured_custom_theme() {
        let temp = assert_fs::TempDir::new().expect("temp codex home");
        let themes_dir = temp.path().join("themes");
        std::fs::create_dir(&themes_dir).expect("themes dir");
        write_minimal_tmtheme(&themes_dir.join("valid.tmTheme"));

        let settings = CodexSyntaxSettings {
            codex_home: Some(temp.path().to_path_buf()),
            theme_name: Some("valid".to_string()),
        };

        let configured = resolve_theme_with_settings(&settings);
        let fallback = default_theme();
        assert_ne!(configured.name, fallback.name);
    }

    #[test]
    fn codex_syntax_settings_fall_back_for_invalid_configured_theme() {
        let settings = CodexSyntaxSettings {
            codex_home: None,
            theme_name: Some("not-a-real-theme".to_string()),
        };

        let configured = resolve_theme_with_settings(&settings);
        let fallback = default_theme();
        assert_eq!(configured.name, fallback.name);
    }

    #[test]
    fn theme_accent_colors_extract_foreground_scopes_from_builtin_theme() {
        let theme = resolve_theme_by_name("dracula", None).expect("builtin theme");

        let accents = theme_accent_colors_for_theme(&theme);

        assert!(
            accents.primary.is_some() || accents.secondary.is_some() || accents.muted.is_some()
        );
    }

    fn write_minimal_tmtheme(path: &Path) {
        std::fs::write(
            path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>name</key><string>Leaf Test</string>
<key>settings</key><array><dict>
<key>settings</key><dict>
<key>foreground</key><string>#FFFFFF</string>
<key>background</key><string>#000000</string>
</dict></dict></array>
</dict></plist>"#,
        )
        .expect("write tmTheme");
    }
}
