use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
struct SourceSpec {
    file: &'static str,
    folders: &'static [&'static str],
    phase: &'static str,
    gate: &'static str,
    required_through_current_gate: bool,
}

const CANONICAL_SOURCES: [SourceSpec; 11] = [
    SourceSpec {
        file: "00-status.md",
        folders: &[],
        phase: "Status",
        gate: "Status",
        required_through_current_gate: true,
    },
    SourceSpec {
        file: "01-Learn/01-intent.md",
        folders: &[],
        phase: "Learn",
        gate: "① Intent",
        required_through_current_gate: true,
    },
    SourceSpec {
        file: "01-Learn/02-unknowns.md",
        folders: &[],
        phase: "Learn",
        gate: "② Unknowns",
        required_through_current_gate: true,
    },
    SourceSpec {
        file: "02-Example/03-criteria.md",
        folders: &[],
        phase: "Example",
        gate: "③ Criteria",
        required_through_current_gate: true,
    },
    SourceSpec {
        file: "02-Example/04-wireframe.md",
        folders: &["02-Example/04-wireframe"],
        phase: "Example",
        gate: "④ Wireframe",
        required_through_current_gate: true,
    },
    SourceSpec {
        file: "03-Architect/05-design.md",
        folders: &[],
        phase: "Architect",
        gate: "⑤ Design",
        required_through_current_gate: true,
    },
    SourceSpec {
        file: "03-Architect/06-critic.md",
        folders: &[],
        phase: "Architect",
        gate: "⑥ Critic",
        required_through_current_gate: true,
    },
    SourceSpec {
        file: "03-Architect/07-tasks.md",
        folders: &[],
        phase: "Architect",
        gate: "⑦ Tasks",
        required_through_current_gate: true,
    },
    SourceSpec {
        file: "03-Architect/08-execution.md",
        folders: &[],
        phase: "Architect",
        gate: "⑧ Execution",
        required_through_current_gate: true,
    },
    SourceSpec {
        file: "04-Feedback/09-review.md",
        folders: &["04-Feedback/09-reviews"],
        phase: "Feedback",
        gate: "⑨ Review",
        required_through_current_gate: true,
    },
    SourceSpec {
        file: "04-Feedback/10-retrospect.md",
        folders: &["04-Feedback/10-retrospective"],
        phase: "Feedback",
        gate: "⑩ Retrospect",
        required_through_current_gate: true,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReviewSource {
    LeafWork {
        root_path: PathBuf,
        root_relative_path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewDocument {
    pub(crate) title: String,
    pub(crate) root_relative_path: String,
    pub(crate) sections: Vec<ReviewSection>,
    pub(crate) lines: Vec<ReviewLine>,
    pub(crate) source_count: usize,
}

/// One reference file under a leaf's `01-Learn/02-references/` folder.
///
/// References are evidence collected during Learn; they are deliberately kept
/// out of the canonical review body (the 11 gate sources) so the conclusions
/// are not buried. They are surfaced separately: the TUI reads `path`, the
/// non-TTY text output lists `relative_path`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceFile {
    pub(crate) relative_path: String,
    pub(crate) path: PathBuf,
}

/// Folder, relative to a leaf root, that holds Learn reference material.
pub(crate) const REFERENCES_RELATIVE_DIR: &str = "01-Learn/02-references";

impl ReviewDocument {
    #[allow(dead_code)]
    pub(crate) fn visible_text(&self) -> String {
        self.lines
            .iter()
            .map(ReviewLine::visible_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewSection {
    pub(crate) relative_path: String,
    pub(crate) start_line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReviewLine {
    Separator {
        relative_path: String,
        phase: String,
        gate: String,
    },
    MissingSource {
        relative_path: String,
    },
    Markdown(crate::preview::PreviewLine),
    #[allow(dead_code)]
    Message(String),
}

impl ReviewLine {
    #[allow(dead_code)]
    pub(crate) fn visible_text(&self) -> String {
        match self {
            ReviewLine::Separator {
                relative_path,
                phase,
                gate,
            } => {
                format!("FILE {phase} / {gate}  {relative_path}")
            }
            ReviewLine::MissingSource { relative_path } => {
                format!("MISSING SOURCE {relative_path}")
            }
            ReviewLine::Markdown(line) => preview_visible_text(line),
            ReviewLine::Message(text) => text.clone(),
        }
    }
}

pub(crate) fn terminal_char_width(ch: char) -> usize {
    unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0)
}

pub(crate) fn build(source: &ReviewSource) -> Result<ReviewDocument> {
    match source {
        ReviewSource::LeafWork {
            root_path,
            root_relative_path,
        } => build_leaf_work(root_path.clone(), root_relative_path.clone()),
    }
}

/// Machine-readable form of a leaf's review: the canonical 11 gate sources in
/// order, each carrying its raw markdown. Consumed by `leaf review --json` and
/// the `leaf serve` web reader. Unlike `build`, this does not parse markdown
/// into TUI spans — the raw file text is handed to the client to render — so
/// the `.leaf` files stay the single source of truth.
#[derive(Debug, Serialize)]
pub(crate) struct ReviewJson {
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) root: String,
    pub(crate) sources: Vec<SourceJson>,
    pub(crate) references: Vec<ReferenceJson>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReferenceJson {
    pub(crate) relative_path: String,
    pub(crate) markdown: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SourceJson {
    pub(crate) phase: String,
    pub(crate) gate: String,
    pub(crate) relative_path: String,
    pub(crate) present: bool,
    pub(crate) markdown: String,
}

/// Build the JSON review by walking `CANONICAL_SOURCES` (the same 11-source
/// contract `build` uses) and reading each file's raw text. Missing sources are
/// emitted with `present: false` so the array is always 11 long. Folder-form
/// gates (e.g. `02-Example/04-wireframe/`) concatenate their markdown files in
/// filename order, matching `append_source`.
pub(crate) fn build_json(source: &ReviewSource) -> Result<ReviewJson> {
    let ReviewSource::LeafWork {
        root_path,
        root_relative_path,
    } = source;
    let title = root_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("leaf")
        .to_string();

    let mut sources = Vec::with_capacity(CANONICAL_SOURCES.len());
    for spec in CANONICAL_SOURCES.iter() {
        let file_path = root_path.join(spec.file);
        let entry = match fs::read_to_string(&file_path) {
            Ok(content) => SourceJson {
                phase: spec.phase.to_string(),
                gate: spec.gate.to_string(),
                relative_path: format!("{root_relative_path}/{}", spec.file),
                present: true,
                markdown: content,
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                read_folder_source(root_path, root_relative_path, spec)?
            }
            Err(err) => {
                return Err(err).context(format!("failed to read {}", file_path.display()));
            }
        };
        sources.push(entry);
    }

    let references = reference_files(source)?
        .into_iter()
        .map(|reference| {
            let markdown = fs::read_to_string(&reference.path).unwrap_or_default();
            ReferenceJson {
                relative_path: reference.relative_path,
                markdown,
            }
        })
        .collect();

    Ok(ReviewJson {
        slug: title.clone(),
        title,
        root: root_relative_path.clone(),
        sources,
        references,
    })
}

/// Resolve a folder-form gate (or report it absent). Concatenates the folder's
/// markdown files in filename order.
fn read_folder_source(
    root_path: &Path,
    root_relative_path: &str,
    spec: &SourceSpec,
) -> Result<SourceJson> {
    for folder in spec.folders {
        let folder_path = root_path.join(folder);
        if !folder_path.is_dir() {
            continue;
        }
        let files = markdown_files_in(&folder_path, folder)?;
        if files.is_empty() {
            continue;
        }
        let mut markdown = String::new();
        for file in &files {
            let content = fs::read_to_string(&file.path)
                .with_context(|| format!("failed to read {}", file.path.display()))?;
            if !markdown.is_empty() {
                markdown.push_str("\n\n");
            }
            markdown.push_str(&content);
        }
        return Ok(SourceJson {
            phase: spec.phase.to_string(),
            gate: spec.gate.to_string(),
            relative_path: format!("{root_relative_path}/{folder}/"),
            present: true,
            markdown,
        });
    }
    Ok(SourceJson {
        phase: spec.phase.to_string(),
        gate: spec.gate.to_string(),
        relative_path: format!("{root_relative_path}/{}", spec.file),
        present: false,
        markdown: String::new(),
    })
}

/// Write a `ReviewJson` as pretty JSON, mirroring `graph::write_json`.
pub(crate) fn write_json<W: Write>(writer: &mut W, document: &ReviewJson) -> Result<()> {
    serde_json::to_writer_pretty(&mut *writer, document).context("serialize review json")?;
    writeln!(writer).context("write leaf review json")?;
    Ok(())
}

pub(crate) fn write_text<W: Write>(writer: &mut W, document: &ReviewDocument) -> Result<()> {
    for line in &document.lines {
        writeln!(writer, "{}", line.visible_text()).context("write leaf review text")?;
    }
    Ok(())
}

/// Append a trailing `REFERENCES` section listing each reference file's
/// relative path. Kept separate from the body so the canonical sources are
/// unchanged; emits nothing when there are no references.
pub(crate) fn write_references_text<W: Write>(
    writer: &mut W,
    references: &[ReferenceFile],
) -> Result<()> {
    if references.is_empty() {
        return Ok(());
    }
    writeln!(writer).context("write leaf review references")?;
    writeln!(writer, "REFERENCES ({})", references.len())
        .context("write leaf review references")?;
    for reference in references {
        writeln!(writer, "  {}", reference.relative_path)
            .context("write leaf review references")?;
    }
    Ok(())
}

/// Build a single-file review document for one reference file, rendered with
/// the same markdown pipeline as the review body so the reading experience
/// matches. Local links resolve relative to the file's own directory.
pub(crate) fn build_reference_read(reference: &ReferenceFile) -> Result<ReviewDocument> {
    let content = fs::read_to_string(&reference.path)
        .with_context(|| format!("failed to read {}", reference.path.display()))?;
    let cwd = reference.path.parent().unwrap_or(&reference.path);
    let mut lines = Vec::new();
    append_markdown_lines(&mut lines, &content, cwd);
    let title = reference
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("reference")
        .to_string();
    Ok(ReviewDocument {
        title,
        root_relative_path: reference.relative_path.clone(),
        sections: vec![ReviewSection {
            relative_path: reference.relative_path.clone(),
            start_line: 0,
        }],
        lines,
        source_count: 1,
    })
}

/// List the markdown reference files under a review source's
/// `01-Learn/02-references/` folder, filename-ascending. A missing folder
/// yields an empty list (sprout/leaf/fallen all carry a leaf root, so they
/// behave identically).
pub(crate) fn reference_files(source: &ReviewSource) -> Result<Vec<ReferenceFile>> {
    let ReviewSource::LeafWork {
        root_path,
        root_relative_path,
    } = source;
    let folder_path = root_path.join(REFERENCES_RELATIVE_DIR);
    if !folder_path.is_dir() {
        return Ok(Vec::new());
    }
    // Use a repo-relative prefix so listed paths match the rest of the review
    // output and can be opened from the repo root.
    let folder_relative = format!("{root_relative_path}/{REFERENCES_RELATIVE_DIR}");
    let files = markdown_files_in(&folder_path, &folder_relative)?;
    Ok(files
        .into_iter()
        .map(|file| ReferenceFile {
            relative_path: file.relative_path,
            path: file.path,
        })
        .collect())
}

fn build_leaf_work(root_path: PathBuf, root_relative_path: String) -> Result<ReviewDocument> {
    let title = root_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("leaf")
        .to_string();
    let status_path = root_path.join(CANONICAL_SOURCES[0].file);
    let current_gate = fs::read_to_string(&status_path)
        .ok()
        .and_then(|content| parse_current_gate(&content));

    let mut sections = Vec::new();
    let mut lines = Vec::new();
    for (index, spec) in CANONICAL_SOURCES.iter().enumerate() {
        let include = if index == 0 {
            true
        } else if let Some(gate) = current_gate {
            source_exists(&root_path, spec) || (index <= gate && spec.required_through_current_gate)
        } else {
            source_exists(&root_path, spec)
        };
        if !include {
            continue;
        }

        append_source(
            &root_path,
            &root_relative_path,
            spec,
            &mut sections,
            &mut lines,
        )?;
    }

    let source_count = sections.len();
    Ok(ReviewDocument {
        title,
        root_relative_path,
        sections,
        lines,
        source_count,
    })
}

fn append_markdown_lines(lines: &mut Vec<ReviewLine>, content: &str, cwd: &Path) {
    lines.extend(
        crate::preview::render_markdown_with_cwd(content, Some(cwd))
            .into_iter()
            .map(ReviewLine::Markdown),
    );
}

fn parse_current_gate(content: &str) -> Option<usize> {
    for line in content.lines() {
        if line.trim_start().starts_with("##") {
            break;
        }
        let Some((key, value)) = parse_status_field_line(line) else {
            continue;
        };
        if key == "current gate" {
            return parse_gate_index(&value);
        }
    }
    None
}

fn parse_status_field_line(line: &str) -> Option<(String, String)> {
    let rest = line.trim_start().strip_prefix("- ")?;
    let (raw_key, raw_value) = rest.split_once(':')?;
    let key = normalize_status_key(raw_key);
    if key.is_empty() {
        return None;
    }
    Some((key, raw_value.trim().to_string()))
}

fn normalize_status_key(raw_key: &str) -> String {
    raw_key
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn source_exists(root_path: &Path, spec: &SourceSpec) -> bool {
    root_path.join(spec.file).exists()
        || spec
            .folders
            .iter()
            .any(|folder| root_path.join(folder).is_dir())
}

fn append_source(
    root_path: &Path,
    root_relative_path: &str,
    spec: &SourceSpec,
    sections: &mut Vec<ReviewSection>,
    lines: &mut Vec<ReviewLine>,
) -> Result<()> {
    let file_path = root_path.join(spec.file);
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            append_file_section(
                root_path,
                root_relative_path,
                spec.file,
                content,
                sections,
                lines,
            );
            return Ok(());
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            return Err(err).context(format!("failed to read {}", file_path.display()));
        }
    }

    for folder in spec.folders {
        let folder_path = root_path.join(folder);
        if !folder_path.is_dir() {
            continue;
        }

        let markdown_files = markdown_files_in(&folder_path, folder)?;
        if markdown_files.is_empty() {
            let relative_path = format!("{root_relative_path}/{folder}/");
            append_section_separator(
                relative_path.clone(),
                spec.phase,
                spec.gate,
                sections,
                lines,
            );
            lines.push(ReviewLine::Message(format!(
                "NO MARKDOWN SOURCES {relative_path}"
            )));
            return Ok(());
        }

        for file in markdown_files {
            let content = fs::read_to_string(&file.path)
                .with_context(|| format!("failed to read {}", file.path.display()))?;
            append_file_section(
                root_path,
                root_relative_path,
                &file.relative_path,
                content,
                sections,
                lines,
            );
        }
        return Ok(());
    }

    let relative_path = format!("{root_relative_path}/{}", spec.file);
    append_section_separator(
        relative_path.clone(),
        spec.phase,
        spec.gate,
        sections,
        lines,
    );
    lines.push(ReviewLine::MissingSource { relative_path });
    Ok(())
}

fn append_file_section(
    root_path: &Path,
    root_relative_path: &str,
    source_relative_path: &str,
    content: String,
    sections: &mut Vec<ReviewSection>,
    lines: &mut Vec<ReviewLine>,
) {
    let relative_path = format!("{root_relative_path}/{source_relative_path}");
    if let Some(spec) = source_spec_for_relative_path(source_relative_path) {
        append_section_separator(relative_path, spec.phase, spec.gate, sections, lines);
    } else {
        append_section_separator(relative_path, "Source", "Source", sections, lines);
    }
    let source_path = root_path.join(source_relative_path);
    let cwd = source_path.parent().unwrap_or(root_path);
    append_markdown_lines(lines, &content, cwd);
}

fn append_section_separator(
    relative_path: String,
    phase: &str,
    gate: &str,
    sections: &mut Vec<ReviewSection>,
    lines: &mut Vec<ReviewLine>,
) {
    if !lines.is_empty() {
        lines.push(ReviewLine::Message(String::new()));
        lines.push(ReviewLine::Message(String::new()));
    }
    sections.push(ReviewSection {
        relative_path: relative_path.clone(),
        start_line: lines.len(),
    });
    lines.push(ReviewLine::Separator {
        relative_path,
        phase: phase.to_string(),
        gate: gate.to_string(),
    });
}

fn source_spec_for_relative_path(relative_path: &str) -> Option<&'static SourceSpec> {
    CANONICAL_SOURCES.iter().find(|spec| {
        relative_path == spec.file
            || spec
                .folders
                .iter()
                .any(|folder| relative_path.starts_with(&format!("{folder}/")))
    })
}

#[derive(Debug)]
struct MarkdownFile {
    relative_path: String,
    path: PathBuf,
}

fn markdown_files_in(folder_path: &Path, folder_relative_path: &str) -> Result<Vec<MarkdownFile>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(folder_path)
        .with_context(|| format!("failed to read {}", folder_path.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read {}", folder_path.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?;
        if !file_type.is_file() {
            continue;
        }
        if entry
            .path()
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("md")
        {
            continue;
        }
        let Some(file_name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        files.push(MarkdownFile {
            relative_path: format!("{folder_relative_path}/{file_name}"),
            path: entry.path(),
        });
    }
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

fn parse_gate_index(value: &str) -> Option<usize> {
    let first = value.chars().next()?;
    match first {
        '①' => Some(1),
        '②' => Some(2),
        '③' => Some(3),
        '④' => Some(4),
        '⑤' => Some(5),
        '⑥' => Some(6),
        '⑦' => Some(7),
        '⑧' => Some(8),
        '⑨' => Some(9),
        '⑩' => Some(10),
        ch if ch.is_ascii_digit() => value
            .chars()
            .take_while(|ch| ch.is_ascii_digit())
            .collect::<String>()
            .parse::<usize>()
            .ok()
            .filter(|number| (1..=10).contains(number)),
        'g' | 'G' => value
            .strip_prefix(['g', 'G'])?
            .chars()
            .take_while(|ch| ch.is_ascii_digit())
            .collect::<String>()
            .parse::<usize>()
            .ok()
            .filter(|number| (1..=10).contains(number)),
        _ => None,
    }
}

#[allow(dead_code)]
fn preview_visible_text(line: &crate::preview::PreviewLine) -> String {
    match line {
        crate::preview::PreviewLine::BlockQuote { prefix, line, .. } => {
            format!("{prefix}{}", preview_visible_text(line))
        }
        crate::preview::PreviewLine::Heading { text, .. }
        | crate::preview::PreviewLine::Code(text)
        | crate::preview::PreviewLine::Plain(text) => text.clone(),
        crate::preview::PreviewLine::CodeSpans(spans) => {
            spans.iter().map(preview_span_text_one).collect()
        }
        crate::preview::PreviewLine::TableHeader { .. }
        | crate::preview::PreviewLine::TableDivider { .. }
        | crate::preview::PreviewLine::TableRow { .. } => {
            crate::preview::table_line_text(line).expect("table line text")
        }
        crate::preview::PreviewLine::Checkbox {
            marker,
            checked,
            text,
        } => {
            let checkbox = if *checked { "[x]" } else { "[ ]" };
            format!("{marker} {checkbox} {text}")
        }
        crate::preview::PreviewLine::ListItem { marker, spans } => {
            format!("{marker} {}", preview_span_text(spans))
        }
        crate::preview::PreviewLine::Styled(spans) => {
            spans.iter().map(preview_span_text_one).collect()
        }
        crate::preview::PreviewLine::SourceBoundary {
            phase,
            gate,
            source,
        } => {
            format!("{phase} / {gate} {source}")
        }
    }
}

fn preview_span_text(spans: &[crate::preview::PreviewSpan]) -> String {
    spans.iter().map(preview_span_text_one).collect()
}

fn preview_span_text_one(span: &crate::preview::PreviewSpan) -> &str {
    match span {
        crate::preview::PreviewSpan::Plain(text)
        | crate::preview::PreviewSpan::Bold(text)
        | crate::preview::PreviewSpan::StyledText { text, .. }
        | crate::preview::PreviewSpan::Code(text)
        | crate::preview::PreviewSpan::Link { text, .. } => text,
        crate::preview::PreviewSpan::Syntax { text, .. } => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    fn source(root: &assert_fs::TempDir, slug: &str) -> ReviewSource {
        ReviewSource::LeafWork {
            root_path: root.path().join(".leaf/02-leaves").join(slug),
            root_relative_path: format!(".leaf/02-leaves/{slug}"),
        }
    }

    fn write_file(root: &assert_fs::TempDir, slug: &str, relative: &str, body: &str) {
        root.child(format!(".leaf/02-leaves/{slug}/{relative}"))
            .write_str(body)
            .expect("write review source");
    }

    fn create_dir(root: &assert_fs::TempDir, slug: &str, relative: &str) {
        root.child(format!(".leaf/02-leaves/{slug}/{relative}"))
            .create_dir_all()
            .expect("create review source dir");
    }

    fn section_paths(document: &ReviewDocument) -> Vec<&str> {
        document
            .sections
            .iter()
            .map(|section| section.relative_path.as_str())
            .collect()
    }

    fn visible_text(document: &ReviewDocument) -> String {
        document
            .lines
            .iter()
            .map(ReviewLine::visible_text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn review_build_includes_status_first_then_current_gate_sources() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(
            &root,
            slug,
            "00-status.md",
            "# Leaf 상태\n\n- current gate: ④ Wireframe\n",
        );
        write_file(&root, slug, "01-Learn/01-intent.md", "# Intent\n\n목표\n");
        write_file(
            &root,
            slug,
            "01-Learn/02-unknowns.md",
            "# Unknowns\n\n맥락\n",
        );
        write_file(
            &root,
            slug,
            "02-Example/03-criteria.md",
            "# Criteria\n\n기준\n",
        );
        write_file(
            &root,
            slug,
            "02-Example/04-wireframe.md",
            "# Wireframe\n\n화면\n",
        );

        let document = build(&source(&root, slug)).expect("review document");

        assert_eq!(
            section_paths(&document),
            vec![
                ".leaf/02-leaves/demo/00-status.md",
                ".leaf/02-leaves/demo/01-Learn/01-intent.md",
                ".leaf/02-leaves/demo/01-Learn/02-unknowns.md",
                ".leaf/02-leaves/demo/02-Example/03-criteria.md",
                ".leaf/02-leaves/demo/02-Example/04-wireframe.md",
            ]
        );
        assert!(visible_text(&document).contains("Wireframe"));
    }

    #[test]
    fn review_build_adds_two_blank_lines_before_file_boundaries_after_first() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(
            &root,
            slug,
            "00-status.md",
            "# Status\n\n- current gate: ① Intent\n",
        );
        write_file(&root, slug, "01-Learn/01-intent.md", "# Intent\n\nGoal\n");

        let document = build(&source(&root, slug)).expect("review document");

        assert!(matches!(
            document.lines.first(),
            Some(ReviewLine::Separator { relative_path, .. })
                if relative_path == ".leaf/02-leaves/demo/00-status.md"
        ));
        let intent_start = document.sections[1].start_line;
        assert!(matches!(
            &document.lines[intent_start],
            ReviewLine::Separator { relative_path, .. }
                if relative_path == ".leaf/02-leaves/demo/01-Learn/01-intent.md"
        ));
        assert_eq!(document.lines[intent_start - 1].visible_text(), "");
        assert_eq!(document.lines[intent_start - 2].visible_text(), "");
    }

    #[test]
    fn review_build_marks_missing_sources_through_current_gate() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(
            &root,
            slug,
            "00-status.md",
            "# Leaf 상태\n\n- current gate: ⑤ Design\n",
        );
        write_file(&root, slug, "01-Learn/01-intent.md", "# Intent\n");
        write_file(&root, slug, "01-Learn/02-unknowns.md", "# Unknowns\n");
        write_file(&root, slug, "02-Example/03-criteria.md", "# Criteria\n");
        write_file(&root, slug, "02-Example/04-wireframe.md", "# Wireframe\n");

        let document = build(&source(&root, slug)).expect("review document");
        let text = visible_text(&document);

        assert!(text.contains("MISSING SOURCE"));
        assert!(text.contains(".leaf/02-leaves/demo/03-Architect/05-design.md"));
        assert!(!text.contains(".leaf/02-leaves/demo/03-Architect/06-critic.md"));
    }

    #[test]
    fn review_build_includes_existing_future_gate_files() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(
            &root,
            slug,
            "00-status.md",
            "# Leaf 상태\n\n- current gate: ④ Wireframe\n",
        );
        write_file(&root, slug, "01-Learn/01-intent.md", "# Intent\n");
        write_file(&root, slug, "01-Learn/02-unknowns.md", "# Unknowns\n");
        write_file(&root, slug, "02-Example/03-criteria.md", "# Criteria\n");
        write_file(&root, slug, "02-Example/04-wireframe.md", "# Wireframe\n");
        write_file(
            &root,
            slug,
            "03-Architect/05-design.md",
            "# Design\n\n미리 작성됨\n",
        );

        let document = build(&source(&root, slug)).expect("review document");

        assert!(
            section_paths(&document).contains(&".leaf/02-leaves/demo/03-Architect/05-design.md")
        );
        assert!(visible_text(&document).contains("미리 작성됨"));
    }

    #[test]
    fn review_build_renders_markdown_tables_as_padded_lines() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(
            &root,
            slug,
            "00-status.md",
            "# Status\n\n- current gate: ③ Criteria\n",
        );
        write_file(
            &root,
            slug,
            "01-Learn/01-intent.md",
            "| Plain check | EARS |\n\
             |---|---|\n\
             | 사용자가 이해한다 | WHEN names render, THE MODEL SHALL be clear. |\n",
        );

        let document = build(&source(&root, slug)).expect("review document");
        let text = visible_text(&document);

        assert!(text.contains("Plain check"));
        assert!(text.contains("━━━━━━━━━━━"));
        assert!(text.contains("사용자가 이해한다"));
        assert!(text.contains("    WHEN names render"));
    }

    #[test]
    fn review_build_resolves_local_links_from_each_source_file_directory() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        let target = root
            .path()
            .join(".leaf/02-leaves/demo/01-Learn/notes/guide.md");
        write_file(
            &root,
            slug,
            "00-status.md",
            "# Status\n\n- current gate: ① Intent\n",
        );
        write_file(
            &root,
            slug,
            "01-Learn/01-intent.md",
            &format!("[guide](file://{})\n", target.display()),
        );

        let document = build(&source(&root, slug)).expect("review document");
        let link_text = document.lines.iter().find_map(|line| match line {
            ReviewLine::Markdown(crate::preview::PreviewLine::Styled(spans)) => {
                spans.iter().find_map(|span| match span {
                    crate::preview::PreviewSpan::Link { text, .. } => Some(text.as_str()),
                    _ => None,
                })
            }
            _ => None,
        });

        assert_eq!(link_text, Some("notes/guide.md"));
    }

    #[test]
    fn review_build_accepts_g_prefixed_current_gate_labels() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(
            &root,
            slug,
            "00-status.md",
            "# Leaf Status\n\n- current gate: G3\n",
        );
        write_file(&root, slug, "01-Learn/01-intent.md", "# Intent\n");
        write_file(&root, slug, "01-Learn/02-unknowns.md", "# Unknowns\n");

        let document = build(&source(&root, slug)).expect("review document");
        let text = visible_text(&document);

        assert!(text.contains("MISSING SOURCE"));
        assert!(text.contains(".leaf/02-leaves/demo/02-Example/03-criteria.md"));
        assert!(!text.contains(".leaf/02-leaves/demo/02-Example/04-wireframe.md"));
    }

    #[test]
    fn review_build_normalizes_current_gate_field_like_inventory_status_parser() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(
            &root,
            slug,
            "00-status.md",
            "# Leaf Status\n\n- current   gate: ⑤ Design\n\n## Later\n\n- current gate: ① Intent\n",
        );
        write_file(&root, slug, "01-Learn/01-intent.md", "# Intent\n");
        write_file(&root, slug, "01-Learn/02-unknowns.md", "# Unknowns\n");
        write_file(&root, slug, "02-Example/03-criteria.md", "# Criteria\n");
        write_file(&root, slug, "02-Example/04-wireframe.md", "# Wireframe\n");

        let document = build(&source(&root, slug)).expect("review document");
        let text = visible_text(&document);

        assert!(text.contains("MISSING SOURCE"));
        assert!(text.contains(".leaf/02-leaves/demo/03-Architect/05-design.md"));
        assert!(!text.contains(".leaf/02-leaves/demo/03-Architect/06-critic.md"));
    }

    #[test]
    fn review_build_requires_critic_file_through_tasks() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(
            &root,
            slug,
            "00-status.md",
            "# Leaf Status\n\n- current gate: ⑦ Tasks\n",
        );
        write_file(&root, slug, "01-Learn/01-intent.md", "# Intent\n");
        write_file(&root, slug, "01-Learn/02-unknowns.md", "# Unknowns\n");
        write_file(&root, slug, "02-Example/03-criteria.md", "# Criteria\n");
        write_file(&root, slug, "02-Example/04-wireframe.md", "# Wireframe\n");
        write_file(
            &root,
            slug,
            "03-Architect/05-design.md",
            "# Design\n\nCritic quick pass: APPROVE.\n",
        );

        let document = build(&source(&root, slug)).expect("review document");
        let text = visible_text(&document);

        assert!(text.contains(".leaf/02-leaves/demo/03-Architect/06-critic.md"));
        assert!(text.contains(".leaf/02-leaves/demo/03-Architect/07-tasks.md"));
        assert!(text.contains("MISSING SOURCE"));
    }

    #[test]
    fn review_build_uses_folder_form_gate_sources_when_markdown_file_is_absent() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(
            &root,
            slug,
            "00-status.md",
            "# Leaf Status\n\n- current gate: ⑩ Retrospect\n",
        );
        write_file(&root, slug, "01-Learn/01-intent.md", "# Intent\n");
        write_file(&root, slug, "01-Learn/02-unknowns.md", "# Unknowns\n");
        write_file(&root, slug, "02-Example/03-criteria.md", "# Criteria\n");
        write_file(
            &root,
            slug,
            "02-Example/04-wireframe/02-second.md",
            "# Second Wireframe\n",
        );
        write_file(
            &root,
            slug,
            "02-Example/04-wireframe/01-first.md",
            "# First Wireframe\n",
        );
        write_file(&root, slug, "03-Architect/05-design.md", "# Design\n");
        write_file(&root, slug, "03-Architect/06-critic.md", "# Critic\n");
        write_file(&root, slug, "03-Architect/07-tasks.md", "# Tasks\n");
        write_file(&root, slug, "03-Architect/08-execution.md", "# Execution\n");
        write_file(
            &root,
            slug,
            "04-Feedback/09-reviews/02-followup.md",
            "# Follow-up Review\n",
        );
        write_file(
            &root,
            slug,
            "04-Feedback/09-reviews/01-initial.md",
            "# Initial Review\n",
        );
        write_file(
            &root,
            slug,
            "04-Feedback/10-retrospective/01-retro.md",
            "# Retrospective\n",
        );

        let document = build(&source(&root, slug)).expect("review document");

        assert_eq!(
            section_paths(&document),
            vec![
                ".leaf/02-leaves/demo/00-status.md",
                ".leaf/02-leaves/demo/01-Learn/01-intent.md",
                ".leaf/02-leaves/demo/01-Learn/02-unknowns.md",
                ".leaf/02-leaves/demo/02-Example/03-criteria.md",
                ".leaf/02-leaves/demo/02-Example/04-wireframe/01-first.md",
                ".leaf/02-leaves/demo/02-Example/04-wireframe/02-second.md",
                ".leaf/02-leaves/demo/03-Architect/05-design.md",
                ".leaf/02-leaves/demo/03-Architect/06-critic.md",
                ".leaf/02-leaves/demo/03-Architect/07-tasks.md",
                ".leaf/02-leaves/demo/03-Architect/08-execution.md",
                ".leaf/02-leaves/demo/04-Feedback/09-reviews/01-initial.md",
                ".leaf/02-leaves/demo/04-Feedback/09-reviews/02-followup.md",
                ".leaf/02-leaves/demo/04-Feedback/10-retrospective/01-retro.md",
            ]
        );
        assert_eq!(document.source_count, document.sections.len());
        assert!(!visible_text(&document).contains("MISSING SOURCE"));
    }

    #[test]
    fn review_build_reports_folder_form_gate_source_without_markdown_files() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(
            &root,
            slug,
            "00-status.md",
            "# Leaf Status\n\n- current gate: ④ Wireframe\n",
        );
        write_file(&root, slug, "01-Learn/01-intent.md", "# Intent\n");
        write_file(&root, slug, "01-Learn/02-unknowns.md", "# Unknowns\n");
        write_file(&root, slug, "02-Example/03-criteria.md", "# Criteria\n");
        create_dir(&root, slug, "02-Example/04-wireframe");
        write_file(
            &root,
            slug,
            "02-Example/04-wireframe/notes.txt",
            "not markdown\n",
        );

        let document = build(&source(&root, slug)).expect("review document");
        let text = visible_text(&document);

        assert!(text.contains("NO MARKDOWN SOURCES"));
        assert!(text.contains(".leaf/02-leaves/demo/02-Example/04-wireframe/"));
        assert!(!text.contains("MISSING SOURCE"));
    }

    #[test]
    fn reference_files_lists_markdown_filename_ascending_ignoring_others() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(&root, slug, "01-Learn/02-references/terrain.md", "# T\n");
        write_file(&root, slug, "01-Learn/02-references/README.md", "# R\n");
        write_file(
            &root,
            slug,
            "01-Learn/02-references/source-map.txt",
            "skip\n",
        );
        create_dir(&root, slug, "01-Learn/02-references/nested");
        write_file(
            &root,
            slug,
            "01-Learn/02-references/nested/inner.md",
            "# skip\n",
        );

        let references = reference_files(&source(&root, slug)).expect("references");
        let relative: Vec<&str> = references
            .iter()
            .map(|reference| reference.relative_path.as_str())
            .collect();

        assert_eq!(
            relative,
            vec![
                ".leaf/02-leaves/demo/01-Learn/02-references/README.md",
                ".leaf/02-leaves/demo/01-Learn/02-references/terrain.md",
            ],
            "only top-level .md, filename ascending, repo-relative"
        );
    }

    #[test]
    fn reference_files_is_empty_when_folder_missing() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(&root, slug, "01-Learn/01-intent.md", "# Intent\n");

        let references = reference_files(&source(&root, slug)).expect("references");

        assert!(references.is_empty());
    }

    #[test]
    fn write_references_text_emits_section_or_nothing() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_file(&root, slug, "01-Learn/02-references/terrain.md", "# T\n");
        write_file(&root, slug, "01-Learn/02-references/README.md", "# R\n");

        let references = reference_files(&source(&root, slug)).expect("references");
        let mut buffer = Vec::new();
        write_references_text(&mut buffer, &references).expect("write references");
        let text = String::from_utf8(buffer).expect("utf8");

        assert!(text.contains("REFERENCES (2)"));
        assert!(text.contains("  .leaf/02-leaves/demo/01-Learn/02-references/README.md"));
        assert!(text.contains("  .leaf/02-leaves/demo/01-Learn/02-references/terrain.md"));

        let mut empty = Vec::new();
        write_references_text(&mut empty, &[]).expect("write empty references");
        assert!(empty.is_empty(), "no section when there are no references");
    }
}
