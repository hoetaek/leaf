use anyhow::{Context, Result};
use std::fs;
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
        required_through_current_gate: false,
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

pub(crate) fn wrapped_line_count(document: &ReviewDocument, width: usize) -> usize {
    document
        .lines
        .iter()
        .map(|line| {
            if let ReviewLine::Markdown(line) = line
                && let Some(lines) = crate::preview::wrapped_table_line_texts(line, width)
            {
                lines.len()
            } else {
                wrapped_text_height(&line.visible_text(), width)
            }
        })
        .sum()
}

fn wrapped_text_height(text: &str, width: usize) -> usize {
    let width = width.max(1);
    let mut line_count = 1;
    let mut current_width = 0;
    for ch in text.chars() {
        let char_width = terminal_char_width(ch);
        if current_width > 0 && current_width + char_width > width {
            line_count += 1;
            current_width = 0;
        }
        current_width += char_width;
    }
    line_count
}

pub(crate) fn terminal_char_width(ch: char) -> usize {
    if ch.is_ascii() { 1 } else { 2 }
}

pub(crate) fn build(source: &ReviewSource) -> Result<ReviewDocument> {
    match source {
        ReviewSource::LeafWork {
            root_path,
            root_relative_path,
        } => build_leaf_work(root_path.clone(), root_relative_path.clone()),
    }
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

fn append_markdown_lines(lines: &mut Vec<ReviewLine>, content: &str) {
    lines.extend(
        crate::preview::marked_lines(content.lines().map(str::to_string).collect())
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
            append_file_section(root_relative_path, spec.file, content, sections, lines);
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
    append_markdown_lines(lines, &content);
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
        crate::preview::PreviewLine::Heading(text)
        | crate::preview::PreviewLine::Code(text)
        | crate::preview::PreviewLine::Plain(text) => text.clone(),
        crate::preview::PreviewLine::TableHeader { .. }
        | crate::preview::PreviewLine::TableDivider { .. }
        | crate::preview::PreviewLine::TableRow { .. } => {
            crate::preview::table_line_text(line).expect("table line text")
        }
        crate::preview::PreviewLine::Checkbox { checked, text } => {
            let marker = if *checked { "[x]" } else { "[ ]" };
            format!("{marker} {text}")
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
        | crate::preview::PreviewSpan::Code(text) => text,
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
        assert!(text.contains("───────────"));
        assert!(text.contains("사용자가 이해한다"));
        assert!(text.contains("    WHEN names render"));
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
    fn review_build_does_not_require_lazy_critic_file_when_missing() {
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

        assert!(!text.contains(".leaf/02-leaves/demo/03-Architect/06-critic.md"));
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
}
