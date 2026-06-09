use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const CANONICAL_SOURCES: [&str; 11] = [
    "00-status.md",
    "01-Learn/01-intent.md",
    "01-Learn/02-unknowns.md",
    "02-Example/03-criteria.md",
    "02-Example/04-wireframe.md",
    "03-Architect/05-design.md",
    "03-Architect/06-critic.md",
    "03-Architect/07-tasks.md",
    "03-Architect/08-execution.md",
    "04-Feedback/09-review.md",
    "04-Feedback/10-retrospect.md",
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
    Separator(String),
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
            ReviewLine::Separator(path) => path.clone(),
            ReviewLine::MissingSource { relative_path } => {
                format!("MISSING SOURCE {relative_path}")
            }
            ReviewLine::Markdown(line) => preview_visible_text(line),
            ReviewLine::Message(text) => text.clone(),
        }
    }
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
    let status_path = root_path.join(CANONICAL_SOURCES[0]);
    let current_gate = fs::read_to_string(&status_path)
        .ok()
        .and_then(|content| parse_current_gate(&content));

    let mut sections = Vec::new();
    let mut lines = Vec::new();
    for (index, relative) in CANONICAL_SOURCES.iter().enumerate() {
        let path = root_path.join(relative);
        let include = if index == 0 {
            true
        } else if let Some(gate) = current_gate {
            index <= gate || path.exists()
        } else {
            path.exists()
        };
        if !include {
            continue;
        }

        let relative_path = format!("{root_relative_path}/{relative}");
        sections.push(ReviewSection {
            relative_path: relative_path.clone(),
            start_line: lines.len(),
        });
        lines.push(ReviewLine::Separator(relative_path.clone()));

        match fs::read_to_string(&path) {
            Ok(content) => append_markdown_lines(&mut lines, &content),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                lines.push(ReviewLine::MissingSource { relative_path });
            }
            Err(err) => {
                return Err(err).context(format!("failed to read {}", path.display()));
            }
        }
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
    let mut in_code_block = false;
    lines.extend(
        content.lines().map(|line| {
            ReviewLine::Markdown(crate::preview::markup_line(line, &mut in_code_block))
        }),
    );
}

fn parse_current_gate(content: &str) -> Option<usize> {
    content.lines().find_map(|line| {
        let lower = line.to_lowercase();
        let (_, value) = lower.split_once("current gate:")?;
        parse_gate_index(value.trim())
    })
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
        _ => None,
    }
}

#[allow(dead_code)]
fn preview_visible_text(line: &crate::preview::PreviewLine) -> String {
    match line {
        crate::preview::PreviewLine::Heading(text)
        | crate::preview::PreviewLine::Code(text)
        | crate::preview::PreviewLine::Plain(text) => text.clone(),
        crate::preview::PreviewLine::Checkbox { checked, text } => {
            let marker = if *checked { "[x]" } else { "[ ]" };
            format!("{marker} {text}")
        }
        crate::preview::PreviewLine::ListItem { marker, text } => format!("{marker} {text}"),
        crate::preview::PreviewLine::Styled(spans) => spans
            .iter()
            .map(|span| match span {
                crate::preview::PreviewSpan::Plain(text)
                | crate::preview::PreviewSpan::Bold(text)
                | crate::preview::PreviewSpan::Code(text) => text.as_str(),
            })
            .collect(),
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
}
