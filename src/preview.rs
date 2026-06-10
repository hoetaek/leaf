use crate::inventory::PreviewSource;
use anyhow::Result;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
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
    TableHeader {
        cells: Vec<String>,
        widths: Vec<usize>,
    },
    TableDivider {
        widths: Vec<usize>,
        kind: TableDividerKind,
    },
    TableRow {
        headers: Vec<String>,
        cells: Vec<String>,
        widths: Vec<usize>,
    },
    Plain(String),
}

pub(crate) const TABLE_COLUMN_GAP: usize = 4;
const TABLE_HEADER_SEPARATOR_CHAR: char = '━';
const TABLE_BODY_SEPARATOR_CHAR: char = '─';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TableDividerKind {
    Header,
    Body,
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

#[cfg(test)]
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

pub(crate) fn marked_lines(lines: Vec<String>) -> Vec<PreviewLine> {
    render_markdown(&lines.join("\n"))
}

#[derive(Debug, Default)]
struct MarkdownState {
    output: Vec<PreviewLine>,
    inline_spans: Vec<PreviewSpan>,
    heading: Option<String>,
    strong: usize,
    code_block: bool,
    list_stack: Vec<ListState>,
    item: Option<ItemState>,
    table: Option<TableParseState>,
}

#[derive(Debug)]
struct ListState {
    next: Option<u64>,
}

#[derive(Debug)]
struct ItemState {
    marker: String,
    spans: Vec<PreviewSpan>,
    checkbox: Option<bool>,
}

#[derive(Debug, Default)]
struct TableParseState {
    rows: Vec<Vec<String>>,
    current_row: Option<Vec<String>>,
    current_cell: Option<String>,
}

fn render_markdown(content: &str) -> Vec<PreviewLine> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let mut state = MarkdownState::default();
    for event in Parser::new_ext(content, options) {
        state.event(event);
    }
    state.finish();
    state.output
}

impl MarkdownState {
    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag) => self.end_tag(tag),
            Event::Text(text) => self.text(text.as_ref()),
            Event::Code(code) => self.code(code.as_ref()),
            Event::SoftBreak | Event::HardBreak => self.soft_break(),
            Event::Rule => self.output.push(PreviewLine::Plain("─".repeat(24))),
            Event::Html(html) | Event::InlineHtml(html) => self.text(html.as_ref()),
            Event::TaskListMarker(checked) => {
                if let Some(item) = &mut self.item {
                    item.checkbox = Some(checked);
                }
            }
            Event::FootnoteReference(_) | Event::InlineMath(_) | Event::DisplayMath(_) => {}
        }
    }

    fn start_tag(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Heading { .. } => self.heading = Some(String::new()),
            Tag::CodeBlock(_kind) => self.code_block = true,
            Tag::List(start) => self.list_stack.push(ListState { next: start }),
            Tag::Item => self.start_item(),
            Tag::Strong => self.strong += 1,
            Tag::Table(_) => self.table = Some(TableParseState::default()),
            Tag::TableHead | Tag::TableRow => {
                if let Some(table) = &mut self.table {
                    table.current_row = Some(Vec::new());
                }
            }
            Tag::TableCell => {
                if let Some(table) = &mut self.table {
                    table.current_cell = Some(String::new());
                }
            }
            Tag::Paragraph
            | Tag::Emphasis
            | Tag::Strikethrough
            | Tag::Link { .. }
            | Tag::BlockQuote(_)
            | Tag::HtmlBlock
            | Tag::FootnoteDefinition(_)
            | Tag::Image { .. }
            | Tag::MetadataBlock(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
            | Tag::Superscript
            | Tag::Subscript => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) => {
                if let Some(heading) = self.heading.take() {
                    self.output.push(PreviewLine::Heading(heading));
                }
            }
            TagEnd::CodeBlock => self.code_block = false,
            TagEnd::List(_) => {
                self.list_stack.pop();
            }
            TagEnd::Item => self.end_item(),
            TagEnd::Paragraph => self.flush_inline_spans(),
            TagEnd::Strong => {
                self.strong = self.strong.saturating_sub(1);
            }
            TagEnd::Table => {
                if let Some(table) = self.table.take() {
                    self.output.extend(render_table_lines(&table.rows));
                }
            }
            TagEnd::TableHead | TagEnd::TableRow => {
                if let Some(table) = &mut self.table
                    && let Some(row) = table.current_row.take()
                {
                    table.rows.push(row);
                }
            }
            TagEnd::TableCell => {
                if let Some(table) = &mut self.table
                    && let Some(cell) = table.current_cell.take()
                    && let Some(row) = &mut table.current_row
                {
                    row.push(cell.trim().to_string());
                }
            }
            TagEnd::Emphasis
            | TagEnd::Strikethrough
            | TagEnd::Link
            | TagEnd::BlockQuote(_)
            | TagEnd::HtmlBlock
            | TagEnd::FootnoteDefinition
            | TagEnd::Image
            | TagEnd::MetadataBlock(_)
            | TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition
            | TagEnd::Superscript
            | TagEnd::Subscript => {}
        }
    }

    fn start_item(&mut self) {
        let marker = match self
            .list_stack
            .last_mut()
            .and_then(|list| list.next.as_mut())
        {
            Some(next) => {
                let marker = format!("{next}.");
                *next += 1;
                marker
            }
            None => "•".to_string(),
        };
        self.item = Some(ItemState {
            marker,
            spans: Vec::new(),
            checkbox: None,
        });
    }

    fn end_item(&mut self) {
        let Some(item) = self.item.take() else {
            return;
        };
        if let Some(checked) = item.checkbox {
            self.output.push(PreviewLine::Checkbox {
                checked,
                text: span_text(&item.spans),
            });
        } else {
            self.output.push(PreviewLine::ListItem {
                marker: item.marker,
                spans: item.spans,
            });
        }
    }

    fn text(&mut self, text: &str) {
        if let Some(table) = &mut self.table
            && let Some(cell) = &mut table.current_cell
        {
            cell.push_str(text);
            return;
        }
        if self.code_block {
            self.push_code_text(text);
            return;
        }
        if let Some(heading) = &mut self.heading {
            heading.push_str(text);
            return;
        }
        if self.item.is_some() {
            let span = self.text_span(text.to_string());
            if let Some(item) = &mut self.item {
                item.spans.push(span);
            }
            return;
        }
        let span = self.text_span(text.to_string());
        self.inline_spans.push(span);
    }

    fn code(&mut self, code: &str) {
        if let Some(table) = &mut self.table
            && let Some(cell) = &mut table.current_cell
        {
            cell.push_str(code);
            return;
        }
        if let Some(heading) = &mut self.heading {
            heading.push_str(code);
            return;
        }
        let span = PreviewSpan::Code(code.to_string());
        if let Some(item) = &mut self.item {
            item.spans.push(span);
        } else {
            self.inline_spans.push(span);
        }
    }

    fn soft_break(&mut self) {
        if self.code_block {
            self.output.push(PreviewLine::Code(String::new()));
        } else if let Some(table) = &mut self.table
            && let Some(cell) = &mut table.current_cell
        {
            cell.push(' ');
        } else if let Some(item) = &mut self.item {
            item.spans.push(PreviewSpan::Plain(" ".to_string()));
        } else {
            self.flush_inline_spans();
        }
    }

    fn push_code_text(&mut self, text: &str) {
        for line in text.split('\n') {
            self.output.push(PreviewLine::Code(line.to_string()));
        }
    }

    fn flush_inline_spans(&mut self) {
        if self.inline_spans.is_empty() || self.item.is_some() {
            return;
        }
        let spans = std::mem::take(&mut self.inline_spans);
        if spans.len() == 1
            && let PreviewSpan::Plain(text) = &spans[0]
        {
            self.output.push(PreviewLine::Plain(text.clone()));
            return;
        }
        self.output.push(PreviewLine::Styled(spans));
    }

    fn finish(&mut self) {
        self.flush_inline_spans();
    }

    fn text_span(&self, text: String) -> PreviewSpan {
        if self.strong > 0 {
            PreviewSpan::Bold(text)
        } else {
            PreviewSpan::Plain(text)
        }
    }
}

fn span_text(spans: &[PreviewSpan]) -> String {
    spans
        .iter()
        .map(|span| match span {
            PreviewSpan::Plain(text) | PreviewSpan::Bold(text) | PreviewSpan::Code(text) => {
                text.as_str()
            }
        })
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

#[cfg(test)]
fn checkbox_text(line: &str) -> Option<(bool, &str)> {
    let trimmed = line.trim_start();
    for (prefix, checked) in [("- [ ]", false), ("- [x]", true), ("- [X]", true)] {
        if let Some(text) = trimmed.strip_prefix(prefix) {
            return Some((checked, text.trim_start()));
        }
    }
    None
}

#[cfg(test)]
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

fn render_table_lines(rows: &[Vec<String>]) -> Vec<PreviewLine> {
    let column_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    if column_count == 0 {
        return Vec::new();
    }

    let widths = (0..column_count)
        .map(|column| {
            rows.iter()
                .filter_map(|row| row.get(column))
                .map(|cell| display_width(cell))
                .max()
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();

    let mut rendered = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if index == 0 {
            rendered.push(PreviewLine::TableHeader {
                cells: row.clone(),
                widths: widths.clone(),
            });
            rendered.push(PreviewLine::TableDivider {
                widths: widths.clone(),
                kind: TableDividerKind::Header,
            });
        } else {
            rendered.push(PreviewLine::TableRow {
                headers: rows[0].clone(),
                cells: row.clone(),
                widths: widths.clone(),
            });
            if index + 1 < rows.len() {
                rendered.push(PreviewLine::TableDivider {
                    widths: widths.clone(),
                    kind: TableDividerKind::Body,
                });
            }
        }
    }
    rendered
}

pub(crate) fn padded_table_row(row: &[String], widths: &[usize]) -> String {
    widths
        .iter()
        .enumerate()
        .map(|(index, width)| {
            let cell = row.get(index).map(String::as_str).unwrap_or("");
            format!(
                "{cell}{}",
                " ".repeat(width.saturating_sub(display_width(cell)))
            )
        })
        .collect::<Vec<_>>()
        .join(&" ".repeat(TABLE_COLUMN_GAP))
}

pub(crate) fn table_divider(widths: &[usize], kind: TableDividerKind) -> String {
    let separator = match kind {
        TableDividerKind::Header => TABLE_HEADER_SEPARATOR_CHAR,
        TableDividerKind::Body => TABLE_BODY_SEPARATOR_CHAR,
    };
    widths
        .iter()
        .map(|width| separator.to_string().repeat((*width).max(3)))
        .collect::<Vec<_>>()
        .join(&" ".repeat(TABLE_COLUMN_GAP))
}

pub(crate) fn fitted_table_widths(widths: &[usize], max_width: usize) -> Vec<usize> {
    if widths.is_empty() {
        return Vec::new();
    }

    let gap_width = TABLE_COLUMN_GAP.saturating_mul(widths.len().saturating_sub(1));
    let target = max_width.max(1).saturating_sub(gap_width).max(widths.len());
    let mut fitted = widths
        .iter()
        .map(|width| (*width).max(1))
        .collect::<Vec<_>>();
    if fitted.iter().sum::<usize>() <= target {
        return fitted;
    }

    let min_column_width = if target >= widths.len() * 3 { 3 } else { 1 };
    let floors = fitted
        .iter()
        .map(|width| (*width).min(16).max(min_column_width))
        .collect::<Vec<_>>();
    shrink_widths_to_target(&mut fitted, &floors, target);
    if fitted.iter().sum::<usize>() > target {
        let hard_floors = vec![min_column_width; fitted.len()];
        shrink_widths_to_target(&mut fitted, &hard_floors, target);
    }
    fitted
}

fn shrink_widths_to_target(widths: &mut [usize], floors: &[usize], target: usize) {
    while widths.iter().sum::<usize>() > target {
        let Some(index) = widths
            .iter()
            .enumerate()
            .filter(|(index, width)| **width > floors[*index])
            .max_by_key(|(_, width)| **width)
            .map(|(index, _)| index)
        else {
            break;
        };
        widths[index] -= 1;
    }
}

pub(crate) fn wrapped_table_row_texts(
    cells: &[String],
    widths: &[usize],
    max_width: usize,
) -> Vec<String> {
    let fitted_widths = fitted_table_widths(widths, max_width);
    let wrapped_cells = fitted_widths
        .iter()
        .enumerate()
        .map(|(index, width)| {
            wrap_text_to_width(cells.get(index).map(String::as_str).unwrap_or(""), *width)
        })
        .collect::<Vec<_>>();
    let row_height = wrapped_cells.iter().map(Vec::len).max().unwrap_or(1);

    (0..row_height)
        .map(|line_index| {
            let row = fitted_widths
                .iter()
                .enumerate()
                .map(|(index, width)| {
                    let line = wrapped_cells
                        .get(index)
                        .and_then(|lines| lines.get(line_index))
                        .cloned()
                        .unwrap_or_default();
                    pad_to_width(&line, *width)
                })
                .collect::<Vec<_>>();
            row.join(&" ".repeat(TABLE_COLUMN_GAP))
                .trim_end()
                .to_string()
        })
        .collect()
}

pub(crate) fn table_line_text(line: &PreviewLine) -> Option<String> {
    match line {
        PreviewLine::TableHeader { cells, widths }
        | PreviewLine::TableRow { cells, widths, .. } => Some(padded_table_row(cells, widths)),
        PreviewLine::TableDivider { widths, kind } => Some(table_divider(widths, *kind)),
        _ => None,
    }
}

pub(crate) fn wrapped_table_line_texts(
    line: &PreviewLine,
    max_width: usize,
) -> Option<Vec<String>> {
    match line {
        PreviewLine::TableHeader { widths, .. }
            if should_render_table_records(widths, max_width) =>
        {
            Some(Vec::new())
        }
        PreviewLine::TableHeader { cells, widths } => {
            Some(wrapped_table_row_texts(cells, widths, max_width))
        }
        PreviewLine::TableRow {
            headers,
            cells,
            widths,
        } if should_render_table_records(widths, max_width) => {
            Some(record_table_row_texts(headers, cells, max_width))
        }
        PreviewLine::TableRow { cells, widths, .. } => {
            Some(wrapped_table_row_texts(cells, widths, max_width))
        }
        PreviewLine::TableDivider { widths, kind }
            if should_render_table_records(widths, max_width) =>
        {
            match kind {
                TableDividerKind::Header => Some(Vec::new()),
                TableDividerKind::Body => Some(vec![
                    TABLE_BODY_SEPARATOR_CHAR
                        .to_string()
                        .repeat(max_width.max(1)),
                ]),
            }
        }
        PreviewLine::TableDivider { widths, kind } => Some(vec![table_divider(
            &fitted_table_widths(widths, max_width),
            *kind,
        )]),
        _ => None,
    }
}

fn should_render_table_records(widths: &[usize], max_width: usize) -> bool {
    if widths.len() < 2 {
        return false;
    }
    let original_width =
        widths.iter().sum::<usize>() + TABLE_COLUMN_GAP.saturating_mul(widths.len() - 1);
    if original_width <= max_width {
        return false;
    }
    let fitted = fitted_table_widths(widths, max_width);
    fitted.iter().skip(1).any(|width| *width < 32)
}

fn record_table_row_texts(headers: &[String], cells: &[String], max_width: usize) -> Vec<String> {
    let label_width = headers
        .iter()
        .map(|header| display_width(header))
        .max()
        .unwrap_or(0);
    let aligned = 1 + label_width + 2 + 12 <= max_width;
    let mut lines = Vec::new();

    for (header, cell) in headers.iter().zip(cells) {
        if aligned {
            let indent = 1 + label_width + 2;
            let value_width = max_width.saturating_sub(indent).max(1);
            for (index, value_line) in wrap_text_to_width(cell, value_width)
                .into_iter()
                .enumerate()
            {
                if index == 0 {
                    lines.push(format!(
                        " {header}{}  {value_line}",
                        " ".repeat(label_width.saturating_sub(display_width(header)))
                    ));
                } else {
                    lines.push(format!("{}{value_line}", " ".repeat(indent)));
                }
            }
        } else {
            lines.push(format!(" {header}"));
            for value_line in wrap_text_to_width(cell, max_width.saturating_sub(2).max(1)) {
                lines.push(format!("  {value_line}"));
            }
        }
    }

    lines
}

fn wrap_text_to_width(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        append_wrapped_word(&mut lines, &mut current, word, width);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn append_wrapped_word(lines: &mut Vec<String>, current: &mut String, word: &str, width: usize) {
    let current_width = display_width(current);
    let word_width = display_width(word);
    if current.is_empty() && word_width <= width {
        current.push_str(word);
        return;
    }
    if !current.is_empty() && current_width + 1 + word_width <= width {
        current.push(' ');
        current.push_str(word);
        return;
    }

    if !current.is_empty() {
        lines.push(std::mem::take(current));
    }

    if word_width <= width {
        current.push_str(word);
        return;
    }

    let chunks = split_long_token(word, width);
    let last_index = chunks.len().saturating_sub(1);
    for (index, chunk) in chunks.into_iter().enumerate() {
        if index == last_index {
            current.push_str(&chunk);
        } else {
            lines.push(chunk);
        }
    }
}

fn split_long_token(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;
    for ch in text.chars() {
        let char_width = display_width_char(ch);
        if current_width > 0 && current_width + char_width > width {
            chunks.push(current);
            current = String::new();
            current_width = 0;
        }
        current.push(ch);
        current_width += char_width;
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn pad_to_width(text: &str, width: usize) -> String {
    format!(
        "{text}{}",
        " ".repeat(width.saturating_sub(display_width(text)))
    )
}

pub(crate) fn display_width(text: &str) -> usize {
    text.chars().map(display_width_char).sum()
}

fn display_width_char(ch: char) -> usize {
    if ch.is_ascii() { 1 } else { 2 }
}

#[cfg(test)]
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

#[cfg(test)]
fn inline_or_plain_spans(text: &str) -> Vec<PreviewSpan> {
    inline_spans(text).unwrap_or_else(|| vec![PreviewSpan::Plain(text.to_string())])
}

#[derive(Clone, Copy)]
#[cfg(test)]
enum InlineMarker {
    Bold,
    Code,
}

#[cfg(test)]
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

#[cfg(test)]
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
            PreviewLine::TableHeader { .. }
            | PreviewLine::TableDivider { .. }
            | PreviewLine::TableRow { .. } => table_line_text(line).expect("table line text"),
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
    fn preview_markup_table_becomes_padded_table_lines() {
        let lines = marked_lines(vec![
            "| Plain check | EARS |".to_string(),
            "|---|---|".to_string(),
            "| 사용자가 이해한다 | WHEN names render, THE MODEL SHALL be clear. |".to_string(),
            "| `fallen` 이유 | WHEN an item enters `fallen`, THE MODEL SHALL record why. |"
                .to_string(),
        ]);
        let text = lines.iter().map(line_text).collect::<Vec<_>>();

        assert!(matches!(lines[0], PreviewLine::TableHeader { .. }));
        assert!(matches!(lines[1], PreviewLine::TableDivider { .. }));
        assert!(matches!(lines[2], PreviewLine::TableRow { .. }));
        assert!(matches!(lines[3], PreviewLine::TableDivider { .. }));
        assert!(text[0].contains("Plain check"));
        assert!(text[1].contains("━━━━━━━━━━━"));
        assert!(text[2].contains("사용자가 이해한다"));
        assert!(text[2].contains("    WHEN names render"));
        assert!(text[3].contains("───────────"));
        assert!(text[4].contains("fallen 이유"));
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
