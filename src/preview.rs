use crate::inventory::PreviewSource;
use anyhow::Result;
use std::fs;
use std::path::Path;

const MAX_LEAF_PREVIEW_LINES: usize = 32;
const STATUS_PREVIEW_LINES: usize = 8;
const SECONDARY_PREVIEW_LINES: usize = 4;
const DIGEST_SUMMARY_LINES: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Preview {
    pub(crate) title: String,
    pub(crate) lines: Vec<PreviewLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreviewLine {
    Heading(String),
    Checkbox {
        checked: bool,
        text: String,
    },
    ListItem {
        marker: String,
        spans: Vec<PreviewSpan>,
    },
    Code(String),
    Styled(Vec<PreviewSpan>),
    SourceBoundary {
        phase: String,
        gate: String,
        source: String,
    },
    Plain(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreviewSpan {
    Plain(String),
    Bold(String),
    Code(String),
}

pub(crate) fn build_from_source(slug: &str, source: &PreviewSource) -> Result<Preview> {
    match source {
        PreviewSource::LeafWork {
            status_path,
            intent_path,
            unknowns_path,
            criteria_path,
        } => {
            let mut lines = Vec::new();
            append_primary_file(&mut lines, status_path, STATUS_PREVIEW_LINES);
            append_gate_file(
                &mut lines,
                intent_path,
                "Learn",
                "① Intent",
                "01-Learn/01-intent.md",
                SECONDARY_PREVIEW_LINES,
            );
            append_gate_file(
                &mut lines,
                unknowns_path,
                "Learn",
                "② Unknowns",
                "01-Learn/02-unknowns.md",
                SECONDARY_PREVIEW_LINES,
            );
            append_gate_file(
                &mut lines,
                criteria_path,
                "Example",
                "③ Criteria",
                "02-Example/03-criteria.md",
                SECONDARY_PREVIEW_LINES,
            );
            lines.truncate(MAX_LEAF_PREVIEW_LINES);

            Ok(Preview {
                title: slug.to_string(),
                lines: fallback_if_empty(lines),
            })
        }
        PreviewSource::PressedDigest { digest_path } => Ok(build_pressed_digest(slug, digest_path)),
    }
}

pub(crate) fn markup_line(line: &str, in_code_block: &mut bool) -> PreviewLine {
    if line.trim_start().starts_with("```") {
        *in_code_block = !*in_code_block;
        return PreviewLine::Code(line.to_string());
    }

    if *in_code_block {
        return PreviewLine::Code(line.to_string());
    }

    if let Some(heading) = heading_text(line) {
        return PreviewLine::Heading(heading.to_string());
    }

    if let Some((checked, text)) = checkbox_text(line) {
        return PreviewLine::Checkbox {
            checked,
            text: text.to_string(),
        };
    }

    if let Some((marker, text)) = list_item_text(line) {
        return PreviewLine::ListItem {
            marker: marker.to_string(),
            spans: inline_or_plain_spans(text),
        };
    }

    match inline_spans(line) {
        Some(spans) => PreviewLine::Styled(spans),
        None => PreviewLine::Plain(line.to_string()),
    }
}

fn build_pressed_digest(slug: &str, digest_path: &Path) -> Preview {
    let content = match fs::read_to_string(digest_path) {
        Ok(content) => content,
        Err(err) => {
            return Preview {
                title: slug.to_string(),
                lines: vec![PreviewLine::Plain(format!(
                    "Unable to read preview source {}: {err}",
                    digest_path.display()
                ))],
            };
        }
    };

    let title = first_heading(&content)
        .map(str::to_string)
        .unwrap_or_else(|| slug.to_string());
    let lines = fallback_if_empty(marked_lines(digest_lines(&content)));

    Preview { title, lines }
}

fn append_primary_file(lines: &mut Vec<PreviewLine>, path: &Path, limit: usize) {
    match fs::read_to_string(path) {
        Ok(content) => lines.extend(marked_lines(useful_lines(&content, limit))),
        Err(err) => lines.push(PreviewLine::Plain(format!(
            "Unable to read preview source {}: {err}",
            path.display()
        ))),
    }
}

fn append_gate_file(
    lines: &mut Vec<PreviewLine>,
    path: &Path,
    phase: &str,
    gate: &str,
    source: &str,
    limit: usize,
) {
    if let Ok(content) = fs::read_to_string(path) {
        if !lines.is_empty() {
            lines.push(PreviewLine::Plain(String::new()));
            lines.push(PreviewLine::Plain(String::new()));
        }
        lines.push(PreviewLine::SourceBoundary {
            phase: phase.to_string(),
            gate: gate.to_string(),
            source: source.to_string(),
        });
        lines.extend(marked_lines(useful_lines(&content, limit)));
    }
}

fn marked_lines(lines: Vec<String>) -> Vec<PreviewLine> {
    let mut in_code_block = false;
    lines
        .iter()
        .map(|line| markup_line(line, &mut in_code_block))
        .collect()
}

fn useful_lines(content: &str, limit: usize) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(limit)
        .map(str::to_string)
        .collect()
}

fn digest_lines(content: &str) -> Vec<String> {
    let lines: Vec<_> = content.lines().collect();
    let title_index = lines.iter().position(|line| heading_text(line).is_some());
    let summary_index = lines.iter().position(|line| {
        heading_text(line)
            .map(|heading| heading.to_lowercase().contains("summary"))
            .unwrap_or(false)
    });

    let mut selected = Vec::new();
    if let Some(index) = title_index {
        selected.push(lines[index].trim().to_string());
    }

    if let Some(index) = summary_index {
        selected.push(lines[index].trim().to_string());
        selected.extend(first_block_after(&lines, index + 1, DIGEST_SUMMARY_LINES));
    } else {
        let start = title_index.map(|index| index + 1).unwrap_or(0);
        selected.extend(first_block_after(&lines, start, DIGEST_SUMMARY_LINES));
    }

    selected
}

fn first_block_after(lines: &[&str], start: usize, limit: usize) -> Vec<String> {
    let mut selected = Vec::new();
    let mut started = false;

    for line in &lines[start..] {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if started {
                break;
            }
            continue;
        }
        if started && heading_text(trimmed).is_some() {
            break;
        }
        selected.push(trimmed.to_string());
        started = true;
        if selected.len() >= limit {
            break;
        }
    }

    selected
}

fn fallback_if_empty(lines: Vec<PreviewLine>) -> Vec<PreviewLine> {
    if lines.is_empty() {
        vec![PreviewLine::Plain("No preview available.".to_string())]
    } else {
        lines
    }
}

fn first_heading(content: &str) -> Option<&str> {
    content.lines().find_map(heading_text)
}

fn heading_text(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let marker_count = trimmed.chars().take_while(|&ch| ch == '#').count();
    if marker_count == 0 || marker_count > 6 {
        return None;
    }

    let after_markers = &trimmed[marker_count..];
    if !after_markers.starts_with(char::is_whitespace) {
        return None;
    }

    let heading = after_markers.trim();
    if heading.is_empty() {
        None
    } else {
        Some(heading)
    }
}

fn checkbox_text(line: &str) -> Option<(bool, &str)> {
    let trimmed = line.trim_start();
    for (prefix, checked) in [("- [ ]", false), ("- [x]", true), ("- [X]", true)] {
        if let Some(text) = trimmed.strip_prefix(prefix) {
            return Some((checked, text.trim_start()));
        }
    }
    None
}

fn list_item_text(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();
    for marker in ["- ", "* ", "+ "] {
        if let Some(text) = trimmed.strip_prefix(marker) {
            return Some(("•", text));
        }
    }

    let dot_index = trimmed.find(". ")?;
    if dot_index == 0 || !trimmed[..dot_index].chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    Some((&trimmed[..=dot_index], &trimmed[dot_index + 2..]))
}

fn inline_spans(line: &str) -> Option<Vec<PreviewSpan>> {
    let mut spans = Vec::new();
    let mut index = 0;
    let mut found_markup = false;

    while index < line.len() {
        let rest = &line[index..];
        let next_bold = rest
            .find("**")
            .map(|position| (position, InlineMarker::Bold));
        let next_code = rest
            .find('`')
            .map(|position| (position, InlineMarker::Code));
        let Some((position, marker)) = earliest_marker(next_bold, next_code) else {
            push_plain(&mut spans, rest);
            break;
        };

        if position > 0 {
            push_plain(&mut spans, &rest[..position]);
        }

        let marker_start = index + position;
        match marker {
            InlineMarker::Bold => {
                let content_start = marker_start + 2;
                let Some(closing) = line[content_start..].find("**") else {
                    push_plain(&mut spans, &line[marker_start..]);
                    break;
                };
                let content_end = content_start + closing;
                spans.push(PreviewSpan::Bold(
                    line[content_start..content_end].to_string(),
                ));
                index = content_end + 2;
                found_markup = true;
            }
            InlineMarker::Code => {
                let content_start = marker_start + 1;
                let Some(closing) = line[content_start..].find('`') else {
                    push_plain(&mut spans, &line[marker_start..]);
                    break;
                };
                let content_end = content_start + closing;
                spans.push(PreviewSpan::Code(
                    line[content_start..content_end].to_string(),
                ));
                index = content_end + 1;
                found_markup = true;
            }
        }
    }

    if found_markup { Some(spans) } else { None }
}

fn inline_or_plain_spans(text: &str) -> Vec<PreviewSpan> {
    inline_spans(text).unwrap_or_else(|| vec![PreviewSpan::Plain(text.to_string())])
}

#[derive(Clone, Copy)]
enum InlineMarker {
    Bold,
    Code,
}

fn earliest_marker(
    left: Option<(usize, InlineMarker)>,
    right: Option<(usize, InlineMarker)>,
) -> Option<(usize, InlineMarker)> {
    match (left, right) {
        (Some(left), Some(right)) if right.0 < left.0 => Some(right),
        (Some(left), _) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn push_plain(spans: &mut Vec<PreviewSpan>, text: &str) {
    if text.is_empty() {
        return;
    }

    match spans.last_mut() {
        Some(PreviewSpan::Plain(existing)) => existing.push_str(text),
        _ => spans.push(PreviewSpan::Plain(text.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{self, Bucket};
    use assert_fs::prelude::*;

    fn line_text(line: &PreviewLine) -> String {
        match line {
            PreviewLine::Heading(text) | PreviewLine::Code(text) | PreviewLine::Plain(text) => {
                text.clone()
            }
            PreviewLine::Checkbox { text, .. } => text.clone(),
            PreviewLine::ListItem { marker, spans } => {
                format!("{marker} {}", span_text(spans))
            }
            PreviewLine::Styled(spans) => spans.iter().map(preview_span_text).collect(),
            PreviewLine::SourceBoundary {
                phase,
                gate,
                source,
            } => {
                format!("{phase} / {gate} {source}")
            }
        }
    }

    fn span_text(spans: &[PreviewSpan]) -> String {
        spans.iter().map(preview_span_text).collect()
    }

    fn preview_span_text(span: &PreviewSpan) -> &str {
        match span {
            PreviewSpan::Plain(text) | PreviewSpan::Bold(text) | PreviewSpan::Code(text) => text,
        }
    }

    fn preview_text(preview: &Preview) -> Vec<String> {
        preview.lines.iter().map(line_text).collect()
    }

    #[test]
    fn preview_markup_headings_become_heading() {
        let mut in_code_block = false;

        let line = markup_line("### 개요", &mut in_code_block);

        match line {
            PreviewLine::Heading(text) => assert_eq!(text, "개요"),
            other => panic!("expected heading, got {other:?}"),
        }
        assert!(!in_code_block);
    }

    #[test]
    fn preview_markup_checkboxes_become_checkbox() {
        let mut in_code_block = false;

        let checked = markup_line("- [x] Done", &mut in_code_block);
        let unchecked = markup_line("- [ ] 다음 작업", &mut in_code_block);

        match checked {
            PreviewLine::Checkbox { checked, text } => {
                assert!(checked);
                assert_eq!(text, "Done");
            }
            other => panic!("expected checked checkbox, got {other:?}"),
        }
        match unchecked {
            PreviewLine::Checkbox { checked, text } => {
                assert!(!checked);
                assert_eq!(text, "다음 작업");
            }
            other => panic!("expected unchecked checkbox, got {other:?}"),
        }
    }

    #[test]
    fn preview_markup_bullet_list_becomes_rendered_list_item() {
        let mut in_code_block = false;

        let line = markup_line("- first item", &mut in_code_block);

        match line {
            PreviewLine::ListItem { marker, spans } => {
                assert_eq!(marker, "•");
                assert_eq!(span_text(&spans), "first item");
            }
            other => panic!("expected list item, got {other:?}"),
        }
    }

    #[test]
    fn preview_markup_numbered_list_becomes_rendered_list_item() {
        let mut in_code_block = false;

        let line = markup_line("12. numbered item", &mut in_code_block);

        match line {
            PreviewLine::ListItem { marker, spans } => {
                assert_eq!(marker, "12.");
                assert_eq!(span_text(&spans), "numbered item");
            }
            other => panic!("expected numbered list item, got {other:?}"),
        }
    }

    #[test]
    fn preview_markup_list_item_preserves_inline_bold_and_code() {
        let mut in_code_block = false;

        let line = markup_line("- **Driver:** use `leaf`", &mut in_code_block);

        match line {
            PreviewLine::ListItem { marker, spans } => {
                assert_eq!(marker, "•");
                assert!(matches!(&spans[0], PreviewSpan::Bold(text) if text == "Driver:"));
                assert!(matches!(&spans[1], PreviewSpan::Plain(text) if text == " use "));
                assert!(matches!(&spans[2], PreviewSpan::Code(text) if text == "leaf"));
                assert_eq!(spans.len(), 3);
            }
            other => panic!("expected styled list item, got {other:?}"),
        }
    }

    #[test]
    fn preview_markup_fenced_code_blocks_toggle_and_emit_code_lines() {
        let mut in_code_block = false;

        let opening = markup_line("```rust", &mut in_code_block);
        assert!(matches!(opening, PreviewLine::Code(ref text) if text == "```rust"));
        assert!(in_code_block);

        let code = markup_line("let value = 1;", &mut in_code_block);
        assert!(matches!(code, PreviewLine::Code(ref text) if text == "let value = 1;"));
        assert!(in_code_block);

        let closing = markup_line("```", &mut in_code_block);
        assert!(matches!(closing, PreviewLine::Code(ref text) if text == "```"));
        assert!(!in_code_block);

        let plain = markup_line("after", &mut in_code_block);
        assert!(matches!(plain, PreviewLine::Plain(ref text) if text == "after"));
    }

    #[test]
    fn preview_markup_inline_bold_and_code_become_styled_spans() {
        let mut in_code_block = false;

        let line = markup_line("Use **bold** and `code` now", &mut in_code_block);

        match line {
            PreviewLine::Styled(spans) => {
                assert!(matches!(&spans[0], PreviewSpan::Plain(text) if text == "Use "));
                assert!(matches!(&spans[1], PreviewSpan::Bold(text) if text == "bold"));
                assert!(matches!(&spans[2], PreviewSpan::Plain(text) if text == " and "));
                assert!(matches!(&spans[3], PreviewSpan::Code(text) if text == "code"));
                assert!(matches!(&spans[4], PreviewSpan::Plain(text) if text == " now"));
                assert_eq!(spans.len(), 5);
            }
            other => panic!("expected styled spans, got {other:?}"),
        }
    }

    #[test]
    fn preview_markup_malformed_inline_markdown_stays_visible() {
        let mut in_code_block = false;

        let input = "Keep **unterminated and `dangling";
        let line = markup_line(input, &mut in_code_block);
        let rendered = line_text(&line);

        assert!(
            matches!(line, PreviewLine::Plain(_) | PreviewLine::Styled(_)),
            "expected visible fallback, got {line:?}"
        );
        assert!(
            rendered.contains("**unterminated"),
            "missing malformed bold marker in {rendered:?}"
        );
        assert!(
            rendered.contains("`dangling"),
            "missing malformed code marker in {rendered:?}"
        );
    }

    #[test]
    fn preview_build_leaf_work_includes_status_first_and_intent_when_files_exist() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/02-leaves/preview/00-status.md")
            .write_str(
                "# Leaf Status\n\n\
                 - state: active\n\
                 - current phase: Learn\n\
                 - current gate: G1\n\
                 - first missing gate: G2\n\
                 - next action: 다음 행동\n",
            )
            .expect("status");
        root.child(".leaf/02-leaves/preview/01-Learn/01-intent.md")
            .write_str("# Intent\n\n이 의도를 보여줘.\n")
            .expect("intent");

        let inventory = inventory::load(root.path()).expect("inventory");
        let item = inventory.buckets[1]
            .items
            .iter()
            .find(|item| item.bucket == Bucket::Leaves && item.slug == "preview")
            .expect("item");

        let preview = build_from_source(&item.slug, &item.preview).expect("preview");
        let text = preview_text(&preview);
        let status_index = text
            .iter()
            .position(|line| line.contains("state: active"))
            .expect("state status line");
        let intent_index = text
            .iter()
            .position(|line| line.contains("이 의도를 보여줘."))
            .expect("intent line");

        assert_eq!(preview.title, "preview");
        assert_eq!(text[0], "Leaf Status");
        assert!(
            status_index < intent_index,
            "status should come before intent: {text:?}"
        );
    }

    #[test]
    fn preview_build_leaf_work_separates_gate_snippets_with_phase_boundaries() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/02-leaves/preview/00-status.md")
            .write_str("# Leaf Status\n\n- current phase: Example\n- current gate: ③ Criteria\n")
            .expect("status");
        root.child(".leaf/02-leaves/preview/01-Learn/01-intent.md")
            .write_str("# Intent\n\n의도\n")
            .expect("intent");
        root.child(".leaf/02-leaves/preview/01-Learn/02-unknowns.md")
            .write_str("# Unknowns\n\n맥락\n")
            .expect("unknowns");
        root.child(".leaf/02-leaves/preview/02-Example/03-criteria.md")
            .write_str("# Criteria\n\n기준\n")
            .expect("criteria");

        let inventory = inventory::load(root.path()).expect("inventory");
        let item = inventory.buckets[1]
            .items
            .iter()
            .find(|item| item.bucket == Bucket::Leaves && item.slug == "preview")
            .expect("item");

        let preview = build_from_source(&item.slug, &item.preview).expect("preview");
        let text = preview_text(&preview);
        let intent_boundary = text
            .iter()
            .position(|line| line.contains("Learn / ① Intent"))
            .expect("intent boundary");
        let criteria_boundary = text
            .iter()
            .position(|line| line.contains("Example / ③ Criteria"))
            .expect("criteria boundary");

        assert_eq!(text[intent_boundary - 1], "");
        assert_eq!(text[intent_boundary - 2], "");
        assert!(
            intent_boundary < criteria_boundary,
            "Learn gate should render before Example gate: {text:?}"
        );
    }

    #[test]
    fn preview_build_pressed_digest_uses_digest_heading_and_summary() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/04-pressed/research.md")
            .write_str(
                "# Research Memo\n\n\
                 - source: .leaf/02-leaves/research\n\n\
                 ## Citation Summary\n\n\
                 첫 요약 문장입니다.\n\
                 두 번째 요약 문장입니다.\n\n\
                 ## Intent\n\n\
                 This later section should stay out.\n",
            )
            .expect("digest");

        let inventory = inventory::load(root.path()).expect("inventory");
        let item = inventory.buckets[3]
            .items
            .iter()
            .find(|item| item.bucket == Bucket::Pressed && item.slug == "research")
            .expect("pressed item");

        let preview = build_from_source(&item.slug, &item.preview).expect("preview");
        let text = preview_text(&preview);

        assert_eq!(preview.title, "Research Memo");
        assert_eq!(text[0], "Research Memo");
        assert!(text.iter().any(|line| line == "Citation Summary"));
        assert!(text.iter().any(|line| line == "첫 요약 문장입니다."));
        assert!(text.iter().any(|line| line == "두 번째 요약 문장입니다."));
        assert!(
            !text.iter().any(|line| line.contains("source:")),
            "metadata should not be part of digest preview: {text:?}"
        );
        assert!(
            !text.iter().any(|line| line.contains("later section")),
            "only first summary block should be included: {text:?}"
        );
    }
}
