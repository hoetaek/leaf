use crate::inventory::PreviewSource;
use anyhow::Result;
use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::fs;
use std::ops::Range;
use std::path::Path;
use url::Url;

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
    BlockQuote {
        depth: usize,
        prefix: String,
        line: Box<PreviewLine>,
    },
    Heading {
        level: u8,
        text: String,
    },
    Checkbox {
        marker: String,
        checked: bool,
        text: String,
    },
    ListItem {
        marker: String,
        spans: Vec<PreviewSpan>,
    },
    Code(String),
    CodeSpans(Vec<PreviewSpan>),
    Styled(Vec<PreviewSpan>),
    SourceBoundary {
        phase: String,
        gate: String,
        source: String,
    },
    TableHeader {
        cells: Vec<String>,
        links: Vec<PreviewTableLink>,
        widths: Vec<usize>,
        alignments: Vec<TableAlignment>,
        /// Table-wide column metrics, computed once across all rows so every
        /// line of the table fits to the same column widths and stays aligned.
        metrics: Vec<TableColumnMetric>,
    },
    TableDivider {
        widths: Vec<usize>,
        kind: TableDividerKind,
        metrics: Vec<TableColumnMetric>,
    },
    TableRow {
        headers: Vec<String>,
        cells: Vec<String>,
        links: Vec<PreviewTableLink>,
        widths: Vec<usize>,
        alignments: Vec<TableAlignment>,
        metrics: Vec<TableColumnMetric>,
    },
    Plain(String),
}

pub(crate) const TABLE_COLUMN_GAP: usize = 4;
const MIN_TABLE_COLUMN_MODE_WIDTH: usize = 72;
const TABLE_HEADER_SEPARATOR_CHAR: char = '━';
const TABLE_BODY_SEPARATOR_CHAR: char = '─';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TableDividerKind {
    Header,
    Body,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum TableAlignment {
    #[default]
    None,
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreviewTableLink {
    pub(crate) cell: usize,
    pub(crate) text: String,
    pub(crate) columns: Range<usize>,
    pub(crate) target: String,
    pub(crate) source_range: PreviewSourceRange,
    pub(crate) local: bool,
}

impl From<Alignment> for TableAlignment {
    fn from(alignment: Alignment) -> Self {
        match alignment {
            Alignment::None => Self::None,
            Alignment::Left => Self::Left,
            Alignment::Center => Self::Center,
            Alignment::Right => Self::Right,
        }
    }
}

fn heading_level_number(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreviewSpan {
    Plain(String),
    Bold(String),
    StyledText {
        text: String,
        style: PreviewTextStyle,
    },
    Code(String),
    Link {
        text: String,
        target: String,
        source_range: PreviewSourceRange,
        local: bool,
    },
    Syntax {
        text: String,
        style: PreviewStyle,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PreviewSourceRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl From<Range<usize>> for PreviewSourceRange {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PreviewStyle {
    pub(crate) fg: Option<PreviewColor>,
    pub(crate) bg: Option<PreviewColor>,
    pub(crate) text_style: PreviewTextStyle,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PreviewTextStyle {
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) underline: bool,
    pub(crate) strikethrough: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreviewColor {
    Rgb(u8, u8, u8),
    Ansi(u8),
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
        return PreviewLine::Heading {
            level: 1,
            text: heading.to_string(),
        };
    }

    if let Some((checked, text)) = checkbox_text(line) {
        return PreviewLine::Checkbox {
            marker: "•".to_string(),
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
    let lines = fallback_if_empty(marked_lines_with_cwd(
        digest_lines(&content),
        digest_path.parent(),
    ));

    Preview { title, lines }
}

fn append_primary_file(lines: &mut Vec<PreviewLine>, path: &Path, limit: usize) {
    match fs::read_to_string(path) {
        Ok(content) => lines.extend(marked_lines_with_cwd(
            useful_lines(&content, limit),
            path.parent(),
        )),
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
        lines.extend(marked_lines_with_cwd(
            useful_lines(&content, limit),
            path.parent(),
        ));
    }
}

#[cfg(test)]
pub(crate) fn marked_lines(lines: Vec<String>) -> Vec<PreviewLine> {
    render_markdown_with_cwd(&lines.join("\n"), std::env::current_dir().ok().as_deref())
}

fn marked_lines_with_cwd(lines: Vec<String>, cwd: Option<&Path>) -> Vec<PreviewLine> {
    render_markdown_with_cwd(&lines.join("\n"), cwd)
}

#[derive(Debug, Default)]
struct MarkdownState {
    source: String,
    output: Vec<PreviewLine>,
    inline_spans: Vec<PreviewSpan>,
    heading: Option<HeadingState>,
    strong: usize,
    emphasis: usize,
    strikethrough: usize,
    code_block: bool,
    code_block_language: Option<String>,
    code_block_buffer: String,
    code_block_prefix: String,
    blockquote_depth: usize,
    blockquote_frames: Vec<BlockQuoteFrame>,
    link: Option<LinkState>,
    line_ends_with_local_link_target: bool,
    pending_local_link_soft_break: bool,
    cwd: Option<std::path::PathBuf>,
    list_stack: Vec<ListState>,
    item_stack: Vec<ItemState>,
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
    emitted: bool,
}

#[derive(Debug)]
struct BlockQuoteFrame {
    prefix: String,
    continuation_prefix: String,
    first_line: bool,
}

#[derive(Debug)]
struct HeadingState {
    level: u8,
    text: String,
}

impl BlockQuoteFrame {
    fn current_prefix(&self) -> &str {
        if self.first_line {
            &self.prefix
        } else {
            &self.continuation_prefix
        }
    }
}

#[derive(Debug, Default)]
struct ItemBlockQuotePrefix {
    first: String,
    continuation: String,
}

impl ItemState {
    fn preview_line(&self) -> PreviewLine {
        if let Some(checked) = self.checkbox {
            PreviewLine::Checkbox {
                marker: self.marker.clone(),
                checked,
                text: span_text(&self.spans),
            }
        } else {
            PreviewLine::ListItem {
                marker: self.marker.clone(),
                spans: self.spans.clone(),
            }
        }
    }

    fn continuation_prefix(&self) -> String {
        " ".repeat(self.marker.chars().count() + 1)
    }
}

#[derive(Debug)]
struct LinkState {
    destination: String,
    source_range: PreviewSourceRange,
    local_target_display: Option<String>,
}

#[derive(Debug, Default)]
struct TableParseState {
    alignments: Vec<TableAlignment>,
    rows: Vec<Vec<TableCell>>,
    row_has_pipe_syntax: Vec<bool>,
    current_row: Option<Vec<TableCell>>,
    current_cell: Option<TableCell>,
    current_row_has_pipe_syntax: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TableCell {
    text: String,
    links: Vec<TableCellLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableCellLink {
    text: String,
    columns: Range<usize>,
    target: String,
    source_range: PreviewSourceRange,
    local: bool,
}

impl TableCell {
    fn push_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    fn push_span(&mut self, span: &PreviewSpan) {
        let start = display_width(&self.text);
        self.text.push_str(span.text());
        if let PreviewSpan::Link {
            text,
            target,
            source_range,
            local,
        } = span
        {
            self.links.push(TableCellLink {
                text: text.clone(),
                columns: start..start + display_width(text),
                target: target.clone(),
                source_range: *source_range,
                local: *local,
            });
        }
    }

    fn trimmed(mut self) -> Self {
        let leading = self.text.len() - self.text.trim_start().len();
        let trailing = self.text.len() - self.text.trim_end().len();
        if leading == 0 && trailing == 0 {
            return self;
        }

        let leading_width = display_width(&self.text[..leading]);
        let end_byte = self.text.len().saturating_sub(trailing);
        let trimmed_text = self.text[leading..end_byte].to_string();
        let trimmed_width = display_width(&trimmed_text);
        let links = self
            .links
            .into_iter()
            .filter_map(|mut link| {
                if link.columns.end <= leading_width
                    || link.columns.start >= leading_width + trimmed_width
                {
                    return None;
                }
                link.columns = link.columns.start.saturating_sub(leading_width)
                    ..link
                        .columns
                        .end
                        .saturating_sub(leading_width)
                        .min(trimmed_width);
                (!link.columns.is_empty()).then_some(link)
            })
            .collect();

        self.text = trimmed_text;
        self.links = links;
        self
    }
}

pub(crate) fn render_markdown_with_cwd(content: &str, cwd: Option<&Path>) -> Vec<PreviewLine> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let mut state = MarkdownState {
        source: content.to_string(),
        cwd: cwd.map(Path::to_path_buf),
        ..MarkdownState::default()
    };
    for (event, range) in Parser::new_ext(content, options).into_offset_iter() {
        state.event(event, range);
    }
    state.finish();
    state.output
}

impl MarkdownState {
    fn event(&mut self, event: Event<'_>, range: Range<usize>) {
        match event {
            Event::Start(tag) => self.start_tag(tag, range),
            Event::End(tag) => self.end_tag(tag),
            Event::Text(text) => self.text_with_range(text.as_ref(), range),
            Event::Code(code) => self.code(code.as_ref()),
            Event::SoftBreak | Event::HardBreak => self.soft_break(),
            Event::Rule => self.push_output(PreviewLine::Plain("─".repeat(24))),
            Event::Html(html) => self.html(html.as_ref()),
            Event::InlineHtml(html) => self.text(html.as_ref()),
            Event::TaskListMarker(checked) => {
                if let Some(item) = self.item_stack.last_mut() {
                    item.checkbox = Some(checked);
                }
            }
            Event::FootnoteReference(_) | Event::InlineMath(_) | Event::DisplayMath(_) => {}
        }
    }

    fn start_tag(&mut self, tag: Tag<'_>, range: Range<usize>) {
        match tag {
            Tag::Heading { level, .. } => {
                self.heading = Some(HeadingState {
                    level: heading_level_number(level),
                    text: String::new(),
                });
            }
            Tag::CodeBlock(kind) => self.start_code_block(kind),
            Tag::List(start) => self.start_list(start),
            Tag::Item => self.start_item(),
            Tag::Strong => self.strong += 1,
            Tag::Emphasis => self.emphasis += 1,
            Tag::Strikethrough => self.strikethrough += 1,
            Tag::Table(alignments) => {
                self.table = Some(TableParseState {
                    alignments: alignments.into_iter().map(TableAlignment::from).collect(),
                    ..TableParseState::default()
                })
            }
            Tag::TableHead | Tag::TableRow => {
                let has_pipe_syntax = self.has_table_row_boundary_pipe(range);
                if let Some(table) = &mut self.table {
                    table.current_row = Some(Vec::new());
                    table.current_row_has_pipe_syntax = has_pipe_syntax;
                }
            }
            Tag::TableCell => {
                if let Some(table) = &mut self.table {
                    table.current_cell = Some(TableCell::default());
                }
            }
            Tag::BlockQuote(_) => self.start_blockquote(),
            Tag::Link { dest_url, .. } => self.start_link(dest_url.as_ref(), range),
            Tag::Paragraph => self.start_paragraph(),
            Tag::HtmlBlock
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

    fn has_table_row_boundary_pipe(&self, range: Range<usize>) -> bool {
        self.source
            .get(range)
            .map(str::trim)
            .is_some_and(|row| row.starts_with('|') || row.ends_with('|'))
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) => {
                if let Some(heading) = self.heading.take() {
                    self.push_output(PreviewLine::Heading {
                        level: heading.level,
                        text: heading.text,
                    });
                }
            }
            TagEnd::CodeBlock => self.end_code_block(),
            TagEnd::List(_) => {
                self.list_stack.pop();
            }
            TagEnd::Item => self.end_item(),
            TagEnd::Paragraph => self.end_paragraph(),
            TagEnd::Strong => {
                self.strong = self.strong.saturating_sub(1);
            }
            TagEnd::Emphasis => {
                self.emphasis = self.emphasis.saturating_sub(1);
            }
            TagEnd::Strikethrough => {
                self.strikethrough = self.strikethrough.saturating_sub(1);
            }
            TagEnd::Table => {
                if let Some(table) = self.table.take() {
                    let alignments = table.alignments.clone();
                    let (rows, spillover) = split_table_rows_and_spillover(table);
                    for line in render_table_lines(&rows, &alignments) {
                        self.push_output(line);
                    }
                    for spillover_text in spillover {
                        self.push_output(PreviewLine::Plain(spillover_text));
                    }
                }
            }
            TagEnd::TableHead | TagEnd::TableRow => {
                if let Some(table) = &mut self.table
                    && let Some(row) = table.current_row.take()
                {
                    table.rows.push(row);
                    table
                        .row_has_pipe_syntax
                        .push(table.current_row_has_pipe_syntax);
                    table.current_row_has_pipe_syntax = false;
                }
            }
            TagEnd::TableCell => {
                if let Some(table) = &mut self.table
                    && let Some(cell) = table.current_cell.take()
                    && let Some(row) = &mut table.current_row
                {
                    row.push(cell.trimmed());
                }
            }
            TagEnd::BlockQuote(_) => {
                self.blockquote_depth = self.blockquote_depth.saturating_sub(1);
                self.blockquote_frames.pop();
            }
            TagEnd::Link => self.end_link(),
            TagEnd::HtmlBlock
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
        let depth = self.list_stack.len().saturating_sub(1);
        let marker = match self
            .list_stack
            .last_mut()
            .and_then(|list| list.next.as_mut())
        {
            Some(next) => {
                let marker = format!("{}{next}.", "  ".repeat(depth));
                *next += 1;
                marker
            }
            None => format!("{}•", "  ".repeat(depth)),
        };
        self.item_stack.push(ItemState {
            marker,
            spans: Vec::new(),
            checkbox: None,
            emitted: false,
        });
    }

    fn end_item(&mut self) {
        let Some(item) = self.item_stack.pop() else {
            return;
        };
        if item.emitted {
            return;
        }
        self.push_item_output(item);
    }

    fn start_list(&mut self, start: Option<u64>) {
        if !self.list_stack.is_empty() {
            self.emit_current_item_if_needed();
        }
        self.list_stack.push(ListState { next: start });
    }

    fn emit_current_item_if_needed(&mut self) {
        let Some(item) = self.item_stack.last_mut() else {
            return;
        };
        if item.emitted || item.spans.is_empty() {
            return;
        }
        let line = item.preview_line();
        item.emitted = true;
        self.push_output(line);
    }

    fn push_item_output(&mut self, item: ItemState) {
        self.push_output(item.preview_line());
    }

    fn start_paragraph(&mut self) {
        if !self.inline_spans.is_empty() {
            return;
        }
        let continuing_blockquote = self.blockquote_depth > 0
            && self
                .blockquote_frames
                .last()
                .is_some_and(|frame| !frame.first_line);
        let continuing_list_item =
            self.blockquote_depth == 0 && self.item_stack.last().is_some_and(|item| item.emitted);
        if continuing_blockquote || continuing_list_item {
            self.push_output(PreviewLine::Plain(String::new()));
        }
    }

    fn end_paragraph(&mut self) {
        if self
            .item_stack
            .last()
            .is_some_and(|item| !item.emitted && !item.spans.is_empty())
        {
            self.emit_current_item_if_needed();
        } else {
            self.flush_inline_spans();
        }
    }

    fn text(&mut self, text: &str) {
        self.text_with_range(text, 0..0);
    }

    fn text_with_range(&mut self, text: &str, range: Range<usize>) {
        self.resolve_pending_local_link_soft_break(text);
        let suppress_local_link_label = self.suppressing_local_link_label();
        let table_span = (!suppress_local_link_label).then(|| self.text_span(text.to_string()));
        if let Some(table) = &mut self.table
            && let Some(cell) = &mut table.current_cell
        {
            if let Some(span) = table_span {
                cell.push_span(&span);
            }
            return;
        }
        if self.code_block {
            self.push_code_text(text);
            return;
        }
        if let Some(heading) = &mut self.heading {
            if !suppress_local_link_label {
                heading.text.push_str(text);
            }
            return;
        }
        if suppress_local_link_label {
            return;
        }
        if self.link.is_none() && self.extend_trailing_bare_url_span(text, range.clone()) {
            self.line_ends_with_local_link_target = false;
            return;
        }
        if self.link.is_none()
            && let Some(spans) = bare_url_spans(text, range.clone())
        {
            for span in spans {
                self.push_span_to_context(span);
            }
            self.line_ends_with_local_link_target = false;
            return;
        }
        if let Some(item) = self.item_stack.last() {
            let span = self.text_span(text.to_string());
            if item.emitted {
                self.push_continuation_span(span);
            } else if let Some(item) = self.item_stack.last_mut() {
                item.spans.push(span);
            }
            return;
        }
        let span = self.text_span(text.to_string());
        self.inline_spans.push(span);
        self.line_ends_with_local_link_target = false;
    }

    fn code(&mut self, code: &str) {
        self.resolve_pending_local_link_soft_break(code);
        let suppress_local_link_label = self.suppressing_local_link_label();
        if let Some(table) = &mut self.table
            && let Some(cell) = &mut table.current_cell
        {
            if !suppress_local_link_label {
                cell.push_span(&PreviewSpan::Code(code.to_string()));
            }
            return;
        }
        if let Some(heading) = &mut self.heading {
            if !suppress_local_link_label {
                heading.text.push_str(code);
            }
            return;
        }
        if suppress_local_link_label {
            return;
        }
        let span = PreviewSpan::Code(code.to_string());
        if let Some(item) = self.item_stack.last() {
            if item.emitted {
                self.push_continuation_span(span);
            } else if let Some(item) = self.item_stack.last_mut() {
                item.spans.push(span);
            }
        } else {
            self.inline_spans.push(span);
        }
        self.line_ends_with_local_link_target = false;
    }

    fn html(&mut self, html: &str) {
        if !html.contains('\n') {
            self.text(html);
            return;
        }
        if self.code_block
            || self.heading.is_some()
            || self
                .table
                .as_ref()
                .is_some_and(|table| table.current_cell.is_some())
            || self.item_stack.last().is_some_and(|item| !item.emitted)
        {
            self.text(html);
            return;
        }
        self.flush_inline_spans();
        for line in html.lines() {
            self.push_output(PreviewLine::Plain(line.to_string()));
        }
    }

    fn push_continuation_span(&mut self, span: PreviewSpan) {
        if self.inline_spans.is_empty()
            && self.blockquote_depth == 0
            && let Some(item) = self.item_stack.last()
        {
            self.inline_spans
                .push(PreviewSpan::Plain(item.continuation_prefix()));
        }
        self.inline_spans.push(span);
    }

    fn start_blockquote(&mut self) {
        let parent_prefix = self
            .blockquote_frames
            .last()
            .map(|frame| frame.current_prefix().to_string())
            .unwrap_or_default();
        let item_prefix = self.blockquote_item_prefix();
        let prefix = format!("{parent_prefix}{}> ", item_prefix.first);
        let continuation_prefix = format!("{parent_prefix}{}> ", item_prefix.continuation);
        self.blockquote_depth += 1;
        self.blockquote_frames.push(BlockQuoteFrame {
            prefix,
            continuation_prefix,
            first_line: true,
        });
    }

    fn blockquote_item_prefix(&mut self) -> ItemBlockQuotePrefix {
        let Some(item) = self.item_stack.last_mut() else {
            return ItemBlockQuotePrefix::default();
        };
        if !item.emitted && item.spans.is_empty() {
            item.emitted = true;
            return ItemBlockQuotePrefix {
                first: format!("{} ", item.marker),
                continuation: item.continuation_prefix(),
            };
        }
        if !item.emitted {
            let line = item.preview_line();
            item.emitted = true;
            let prefix = item.continuation_prefix();
            self.push_output(line);
            return ItemBlockQuotePrefix {
                first: prefix.clone(),
                continuation: prefix,
            };
        }
        let prefix = item.continuation_prefix();
        ItemBlockQuotePrefix {
            first: prefix.clone(),
            continuation: prefix,
        }
    }

    fn soft_break(&mut self) {
        let suppress_local_link_label = self.suppressing_local_link_label();
        if self.code_block {
            self.push_output(PreviewLine::Code(String::new()));
        } else if self.line_ends_with_local_link_target {
            self.pending_local_link_soft_break = true;
            self.line_ends_with_local_link_target = false;
        } else if let Some(table) = &mut self.table
            && let Some(cell) = &mut table.current_cell
        {
            if !suppress_local_link_label {
                cell.push_text(" ");
            }
        } else if let Some(item) = self.item_stack.last() {
            if item.emitted && self.blockquote_depth > 0 {
                self.flush_inline_spans();
            } else if item.emitted {
                self.inline_spans.push(PreviewSpan::Plain(" ".to_string()));
            } else if let Some(item) = self.item_stack.last_mut() {
                item.spans.push(PreviewSpan::Plain(" ".to_string()));
            }
        } else {
            self.flush_inline_spans();
        }
    }

    fn push_code_text(&mut self, text: &str) {
        if self.code_block_language.is_some() {
            self.code_block_buffer.push_str(text);
            return;
        }
        for line in text.split('\n') {
            self.push_output(PreviewLine::Code(format!(
                "{}{line}",
                self.code_block_prefix
            )));
        }
    }

    fn start_code_block(&mut self, kind: CodeBlockKind<'_>) {
        if !self.item_stack.is_empty() {
            self.emit_current_item_if_needed();
        }
        self.code_block = true;
        self.code_block_language = match kind {
            CodeBlockKind::Fenced(info) => crate::syntax::language_from_info_string(info.as_ref()),
            CodeBlockKind::Indented => None,
        };
        self.code_block_prefix = if self.list_stack.is_empty() {
            String::new()
        } else {
            "  ".repeat(self.list_stack.len())
        };
        self.code_block_buffer.clear();
    }

    fn end_code_block(&mut self) {
        if let Some(language) = self.code_block_language.take() {
            let code = std::mem::take(&mut self.code_block_buffer);
            if let Some(lines) = crate::syntax::highlight_code(&code, &language) {
                for line in lines {
                    let spans = line
                        .into_iter()
                        .map(|span| PreviewSpan::Syntax {
                            text: span.text,
                            style: span.style,
                        })
                        .collect::<Vec<_>>();
                    self.push_output(PreviewLine::CodeSpans(self.prefixed_code_spans(spans)));
                }
            } else {
                self.push_code_plain(&code);
            }
        }
        self.code_block = false;
        self.code_block_buffer.clear();
        self.code_block_prefix.clear();
    }

    fn push_code_plain(&mut self, code: &str) {
        let mut emitted = false;
        for line in code.lines() {
            emitted = true;
            self.push_output(PreviewLine::Code(format!(
                "{}{line}",
                self.code_block_prefix
            )));
        }
        if !emitted {
            self.push_output(PreviewLine::Code(self.code_block_prefix.clone()));
        }
    }

    fn prefixed_code_spans(&self, spans: Vec<PreviewSpan>) -> Vec<PreviewSpan> {
        if self.code_block_prefix.is_empty() {
            return spans;
        }
        let mut prefixed = Vec::with_capacity(spans.len() + 1);
        prefixed.push(PreviewSpan::Plain(self.code_block_prefix.clone()));
        prefixed.extend(spans);
        prefixed
    }

    fn flush_inline_spans(&mut self) {
        if self.inline_spans.is_empty() || self.item_stack.last().is_some_and(|item| !item.emitted)
        {
            return;
        }
        let spans = std::mem::take(&mut self.inline_spans);
        if spans.len() == 1
            && let PreviewSpan::Plain(text) = &spans[0]
        {
            self.push_output(PreviewLine::Plain(text.clone()));
            return;
        }
        self.push_output(PreviewLine::Styled(spans));
    }

    fn finish(&mut self) {
        self.flush_inline_spans();
    }

    fn text_span(&self, text: String) -> PreviewSpan {
        if let Some(link) = &self.link {
            return PreviewSpan::Link {
                text,
                target: link.destination.clone(),
                source_range: link.source_range,
                local: false,
            };
        }
        let style = self.current_text_style();
        if style == PreviewTextStyle::default() {
            PreviewSpan::Plain(text)
        } else if style.bold && !style.italic && !style.underline && !style.strikethrough {
            PreviewSpan::Bold(text)
        } else {
            PreviewSpan::StyledText { text, style }
        }
    }

    fn current_text_style(&self) -> PreviewTextStyle {
        PreviewTextStyle {
            bold: self.strong > 0,
            italic: self.emphasis > 0,
            underline: false,
            strikethrough: self.strikethrough > 0,
        }
    }

    fn push_output(&mut self, line: PreviewLine) {
        if self.blockquote_depth == 0 {
            self.output.push(line);
        } else {
            let prefix = self
                .blockquote_frames
                .last()
                .map(|frame| frame.current_prefix().to_string())
                .unwrap_or_else(|| "> ".repeat(self.blockquote_depth));
            self.output.push(PreviewLine::BlockQuote {
                depth: self.blockquote_depth,
                prefix,
                line: Box::new(line),
            });
            if let Some(frame) = self.blockquote_frames.last_mut() {
                frame.first_line = false;
            }
        }
    }

    fn start_link(&mut self, destination: &str, range: Range<usize>) {
        let local_target_display = is_local_path_like_link(destination)
            .then(|| render_local_link_target(destination, self.cwd.as_deref()))
            .flatten();
        self.link = Some(LinkState {
            destination: destination.to_string(),
            source_range: range.into(),
            local_target_display,
        });
    }

    fn end_link(&mut self) {
        let Some(link) = self.link.take() else {
            return;
        };
        if let Some(local_target_display) = link.local_target_display {
            self.push_span_to_context(PreviewSpan::Link {
                text: local_target_display,
                target: link.destination,
                source_range: link.source_range,
                local: true,
            });
            self.line_ends_with_local_link_target = true;
        } else {
            let target = link.destination;
            self.push_span_to_context(PreviewSpan::Plain(" (".to_string()));
            self.push_span_to_context(PreviewSpan::Link {
                text: target.clone(),
                target,
                source_range: link.source_range,
                local: false,
            });
            self.push_span_to_context(PreviewSpan::Plain(")".to_string()));
            self.line_ends_with_local_link_target = false;
        }
    }

    fn suppressing_local_link_label(&self) -> bool {
        self.link
            .as_ref()
            .and_then(|link| link.local_target_display.as_ref())
            .is_some()
    }

    fn resolve_pending_local_link_soft_break(&mut self, next_text: &str) {
        if !self.pending_local_link_soft_break {
            return;
        }
        self.pending_local_link_soft_break = false;
        if next_text.trim_start().starts_with(':') {
            return;
        }
        self.push_span_to_current(PreviewSpan::Plain(" ".to_string()));
    }

    fn push_span_to_current(&mut self, span: PreviewSpan) {
        self.push_span_to_context(span);
    }

    fn push_span_to_context(&mut self, span: PreviewSpan) {
        if let Some(table) = &mut self.table
            && let Some(cell) = &mut table.current_cell
        {
            cell.push_span(&span);
            return;
        }
        if let Some(item) = self.item_stack.last() {
            if item.emitted {
                self.push_continuation_span(span);
            } else if let Some(item) = self.item_stack.last_mut() {
                item.spans.push(span);
            }
        } else {
            self.inline_spans.push(span);
        }
    }

    fn extend_trailing_bare_url_span(&mut self, text: &str, range: Range<usize>) -> bool {
        if text.is_empty() || text.chars().any(char::is_whitespace) {
            return false;
        }
        let spans = if let Some(item) = self.item_stack.last_mut() {
            if item.emitted {
                &mut self.inline_spans
            } else {
                &mut item.spans
            }
        } else {
            &mut self.inline_spans
        };
        let Some(PreviewSpan::Link {
            text: link_text,
            target,
            source_range,
            local: false,
        }) = spans.last_mut()
        else {
            return false;
        };
        if !(target.starts_with("https://") || target.starts_with("http://")) {
            return false;
        }
        let (url_suffix, trailing) = split_bare_url_trailing_punctuation(text);
        if url_suffix.is_empty() {
            spans.push(PreviewSpan::Plain(trailing.to_string()));
            return true;
        }
        link_text.push_str(url_suffix);
        target.push_str(url_suffix);
        source_range.end = range.start + url_suffix.len();
        if !trailing.is_empty() {
            spans.push(PreviewSpan::Plain(trailing.to_string()));
        }
        true
    }
}

fn span_text(spans: &[PreviewSpan]) -> String {
    spans.iter().map(PreviewSpan::text).collect()
}

fn bare_url_spans(text: &str, range: Range<usize>) -> Option<Vec<PreviewSpan>> {
    if text.is_empty() || text.chars().any(char::is_whitespace) {
        return None;
    }
    if !(text.starts_with("https://") || text.starts_with("http://")) {
        return None;
    }
    let (url_text, trailing) = split_bare_url_trailing_punctuation(text);
    if url_text.is_empty() {
        return None;
    }
    let mut spans = vec![PreviewSpan::Link {
        text: url_text.to_string(),
        target: url_text.to_string(),
        source_range: (range.start..range.start + url_text.len()).into(),
        local: false,
    }];
    if !trailing.is_empty() {
        spans.push(PreviewSpan::Plain(trailing.to_string()));
    }
    Some(spans)
}

fn split_bare_url_trailing_punctuation(text: &str) -> (&str, &str) {
    let trimmed = text.trim_end_matches(['.', ',', ';', ':', '!', '?', ')']);
    text.split_at(trimmed.len())
}

impl PreviewSpan {
    pub(crate) fn text(&self) -> &str {
        match self {
            PreviewSpan::Plain(text)
            | PreviewSpan::Bold(text)
            | PreviewSpan::StyledText { text, .. }
            | PreviewSpan::Code(text)
            | PreviewSpan::Link { text, .. }
            | PreviewSpan::Syntax { text, .. } => text,
        }
    }
}

fn is_local_path_like_link(destination: &str) -> bool {
    destination.starts_with("file://")
        || destination.starts_with('/')
        || destination.starts_with("~/")
        || destination.starts_with("./")
        || destination.starts_with("../")
        || destination.starts_with("\\\\")
        || matches!(
            destination.as_bytes(),
            [drive, b':', separator, ..]
                if drive.is_ascii_alphabetic() && matches!(separator, b'/' | b'\\')
        )
}

fn render_local_link_target(destination: &str, cwd: Option<&Path>) -> Option<String> {
    let (path_text, location_suffix) = parse_local_link_target(destination)?;
    let mut rendered = display_local_link_path(&path_text, cwd);
    if let Some(location_suffix) = location_suffix {
        rendered.push_str(&location_suffix);
    }
    Some(rendered)
}

fn parse_local_link_target(destination: &str) -> Option<(String, Option<String>)> {
    if destination.starts_with("file://") {
        let url = Url::parse(destination).ok()?;
        let path_text = file_url_to_local_path_text(&url)?;
        let location_suffix = url
            .fragment()
            .and_then(normalize_hash_location_suffix_fragment);
        return Some((path_text, location_suffix));
    }

    let mut path_text = destination;
    let mut location_suffix = None;
    if let Some((candidate_path, fragment)) = destination.rsplit_once('#')
        && let Some(normalized) = normalize_hash_location_suffix_fragment(fragment)
    {
        path_text = candidate_path;
        location_suffix = Some(normalized);
    }
    if location_suffix.is_none()
        && let Some(suffix) = extract_colon_location_suffix(path_text)
    {
        let path_len = path_text.len().saturating_sub(suffix.len());
        path_text = &path_text[..path_len];
        location_suffix = Some(suffix);
    }

    Some((expand_local_link_path(path_text), location_suffix))
}

fn normalize_hash_location_suffix_fragment(fragment: &str) -> Option<String> {
    parse_hash_location_range(fragment).map(|(start, end)| {
        let mut suffix = format!(":{}", start.line);
        if let Some(column) = start.column {
            suffix.push(':');
            suffix.push_str(column);
        }
        if let Some(end) = end {
            suffix.push('-');
            suffix.push_str(end.line);
            if let Some(column) = end.column {
                suffix.push(':');
                suffix.push_str(column);
            }
        }
        suffix
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HashLocationPoint<'a> {
    line: &'a str,
    column: Option<&'a str>,
}

fn parse_hash_location_range(
    fragment: &str,
) -> Option<(HashLocationPoint<'_>, Option<HashLocationPoint<'_>>)> {
    let (start, end) = match fragment.split_once('-') {
        Some((start, end)) => (start, Some(end)),
        None => (fragment, None),
    };
    Some((
        parse_hash_location_point(start)?,
        match end {
            Some(end) => Some(parse_hash_location_point(end)?),
            None => None,
        },
    ))
}

fn parse_hash_location_point(point: &str) -> Option<HashLocationPoint<'_>> {
    let point = point.strip_prefix('L')?;
    if point.is_empty() {
        return None;
    }
    match point.split_once('C') {
        Some((line, column))
            if !line.is_empty()
                && !column.is_empty()
                && line.chars().all(|ch| ch.is_ascii_digit())
                && column.chars().all(|ch| ch.is_ascii_digit()) =>
        {
            Some(HashLocationPoint {
                line,
                column: Some(column),
            })
        }
        None if point.chars().all(|ch| ch.is_ascii_digit()) => Some(HashLocationPoint {
            line: point,
            column: None,
        }),
        _ => None,
    }
}

fn extract_colon_location_suffix(path_text: &str) -> Option<String> {
    let (path, suffix) = path_text.rsplit_once(':')?;
    if suffix.is_empty() {
        return None;
    }
    let suffix_start = path_text.len().saturating_sub(suffix.len() + 1);
    let mut suffix_text = &path_text[suffix_start..];
    if is_location_suffix(suffix_text) {
        if let Some((prefix, previous)) = path.rsplit_once(':') {
            let candidate_start = path_text
                .len()
                .saturating_sub(previous.len() + suffix.len() + 2);
            let candidate = &path_text[candidate_start..];
            if is_location_suffix(candidate) && !looks_like_windows_drive(prefix) {
                suffix_text = candidate;
            }
        }
        return Some(suffix_text.to_string());
    }
    None
}

fn is_location_suffix(suffix: &str) -> bool {
    let suffix = suffix.strip_prefix(':').unwrap_or(suffix);
    let (start, end) = match suffix.split_once('-') {
        Some((start, end)) => (start, Some(end)),
        None => (suffix, None),
    };
    is_location_point(start) && end.is_none_or(is_location_point)
}

fn is_location_point(point: &str) -> bool {
    let mut parts = point.split(':');
    let Some(line) = parts.next() else {
        return false;
    };
    let column = parts.next();
    parts.next().is_none()
        && !line.is_empty()
        && line.chars().all(|ch| ch.is_ascii_digit())
        && column
            .is_none_or(|column| !column.is_empty() && column.chars().all(|ch| ch.is_ascii_digit()))
}

fn looks_like_windows_drive(path_text: &str) -> bool {
    matches!(path_text.as_bytes(), [drive] if drive.is_ascii_alphabetic())
}

fn expand_local_link_path(path_text: &str) -> String {
    let decoded = percent_decode_path_text(path_text);
    if let Some(rest) = decoded.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return normalize_local_link_path_text(&Path::new(&home).join(rest).to_string_lossy());
    }
    normalize_local_link_path_text(&decoded)
}

fn percent_decode_path_text(path_text: &str) -> String {
    percent_decode_bytes(path_text.as_bytes())
}

fn percent_decode_bytes(bytes: &[u8]) -> String {
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
        {
            decoded.push(high * 16 + low);
            index += 3;
            continue;
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn file_url_to_local_path_text(url: &Url) -> Option<String> {
    if let Ok(path) = url.to_file_path() {
        return Some(normalize_local_link_path_text(&path.to_string_lossy()));
    }

    let mut path_text = percent_decode_path_text(url.path());
    if let Some(host) = url.host_str()
        && !host.is_empty()
        && host != "localhost"
    {
        path_text = format!("//{host}{path_text}");
    } else if matches!(
        path_text.as_bytes(),
        [b'/', drive, b':', b'/', ..] if drive.is_ascii_alphabetic()
    ) {
        path_text.remove(0);
    }
    Some(normalize_local_link_path_text(&path_text))
}

fn normalize_local_link_path_text(path_text: &str) -> String {
    if let Some(rest) = path_text.strip_prefix("\\\\") {
        format!("//{}", rest.replace('\\', "/").trim_start_matches('/'))
    } else {
        path_text.replace('\\', "/")
    }
}

fn is_absolute_local_link_path(path_text: &str) -> bool {
    path_text.starts_with('/')
        || path_text.starts_with("//")
        || matches!(
            path_text.as_bytes(),
            [drive, b':', b'/', ..] if drive.is_ascii_alphabetic()
        )
}

fn trim_trailing_local_path_separator(path_text: &str) -> &str {
    if path_text == "/" || path_text == "//" {
        return path_text;
    }
    if matches!(path_text.as_bytes(), [drive, b':', b'/'] if drive.is_ascii_alphabetic()) {
        return path_text;
    }
    path_text.trim_end_matches('/')
}

fn strip_local_path_prefix<'a>(path_text: &'a str, cwd_text: &str) -> Option<&'a str> {
    let path_text = trim_trailing_local_path_separator(path_text);
    let cwd_text = trim_trailing_local_path_separator(cwd_text);
    if path_text == cwd_text {
        return None;
    }
    if cwd_text == "/" || cwd_text == "//" {
        return path_text.strip_prefix('/');
    }
    path_text
        .strip_prefix(cwd_text)
        .and_then(|rest| rest.strip_prefix('/'))
}

fn display_local_link_path(path_text: &str, cwd: Option<&Path>) -> String {
    let path_text = normalize_local_link_path_text(path_text);
    if !is_absolute_local_link_path(&path_text) {
        return path_text;
    }
    if let Some(cwd) = cwd {
        let cwd_text = normalize_local_link_path_text(&cwd.to_string_lossy());
        if let Some(stripped) = strip_local_path_prefix(&path_text, &cwd_text) {
            return stripped.to_string();
        }
    }
    path_text
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

fn render_table_lines(rows: &[Vec<TableCell>], alignments: &[TableAlignment]) -> Vec<PreviewLine> {
    let column_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    if column_count == 0 {
        return Vec::new();
    }

    let widths = (0..column_count)
        .map(|column| {
            rows.iter()
                .filter_map(|row| row.get(column))
                .map(|cell| display_width(&cell.text))
                .max()
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();
    let alignments = (0..column_count)
        .map(|column| {
            alignments
                .get(column)
                .copied()
                .unwrap_or(TableAlignment::None)
        })
        .collect::<Vec<_>>();

    let metrics = collect_table_column_metrics_for_table(rows, &widths);
    let mut rendered = Vec::new();
    let headers = row_texts(&rows[0]);
    for (index, row) in rows.iter().enumerate() {
        if index == 0 {
            rendered.push(PreviewLine::TableHeader {
                cells: row_texts(row),
                links: row_links(row),
                widths: widths.clone(),
                alignments: alignments.clone(),
                metrics: metrics.clone(),
            });
            rendered.push(PreviewLine::TableDivider {
                widths: widths.clone(),
                kind: TableDividerKind::Header,
                metrics: metrics.clone(),
            });
        } else {
            rendered.push(PreviewLine::TableRow {
                headers: headers.clone(),
                cells: row_texts(row),
                links: row_links(row),
                widths: widths.clone(),
                alignments: alignments.clone(),
                metrics: metrics.clone(),
            });
            if index + 1 < rows.len() {
                rendered.push(PreviewLine::TableDivider {
                    widths: widths.clone(),
                    kind: TableDividerKind::Body,
                    metrics: metrics.clone(),
                });
            }
        }
    }
    rendered
}

fn split_table_rows_and_spillover(table: TableParseState) -> (Vec<Vec<TableCell>>, Vec<String>) {
    let mut rows = Vec::with_capacity(table.rows.len());
    let mut spillover = Vec::new();

    for (index, row) in table.rows.into_iter().enumerate() {
        let has_pipe_syntax = table
            .row_has_pipe_syntax
            .get(index)
            .copied()
            .unwrap_or(true);
        if index > 0 && is_table_spillover_row(&row, has_pipe_syntax) {
            let text = row_texts(&row).join(" ").trim().to_string();
            if !text.is_empty() {
                spillover.push(text);
            }
        } else {
            rows.push(row);
        }
    }

    (rows, spillover)
}

fn is_table_spillover_row(row: &[TableCell], has_pipe_syntax: bool) -> bool {
    if has_pipe_syntax {
        return false;
    }
    let non_empty = row
        .iter()
        .filter(|cell| !cell.text.trim().is_empty())
        .collect::<Vec<_>>();
    non_empty.len() == 1
}

fn row_texts(row: &[TableCell]) -> Vec<String> {
    row.iter().map(|cell| cell.text.clone()).collect()
}

fn row_links(row: &[TableCell]) -> Vec<PreviewTableLink> {
    row.iter()
        .enumerate()
        .flat_map(|(cell_index, cell)| {
            cell.links.iter().map(move |link| PreviewTableLink {
                cell: cell_index,
                text: link.text.clone(),
                columns: link.columns.clone(),
                target: link.target.clone(),
                source_range: link.source_range,
                local: link.local,
            })
        })
        .collect()
}

pub(crate) fn padded_table_row(
    row: &[String],
    widths: &[usize],
    alignments: &[TableAlignment],
) -> String {
    widths
        .iter()
        .enumerate()
        .map(|(index, width)| {
            let cell = row.get(index).map(String::as_str).unwrap_or("");
            pad_aligned(
                cell,
                *width,
                alignments
                    .get(index)
                    .copied()
                    .unwrap_or(TableAlignment::None),
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

#[cfg(test)]
fn fitted_table_widths_for_cells(
    headers: &[String],
    cells: &[String],
    widths: &[usize],
    max_width: usize,
) -> Vec<usize> {
    let metrics = collect_table_column_metrics(headers, cells, widths);
    fitted_table_widths_with_metrics(widths, max_width, &metrics)
}

fn fitted_table_widths_with_metrics(
    widths: &[usize],
    max_width: usize,
    metrics: &[TableColumnMetric],
) -> Vec<usize> {
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
    let floors = metrics
        .iter()
        .enumerate()
        .map(|(index, metric)| {
            preferred_column_floor(metric, min_column_width)
                .min(*fitted.get(index).unwrap_or(&metric.max_width))
        })
        .collect::<Vec<_>>();
    shrink_widths_to_target(&mut fitted, &floors, target, Some(metrics));
    if fitted.iter().sum::<usize>() > target {
        let hard_floors = vec![min_column_width; fitted.len()];
        shrink_widths_to_target(&mut fitted, &hard_floors, target, Some(metrics));
    }
    fitted
}

fn shrink_widths_to_target(
    widths: &mut [usize],
    floors: &[usize],
    target: usize,
    metrics: Option<&[TableColumnMetric]>,
) {
    while widths.iter().sum::<usize>() > target {
        let Some(index) = widths
            .iter()
            .enumerate()
            .filter(|(index, width)| **width > floors[*index])
            .min_by_key(|(index, width)| {
                let priority = metrics
                    .and_then(|metrics| metrics.get(*index))
                    .map(|metric| column_shrink_priority(metric.kind))
                    .unwrap_or(0);
                let slack = width.saturating_sub(floors[*index]);
                (priority, usize::MAX.saturating_sub(slack))
            })
            .map(|(index, _)| index)
        else {
            break;
        };
        widths[index] -= 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TableColumnKind {
    TokenHeavy,
    Narrative,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TableColumnMetric {
    max_width: usize,
    header_token_width: usize,
    body_token_width: usize,
    kind: TableColumnKind,
}

/// Classify one column from a single representative cell.
fn column_metric(header: &str, cell: &str, width: usize) -> TableColumnMetric {
    let word_count = cell.split_whitespace().count();
    let token_width = longest_token_width(cell);
    let cell_width = display_width(cell);
    let long_token_count = cell
        .split_whitespace()
        .filter(|token| display_width(token) >= 20)
        .count();
    let kind = if long_token_count > 0
        && long_token_count >= word_count.saturating_sub(long_token_count)
    {
        TableColumnKind::TokenHeavy
    } else if word_count >= 4 || cell_width >= 28 {
        TableColumnKind::Narrative
    } else {
        TableColumnKind::Compact
    };
    TableColumnMetric {
        max_width: width.max(1),
        header_token_width: longest_token_width(header),
        body_token_width: token_width,
        kind,
    }
}

#[cfg(test)]
fn collect_table_column_metrics(
    headers: &[String],
    cells: &[String],
    widths: &[usize],
) -> Vec<TableColumnMetric> {
    widths
        .iter()
        .enumerate()
        .map(|(index, width)| {
            let header = headers.get(index).map(String::as_str).unwrap_or("");
            let cell = cells.get(index).map(String::as_str).unwrap_or("");
            column_metric(header, cell, *width)
        })
        .collect()
}

/// Compute one metric per column from ALL table rows, so every line of the
/// table shares the same fitting decision and the columns line up vertically.
/// The widest cell in a column is its representative — it is the cell that
/// determines how much space the column wants.
fn collect_table_column_metrics_for_table(
    rows: &[Vec<TableCell>],
    widths: &[usize],
) -> Vec<TableColumnMetric> {
    let header_texts = rows.first().map(|row| row_texts(row)).unwrap_or_default();
    widths
        .iter()
        .enumerate()
        .map(|(column, width)| {
            let header = header_texts.get(column).map(String::as_str).unwrap_or("");
            let representative = rows
                .iter()
                .filter_map(|row| row.get(column))
                .map(|cell| cell.text.as_str())
                .max_by_key(|text| display_width(text))
                .unwrap_or("");
            column_metric(header, representative, *width)
        })
        .collect()
}

/// Uniform fallback metrics (treat every column as plain narrative text),
/// used when a table line carries no precomputed table-wide metrics.
fn uniform_table_column_metrics(widths: &[usize]) -> Vec<TableColumnMetric> {
    widths
        .iter()
        .map(|width| TableColumnMetric {
            max_width: (*width).max(1),
            header_token_width: 0,
            body_token_width: 0,
            kind: TableColumnKind::Narrative,
        })
        .collect()
}

/// Resolve the metrics a table line should fit against: its stored table-wide
/// metrics when present, otherwise a uniform fallback derived from `widths`.
fn resolve_table_metrics(
    metrics: &[TableColumnMetric],
    widths: &[usize],
) -> Vec<TableColumnMetric> {
    if metrics.len() == widths.len() {
        metrics.to_vec()
    } else {
        uniform_table_column_metrics(widths)
    }
}

fn longest_token_width(text: &str) -> usize {
    text.split_whitespace()
        .map(display_width)
        .max()
        .unwrap_or(0)
}

fn preferred_column_floor(metric: &TableColumnMetric, min_column_width: usize) -> usize {
    let token_target = match metric.kind {
        TableColumnKind::Narrative | TableColumnKind::TokenHeavy => 16,
        TableColumnKind::Compact => metric
            .header_token_width
            .max(metric.body_token_width.min(16)),
    };
    token_target.max(min_column_width).min(metric.max_width)
}

fn column_shrink_priority(kind: TableColumnKind) -> usize {
    match kind {
        TableColumnKind::TokenHeavy => 0,
        TableColumnKind::Narrative => 1,
        TableColumnKind::Compact => 2,
    }
}

pub(crate) fn wrapped_table_row_texts(
    cells: &[String],
    widths: &[usize],
    alignments: &[TableAlignment],
    metrics: &[TableColumnMetric],
    max_width: usize,
) -> Vec<String> {
    let fitted_widths = fitted_table_widths_with_metrics(widths, max_width, metrics);
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
                    pad_aligned(
                        &line,
                        *width,
                        alignments
                            .get(index)
                            .copied()
                            .unwrap_or(TableAlignment::None),
                    )
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
        PreviewLine::BlockQuote { prefix, line, .. } => {
            table_line_text(line).map(|line| format!("{prefix}{line}"))
        }
        PreviewLine::TableHeader {
            cells,
            widths,
            alignments,
            ..
        }
        | PreviewLine::TableRow {
            cells,
            widths,
            alignments,
            ..
        } => Some(
            padded_table_row(cells, widths, alignments)
                .trim_end()
                .to_string(),
        ),
        PreviewLine::TableDivider { widths, kind, .. } => Some(table_divider(widths, *kind)),
        _ => None,
    }
}

pub(crate) fn wrapped_table_line_texts(
    line: &PreviewLine,
    max_width: usize,
) -> Option<Vec<String>> {
    match line {
        PreviewLine::BlockQuote { prefix, line, .. } => {
            let inner_width = max_width.saturating_sub(prefix.len()).max(1);
            let lines = wrapped_table_line_texts(line, inner_width)?;
            Some(
                lines
                    .into_iter()
                    .map(|line| format!("{prefix}{line}"))
                    .collect(),
            )
        }
        PreviewLine::TableHeader {
            widths, metrics, ..
        } if should_render_table_records(
            widths,
            &resolve_table_metrics(metrics, widths),
            max_width,
        ) =>
        {
            Some(Vec::new())
        }
        PreviewLine::TableHeader {
            cells,
            widths,
            alignments,
            metrics,
            ..
        } => Some(wrapped_table_row_texts(
            cells,
            widths,
            alignments,
            &resolve_table_metrics(metrics, widths),
            max_width,
        )),
        PreviewLine::TableRow {
            headers,
            cells,
            widths,
            metrics,
            ..
        } if should_render_table_records(
            widths,
            &resolve_table_metrics(metrics, widths),
            max_width,
        ) =>
        {
            Some(record_table_row_texts(headers, cells, max_width))
        }
        PreviewLine::TableRow {
            cells,
            widths,
            alignments,
            metrics,
            ..
        } => Some(wrapped_table_row_texts(
            cells,
            widths,
            alignments,
            &resolve_table_metrics(metrics, widths),
            max_width,
        )),
        PreviewLine::TableDivider {
            widths,
            metrics,
            kind,
        } if should_render_table_records(
            widths,
            &resolve_table_metrics(metrics, widths),
            max_width,
        ) =>
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
        PreviewLine::TableDivider {
            widths,
            metrics,
            kind,
        } => Some(vec![table_divider(
            &fitted_table_widths_with_metrics(
                widths,
                max_width,
                &resolve_table_metrics(metrics, widths),
            ),
            *kind,
        )]),
        _ => None,
    }
}

fn should_render_table_records(
    widths: &[usize],
    metrics: &[TableColumnMetric],
    max_width: usize,
) -> bool {
    if widths.len() < 2 {
        return false;
    }
    let original_width =
        widths.iter().sum::<usize>() + TABLE_COLUMN_GAP.saturating_mul(widths.len() - 1);
    if original_width <= max_width {
        return false;
    }
    if max_width < MIN_TABLE_COLUMN_MODE_WIDTH {
        return true;
    }
    let fitted = fitted_table_widths_with_metrics(widths, max_width, metrics);
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

fn pad_aligned(text: &str, width: usize, alignment: TableAlignment) -> String {
    let padding = width.saturating_sub(display_width(text));
    match alignment {
        TableAlignment::Right => format!("{}{text}", " ".repeat(padding)),
        TableAlignment::Center => {
            let left = padding / 2;
            let right = padding.saturating_sub(left);
            format!("{}{text}{}", " ".repeat(left), " ".repeat(right))
        }
        TableAlignment::None | TableAlignment::Left => format!("{text}{}", " ".repeat(padding)),
    }
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
    use crate::inventory::{self, StageDir};
    use assert_fs::prelude::*;

    fn line_text(line: &PreviewLine) -> String {
        match line {
            PreviewLine::BlockQuote { prefix, line, .. } => {
                format!("{prefix}{}", line_text(line))
            }
            PreviewLine::Heading { text, .. }
            | PreviewLine::Code(text)
            | PreviewLine::Plain(text) => text.clone(),
            PreviewLine::CodeSpans(spans) => spans.iter().map(preview_span_text).collect(),
            PreviewLine::TableHeader { .. }
            | PreviewLine::TableDivider { .. }
            | PreviewLine::TableRow { .. } => table_line_text(line).expect("table line text"),
            PreviewLine::Checkbox {
                marker,
                checked,
                text,
            } => {
                let checkbox = if *checked { "[x]" } else { "[ ]" };
                format!("{marker} {checkbox} {text}")
            }
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
            PreviewSpan::Plain(text)
            | PreviewSpan::Bold(text)
            | PreviewSpan::StyledText { text, .. }
            | PreviewSpan::Code(text)
            | PreviewSpan::Link { text, .. } => text,
            PreviewSpan::Syntax { text, .. } => text,
        }
    }

    fn preview_text(preview: &Preview) -> Vec<String> {
        preview.lines.iter().map(line_text).collect()
    }

    fn render_snapshot_lines(markdown: &str, cwd: Option<&Path>) -> Vec<String> {
        render_markdown_with_cwd(markdown, cwd)
            .iter()
            .map(line_text)
            .collect()
    }

    #[test]
    fn preview_markup_headings_become_heading() {
        let mut in_code_block = false;

        let line = markup_line("### 개요", &mut in_code_block);

        match line {
            PreviewLine::Heading { level, text } => {
                assert_eq!(level, 1);
                assert_eq!(text, "개요");
            }
            other => panic!("expected heading, got {other:?}"),
        }
        assert!(!in_code_block);
    }

    #[test]
    fn preview_markdown_preserves_heading_levels() {
        let lines = render_markdown_with_cwd("# H1\n### H3\n###### H6\n", None);

        assert!(matches!(
            &lines[0],
            PreviewLine::Heading { level: 1, text } if text == "H1"
        ));
        assert!(matches!(
            &lines[1],
            PreviewLine::Heading { level: 3, text } if text == "H3"
        ));
        assert!(matches!(
            &lines[2],
            PreviewLine::Heading { level: 6, text } if text == "H6"
        ));
    }

    #[test]
    fn preview_markup_checkboxes_become_checkbox() {
        let mut in_code_block = false;

        let checked = markup_line("- [x] Done", &mut in_code_block);
        let unchecked = markup_line("- [ ] 다음 작업", &mut in_code_block);

        match checked {
            PreviewLine::Checkbox { checked, text, .. } => {
                assert!(checked);
                assert_eq!(text, "Done");
            }
            other => panic!("expected checked checkbox, got {other:?}"),
        }
        match unchecked {
            PreviewLine::Checkbox { checked, text, .. } => {
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
    fn preview_markdown_known_language_code_block_uses_syntax_spans() {
        let lines = marked_lines(vec![
            "```rust".to_string(),
            "fn main() {}".to_string(),
            "```".to_string(),
        ]);

        let code = lines
            .iter()
            .find_map(|line| match line {
                PreviewLine::CodeSpans(spans) => Some(spans),
                _ => None,
            })
            .expect("highlighted code line");
        assert_eq!(span_text(code), "fn main() {}");
        assert!(
            code.iter().any(|span| {
                matches!(
                    span,
                    PreviewSpan::Syntax { style, .. }
                        if style.fg.is_some() || style.text_style.bold
                )
            }),
            "expected syntax style on at least one span: {code:?}"
        );
    }

    #[test]
    fn preview_markdown_code_info_metadata_still_highlights() {
        let lines = marked_lines(vec![
            "```rust,no_run".to_string(),
            "fn main() {}".to_string(),
            "```".to_string(),
        ]);

        assert!(
            lines
                .iter()
                .any(|line| matches!(line, PreviewLine::CodeSpans(_))),
            "expected info string metadata to preserve rust highlighting: {lines:?}"
        );
    }

    #[test]
    fn preview_markdown_unknown_language_code_block_stays_plain() {
        let lines = marked_lines(vec![
            "```xyzlang".to_string(),
            "hello world".to_string(),
            "```".to_string(),
        ]);

        assert_eq!(
            lines.iter().map(line_text).collect::<Vec<_>>(),
            vec!["hello world"]
        );
        assert!(matches!(lines[0], PreviewLine::Code(ref text) if text == "hello world"));
    }

    #[test]
    fn preview_markdown_code_block_with_inner_triple_backticks_stays_literal() {
        let text = render_snapshot_lines(
            r#"````text
Here is a code block:

```md
# Inside fence
- `inline code`
```
````
"#,
            None,
        );

        assert_eq!(
            text,
            vec![
                "Here is a code block:",
                "",
                "```md",
                "# Inside fence",
                "- `inline code`",
                "```",
            ]
        );
    }

    #[test]
    fn preview_markdown_code_block_preserves_trailing_blank_lines() {
        let text = render_snapshot_lines("```rust\nfn main() {}\n\n```\n", None);
        let code_index = text
            .iter()
            .position(|line| line == "fn main() {}")
            .expect("code line");

        assert_eq!(text.get(code_index + 1).map(String::as_str), Some(""));
    }

    #[test]
    fn preview_markdown_code_block_inside_list_keeps_parent_then_indented_code() {
        let text =
            render_snapshot_lines("- Item\n\n  ```xyzlang\n  first\n  second\n  ```\n", None);

        assert_eq!(text, vec!["• Item", "  first", "  second"]);
    }

    #[test]
    fn preview_markdown_inline_html_is_verbatim() {
        let text = render_snapshot_lines("Hello <span>world</span>!", None);

        assert_eq!(text, vec!["Hello <span>world</span>!"]);
    }

    #[test]
    fn preview_markdown_html_block_is_verbatim_multiline() {
        let text = render_snapshot_lines("<div>\n  <span>hi</span>\n</div>\n", None);

        assert_eq!(text, vec!["<div>", "  <span>hi</span>", "</div>"]);
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
    fn preview_markdown_emphasis_becomes_italic_style_span() {
        let lines = render_markdown_with_cwd("*Emphasis*", None);

        let PreviewLine::Styled(spans) = &lines[0] else {
            panic!("expected styled emphasis line, got {:?}", lines[0]);
        };
        assert!(matches!(
            &spans[0],
            PreviewSpan::StyledText { text, style }
                if text == "Emphasis" && style.italic && !style.bold && !style.strikethrough
        ));
    }

    #[test]
    fn preview_markdown_strikethrough_becomes_crossed_out_style_span() {
        let lines = render_markdown_with_cwd("~~Strikethrough~~", None);

        let PreviewLine::Styled(spans) = &lines[0] else {
            panic!("expected styled strikethrough line, got {:?}", lines[0]);
        };
        assert!(matches!(
            &spans[0],
            PreviewSpan::StyledText { text, style }
                if text == "Strikethrough" && style.strikethrough && !style.bold && !style.italic
        ));
    }

    #[test]
    fn preview_markdown_strong_emphasis_combines_bold_and_italic() {
        let lines = render_markdown_with_cwd("**Strong *emphasis***", None);

        let PreviewLine::Styled(spans) = &lines[0] else {
            panic!("expected styled strong emphasis line, got {:?}", lines[0]);
        };
        assert!(matches!(&spans[0], PreviewSpan::Bold(text) if text == "Strong "));
        assert!(matches!(
            &spans[1],
            PreviewSpan::StyledText { text, style }
                if text == "emphasis" && style.bold && style.italic && !style.strikethrough
        ));
    }

    #[test]
    fn preview_markdown_url_link_renders_destination_and_preserves_range() {
        let input = "[docs](https://example.com/docs)";
        let lines = render_markdown_with_cwd(input, None);

        let PreviewLine::Styled(spans) = &lines[0] else {
            panic!("expected styled link line, got {:?}", lines[0]);
        };
        assert_eq!(span_text(spans), "docs (https://example.com/docs)");
        let link = spans
            .iter()
            .find(|span| matches!(span, PreviewSpan::Link { .. }))
            .expect("link span");
        match link {
            PreviewSpan::Link {
                text,
                target,
                source_range,
                local,
            } => {
                assert_eq!(text, "docs");
                assert_eq!(target, "https://example.com/docs");
                assert_eq!(
                    *source_range,
                    PreviewSourceRange {
                        start: 0,
                        end: input.len()
                    }
                );
                assert!(!local);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn preview_markdown_bare_url_with_tilde_becomes_link_span() {
        let input = "https://www.cs.tufts.edu/~nr/cs257/archive/olin-shivers/dissertation.pdf";
        let lines = render_markdown_with_cwd(input, None);

        let PreviewLine::Styled(spans) = &lines[0] else {
            panic!("expected styled bare URL line, got {:?}", lines[0]);
        };
        assert_eq!(span_text(spans), input);
        assert!(matches!(
            &spans[0],
            PreviewSpan::Link {
                text,
                target,
                source_range,
                local: false,
            } if text == input
                && target == input
                && *source_range == (PreviewSourceRange { start: 0, end: input.len() })
        ));
    }

    #[test]
    fn preview_markdown_bare_url_entity_decoding_preserves_target_text() {
        let source = "https://example.com/a&amp;b~c";
        let destination = "https://example.com/a&b~c";
        let lines = render_markdown_with_cwd(source, None);

        assert_eq!(line_text(&lines[0]), destination);
        let PreviewLine::Styled(spans) = &lines[0] else {
            panic!("expected styled bare URL line, got {:?}", lines[0]);
        };
        assert!(matches!(
            &spans[0],
            PreviewSpan::Link {
                text,
                target,
                source_range,
                local: false,
            } if text == destination
                && target == destination
                && *source_range == (PreviewSourceRange { start: 0, end: source.len() })
        ));
    }

    #[test]
    fn preview_markdown_bare_url_keeps_sentence_punctuation_out_of_link_target() {
        let input = "https://example.com/docs.";
        let lines = render_markdown_with_cwd(input, None);

        assert_eq!(line_text(&lines[0]), input);
        let PreviewLine::Styled(spans) = &lines[0] else {
            panic!("expected styled bare URL line, got {:?}", lines[0]);
        };
        assert!(matches!(
            &spans[0],
            PreviewSpan::Link {
                text,
                target,
                source_range,
                local: false,
            } if text == "https://example.com/docs"
                && target == "https://example.com/docs"
                && *source_range == (PreviewSourceRange { start: 0, end: "https://example.com/docs".len() })
        ));
        assert!(matches!(&spans[1], PreviewSpan::Plain(text) if text == "."));
    }

    #[test]
    fn preview_markdown_table_url_with_tilde_keeps_complete_target_text() {
        let destination =
            "https://www.cs.tufts.edu/~nr/cs257/archive/olin-shivers/dissertation.pdf";
        let markdown = format!("| URL |\n| --- |\n| {destination} |\n");
        let text = render_snapshot_lines(&markdown, None);

        assert!(
            text.iter().any(|line| line.contains(destination)),
            "expected table URL to remain complete: {text:?}"
        );
    }

    #[test]
    fn preview_markdown_local_file_link_uses_normalized_target() {
        let input = "[markdown_render.rs](file:///Users/example/code/codex/codex-rs/tui/src/markdown_render.rs#L74C3)";
        let lines = render_markdown_with_cwd(input, Some(Path::new("/Users/example/code/codex")));

        let PreviewLine::Styled(spans) = &lines[0] else {
            panic!("expected styled local link line, got {:?}", lines[0]);
        };
        assert_eq!(span_text(spans), "codex-rs/tui/src/markdown_render.rs:74:3");
        assert!(matches!(
            &spans[0],
            PreviewSpan::Link {
                target,
                source_range,
                local: true,
                ..
            } if target == "file:///Users/example/code/codex/codex-rs/tui/src/markdown_render.rs#L74C3"
                && *source_range == (PreviewSourceRange { start: 0, end: input.len() })
        ));
    }

    #[test]
    fn preview_markdown_list_url_link_keeps_destination_in_item() {
        let input = "- [docs](https://example.com/docs)";
        let lines = render_markdown_with_cwd(input, None);

        assert_eq!(
            lines.iter().map(line_text).collect::<Vec<_>>(),
            vec!["• docs (https://example.com/docs)"]
        );
        let PreviewLine::ListItem { spans, .. } = &lines[0] else {
            panic!("expected list item, got {:?}", lines[0]);
        };
        let link_spans = spans
            .iter()
            .filter_map(|span| match span {
                PreviewSpan::Link {
                    target,
                    source_range,
                    ..
                } => Some((target.as_str(), *source_range)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            link_spans,
            vec![
                (
                    "https://example.com/docs",
                    PreviewSourceRange {
                        start: 2,
                        end: input.len()
                    }
                ),
                (
                    "https://example.com/docs",
                    PreviewSourceRange {
                        start: 2,
                        end: input.len()
                    }
                )
            ]
        );
    }

    #[test]
    fn preview_markdown_table_url_link_keeps_destination_in_cell() {
        let text = render_snapshot_lines(
            "| Link |\n| --- |\n| [docs](https://example.com/docs) |\n",
            None,
        );

        assert!(
            text.iter()
                .any(|line| line.contains("docs (https://example.com/docs)")),
            "expected table cell to keep URL destination: {text:?}"
        );
    }

    #[test]
    fn preview_markdown_table_local_file_link_preserves_target_metadata() {
        let input =
            "| Link |\n| --- |\n| [file](file:///Users/example/code/codex/README.md#L12) |\n";
        let lines = render_markdown_with_cwd(input, Some(Path::new("/Users/example/code/codex")));

        let PreviewLine::TableRow { cells, links, .. } = &lines[2] else {
            panic!("expected table row, got {:?}", lines[2]);
        };

        assert_eq!(cells, &vec!["README.md:12".to_string()]);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].cell, 0);
        assert_eq!(links[0].text, "README.md:12");
        assert_eq!(
            links[0].target,
            "file:///Users/example/code/codex/README.md#L12"
        );
        assert!(links[0].local);
    }

    #[test]
    fn preview_markdown_local_file_link_keeps_absolute_paths_outside_cwd() {
        let lines = render_markdown_with_cwd(
            "[README.md](/Users/example/code/codex/README.md:74)",
            Some(Path::new("/Users/example/code/codex/codex-rs/tui")),
        );

        assert_eq!(
            line_text(&lines[0]),
            "/Users/example/code/codex/README.md:74"
        );
    }

    #[test]
    fn preview_markdown_local_file_link_decodes_percent_encoded_path() {
        let text = render_snapshot_lines(
            "[file](./docs/My%20Report.md:12)",
            Some(Path::new("/Users/example/code/codex")),
        );

        assert_eq!(text, vec!["./docs/My Report.md:12"]);
    }

    #[test]
    fn preview_markdown_local_file_link_preserves_hash_range() {
        let text = render_snapshot_lines(
            "[markdown_render.rs](file:///Users/example/code/codex/codex-rs/tui/src/markdown_render.rs#L74C3-L76C9)",
            Some(Path::new("/Users/example/code/codex")),
        );

        assert_eq!(text, vec!["codex-rs/tui/src/markdown_render.rs:74:3-76:9"]);
    }

    #[test]
    fn preview_markdown_relative_local_file_link_preserves_hash_line_range() {
        let text = render_snapshot_lines(
            "[lib](./src/lib.rs#L10-L12)",
            Some(Path::new("/Users/example/code/codex")),
        );

        assert_eq!(text, vec!["./src/lib.rs:10-12"]);
    }

    #[test]
    fn preview_markdown_relative_local_file_link_preserves_hash_column_range() {
        let text = render_snapshot_lines(
            "[lib](../codex/src/lib.rs#L10C2-L12C4)",
            Some(Path::new("/Users/example/code/codex/crates/leaf")),
        );

        assert_eq!(text, vec!["../codex/src/lib.rs:10:2-12:4"]);
    }

    #[test]
    fn preview_markdown_local_file_link_keeps_invalid_hash_fragment_visible() {
        let text = render_snapshot_lines(
            "[lib](./src/lib.rs#LfooC2)",
            Some(Path::new("/Users/example/code/codex")),
        );

        assert_eq!(text, vec!["./src/lib.rs#LfooC2"]);
    }

    #[test]
    fn preview_markdown_local_file_link_preserves_colon_line_column_suffix() {
        let text = render_snapshot_lines(
            "[lib](./src/lib.rs:10:2)",
            Some(Path::new("/Users/example/code/codex")),
        );

        assert_eq!(text, vec!["./src/lib.rs:10:2"]);
    }

    #[test]
    fn preview_markdown_list_local_file_link_soft_break_before_colon_stays_inline() {
        let text = render_snapshot_lines(
            "- [binary](/Users/example/code/codex/codex-rs/README.md:93)\n  : cli is the top-level multitool binary.",
            Some(Path::new("/Users/example/code/codex")),
        );

        assert_eq!(
            text,
            vec!["• codex-rs/README.md:93: cli is the top-level multitool binary."]
        );
    }

    #[test]
    fn preview_markdown_consecutive_list_local_file_links_do_not_detach_paths() {
        let text = render_snapshot_lines(
            "- [binary](/Users/example/code/codex/codex-rs/README.md:93)\n  : cli is the top-level multitool binary.\n- [expectations](/Users/example/code/codex/codex-rs/core/README.md:1)\n  : codex-core owns the real runtime behavior.",
            Some(Path::new("/Users/example/code/codex")),
        );

        assert_eq!(
            text,
            vec![
                "• codex-rs/README.md:93: cli is the top-level multitool binary.",
                "• codex-rs/core/README.md:1: codex-core owns the real runtime behavior.",
            ]
        );
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
    fn preview_markup_blockquote_prefixes_lines() {
        let lines = marked_lines(vec![
            "> quoted".to_string(),
            ">".to_string(),
            "> - listed".to_string(),
        ]);
        let text = lines.iter().map(line_text).collect::<Vec<_>>();

        assert!(matches!(lines[0], PreviewLine::BlockQuote { depth: 1, .. }));
        assert_eq!(text[0], "> quoted");
        assert_eq!(text[1], "> • listed");
    }

    #[test]
    fn preview_markup_nested_blockquote_repeats_prefix() {
        let lines = marked_lines(vec!["> outer".to_string(), "> > inner".to_string()]);
        let text = lines.iter().map(line_text).collect::<Vec<_>>();

        assert!(text.contains(&"> outer".to_string()));
        assert!(text.contains(&"> > inner".to_string()));
    }

    #[test]
    fn preview_markup_table_inside_blockquote_prefixes_each_table_line() {
        let lines = marked_lines(vec![
            "> | Plain check | EARS |".to_string(),
            "> |---|---|".to_string(),
            "> | reason | WHEN an item enters fallen, THE MODEL SHALL record why. |".to_string(),
        ]);
        let text = lines.iter().map(line_text).collect::<Vec<_>>();

        assert!(text.iter().all(|line| line.starts_with("> ")));
        assert!(text.iter().any(|line| line.contains("Plain check")));
        assert!(text.iter().any(|line| line.contains("━━━━")));
        assert!(text.iter().any(|line| line.contains("reason")));
    }

    #[test]
    fn preview_markdown_escaped_pipes_render_in_table_cells() {
        let text = render_snapshot_lines("| Col |\n| --- |\n| a \\| b |\n", None);

        assert!(text.iter().any(|line| line.contains("a | b")));
    }

    #[test]
    fn preview_markdown_table_alignment_respects_markers() {
        let text = render_snapshot_lines(
            "| Left | Center | Right |\n|:-----|:------:|------:|\n| a | b | c |\n",
            None,
        );

        assert_eq!(text[0], "Left    Center    Right");
        assert_eq!(text[2], "a         b           c");
    }

    #[test]
    fn preview_markdown_table_aligns_second_column_when_first_column_wraps() {
        // Mixed first-column lengths: the long one wraps to multiple lines.
        // Every body row's second column must start at the same display column.
        let lines = render_markdown_with_cwd(
            "| Criteria | Verdict |\n\
             | --- | --- |\n\
             | MySQL-first schema | ALPHA expressed as plain columns |\n\
             | separation of source history derived and baseline values across all the tables | BRAVO seven tables separated cleanly here |\n\
             | payment dedup key | CHARLIE natural keys kept together |\n",
            None,
        );
        let rendered = lines
            .iter()
            .flat_map(|line| {
                wrapped_table_line_texts(line, 78).unwrap_or_else(|| vec![line_text(line)])
            })
            .collect::<Vec<_>>();

        // Display column where a token first appears (counts wide chars correctly).
        let display_col = |token: &str| -> usize {
            rendered
                .iter()
                .find_map(|line| line.find(token).map(|byte| display_width(&line[..byte])))
                .unwrap_or_else(|| panic!("token {token:?} not found in {rendered:?}"))
        };

        let header = display_col("Verdict");
        let short_a = display_col("ALPHA");
        let wrapped = display_col("BRAVO");
        let short_c = display_col("CHARLIE");

        assert_eq!(
            (short_a, wrapped, short_c),
            (header, header, header),
            "second column must start at the same display column for every row\n{rendered:#?}"
        );
    }

    #[test]
    fn preview_markdown_table_separates_logical_rows_after_wrapped_content() {
        let lines = render_markdown_with_cwd(
            "| Key | Description |\n| --- | --- |\n| -v | Enable very verbose logging output for debugging |\n| -q | Quiet output |\n",
            None,
        );
        let rendered = lines
            .iter()
            .flat_map(|line| {
                wrapped_table_line_texts(line, 64).unwrap_or_else(|| vec![line_text(line)])
            })
            .collect::<Vec<_>>();

        let wrapped_row_end = rendered
            .iter()
            .position(|line| line.contains("debugging"))
            .expect("wrapped row final line");
        let separator_indices = rendered
            .iter()
            .enumerate()
            .filter_map(|(index, line)| (line.contains('━') || line.contains('─')).then_some(index))
            .collect::<Vec<_>>();

        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Enable very verbose"))
                && rendered.iter().any(|line| line.contains("debugging")),
            "expected wrapped row content: {rendered:?}"
        );
        assert_eq!(separator_indices.len(), 2);
        assert!(separator_indices[1] > wrapped_row_end);
        assert!(
            rendered
                .last()
                .is_some_and(|line| line.contains("Quiet output"))
        );
    }

    #[test]
    fn preview_markdown_table_falls_back_to_records_when_width_is_tiny() {
        let lines = render_markdown_with_cwd(
            "| Key | Content | Extra | More |\n|---|---|---|---|\n| item | [link](https://example.com) | **bold** | `code` |\n",
            None,
        );
        let rendered = lines
            .iter()
            .flat_map(|line| {
                wrapped_table_line_texts(line, 16).unwrap_or_else(|| vec![line_text(line)])
            })
            .collect::<Vec<_>>();

        assert!(rendered.iter().any(|line| line.contains("Key")));
        assert!(rendered.iter().any(|line| line.contains("item")));
        assert!(rendered.iter().any(|line| line.contains("link")));
        assert!(rendered.iter().any(|line| line.contains("bold")));
        assert!(rendered.iter().any(|line| line.contains("code")));
        assert!(!rendered.iter().any(|line| line.contains('━')));
    }

    #[test]
    fn preview_markdown_table_falls_back_to_records_before_columns_feel_cramped() {
        let lines = render_markdown_with_cwd(
            "| Plain check | EARS |\n| --- | --- |\n| fallen reason is narrow | WHEN an item enters fallen, THE MODEL SHALL record a narrowly scoped removal reason instead of a broad lifecycle outcome term. |\n",
            None,
        );
        let rendered = lines
            .iter()
            .flat_map(|line| {
                wrapped_table_line_texts(line, 64).unwrap_or_else(|| vec![line_text(line)])
            })
            .collect::<Vec<_>>();

        assert!(rendered.iter().any(|line| line.contains("Plain check")));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("fallen reason is narrow"))
        );
        assert!(rendered.iter().any(|line| line.contains("EARS")));
        assert!(rendered.iter().any(|line| line.contains("WHEN an item")));
        assert!(!rendered.iter().any(|line| line.contains('━')));
    }

    #[test]
    fn preview_markdown_table_record_fallback_preserves_labels_and_values() {
        let lines = render_markdown_with_cwd(
            "| Key | Content | Extra | More |\n|---|---|---|---|\n| item | [link](https://example.com) | **bold** | `code` |\n",
            None,
        );
        let rendered = lines
            .iter()
            .flat_map(|line| {
                wrapped_table_line_texts(line, 16).unwrap_or_else(|| vec![line_text(line)])
            })
            .collect::<Vec<_>>();

        assert!(
            rendered
                .iter()
                .any(|line| line.trim_start().starts_with("Key") || line.contains("Key  item")),
            "expected record label for Key: {rendered:?}"
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Content") || line.contains("link")),
            "expected record label/value for Content: {rendered:?}"
        );
        assert!(rendered.iter().any(|line| line.contains("item")));
        assert!(rendered.iter().any(|line| line.contains("link")));
        assert!(rendered.iter().any(|line| line.contains("bold")));
        assert!(rendered.iter().any(|line| line.contains("code")));
    }

    #[test]
    fn fitted_table_widths_for_cells_shrinks_token_heavy_before_narrative() {
        let headers = vec![
            "URL".to_string(),
            "Description".to_string(),
            "OK".to_string(),
        ];
        let cells = vec![
            "https://example.com/a/very/long/path/that/can/wrap".to_string(),
            "This narrative column should keep enough width to remain readable".to_string(),
            "ok".to_string(),
        ];
        let widths = vec![52, 64, 2];

        let fitted = fitted_table_widths_for_cells(&headers, &cells, &widths, 82);

        assert_eq!(fitted.iter().sum::<usize>() + TABLE_COLUMN_GAP * 2, 82);
        assert!(
            fitted[0] < fitted[1],
            "token-heavy column should give width to narrative text: {fitted:?}"
        );
        assert_eq!(
            fitted[2], 2,
            "compact column should stay stable: {fitted:?}"
        );
    }

    #[test]
    fn fitted_table_widths_for_cells_uses_header_token_floor_for_compact_columns() {
        let headers = vec!["CurrentGate".to_string(), "N".to_string()];
        let cells = vec!["ok".to_string(), "1".to_string()];
        let widths = vec![20, 10];

        let fitted = fitted_table_widths_for_cells(&headers, &cells, &widths, 18);

        assert_eq!(fitted.iter().sum::<usize>() + TABLE_COLUMN_GAP, 18);
        assert!(
            fitted[0] >= "CurrentGate".len(),
            "compact column should keep header readable: {fitted:?}"
        );
    }

    #[test]
    fn preview_markdown_table_does_not_absorb_following_pipe_less_paragraph() {
        let text = render_snapshot_lines(
            "| Name | Meaning |\n| --- | --- |\n| alpha | first value |\ntrailing paragraph without pipes\n",
            None,
        );

        assert_eq!(
            text,
            vec![
                "Name     Meaning",
                "━━━━━    ━━━━━━━━━━━",
                "alpha    first value",
                "trailing paragraph without pipes",
            ]
        );
    }

    #[test]
    fn preview_markdown_table_keeps_explicit_single_cell_pipe_row() {
        let text = render_snapshot_lines(
            "| Only |\n| --- |\n| value |\n| explicit sparse row |\n",
            None,
        );

        assert_eq!(
            text,
            vec![
                "Only",
                "━━━━━━━━━━━━━━━━━━━",
                "value",
                "───────────────────",
                "explicit sparse row",
            ]
        );
    }

    #[test]
    fn preview_markdown_nested_list_keeps_parent_before_child() {
        let lines = marked_lines(vec![
            "- parent".to_string(),
            "  - child".to_string(),
            "  - [x] checked".to_string(),
            "- next".to_string(),
        ]);
        let text = lines.iter().map(line_text).collect::<Vec<_>>();

        assert_eq!(
            text,
            vec!["• parent", "  • child", "  • [x] checked", "• next"]
        );
    }

    #[test]
    fn preview_markdown_task_list_keeps_marker_checked_state_and_indent() {
        let text = render_snapshot_lines("- [ ] todo\n  - [x] nested done\n", None);

        assert_eq!(text, vec!["• [ ] todo", "  • [x] nested done"]);
    }

    #[test]
    fn preview_markdown_blockquote_list_then_nested_blockquote_keeps_indentation() {
        let text = render_snapshot_lines("> - parent\n>   > child\n", None);

        assert_eq!(text, vec!["> • parent", ">   > child"]);
    }

    #[test]
    fn preview_markdown_blockquote_inside_ordered_item_uses_item_marker_line() {
        let text = render_snapshot_lines("1.\n   > quoted\n", None);

        assert_eq!(text, vec!["1. > quoted"]);
    }

    #[test]
    fn preview_markdown_blockquote_after_ordered_item_aligns_with_content() {
        let text = render_snapshot_lines("1. before\n   > quoted\n", None);

        assert_eq!(text, vec!["1. before", "   > quoted"]);
    }

    #[test]
    fn preview_markdown_blockquote_with_multiline_code_block_keeps_quote_prefix() {
        let text = render_snapshot_lines("> ```xyzlang\n> first\n> second\n> ```\n", None);

        assert_eq!(text, vec!["> first", "> second"]);
    }

    #[test]
    fn preview_markdown_unordered_item_continuation_paragraph_is_indented() {
        let text = render_snapshot_lines(
            "- Intro\n\n  Continuation paragraph line 1\n  Continuation paragraph line 2\n",
            None,
        );

        assert_eq!(
            text,
            vec![
                "• Intro",
                "",
                "  Continuation paragraph line 1 Continuation paragraph line 2"
            ]
        );
    }

    #[test]
    fn preview_markdown_ordered_item_continuation_paragraph_is_indented() {
        let text = render_snapshot_lines("1. Intro\n\n   More details about intro\n", None);

        assert_eq!(text, vec!["1. Intro", "", "   More details about intro"]);
    }

    #[test]
    fn preview_markdown_nested_item_continuation_paragraph_is_indented() {
        let text = render_snapshot_lines("1. A\n    - B\n\n      Continuation for B\n2. C\n", None);

        assert_eq!(
            text,
            vec!["1. A", "  • B", "", "    Continuation for B", "2. C"]
        );
    }

    #[test]
    fn preview_markdown_blockquote_two_paragraphs_inside_ordered_item_has_blank_line() {
        let text = render_snapshot_lines("1.\n   > para 1\n   >\n   > para 2\n", None);

        assert_eq!(text, vec!["1. > para 1", "   > ", "   > para 2"]);
    }

    #[test]
    fn preview_markdown_list_item_blockquote_then_text_keeps_quote_context() {
        let text = render_snapshot_lines("1.\n   > quoted\n   after\n", None);

        assert_eq!(text, vec!["1. > quoted", "   > after"]);
    }

    #[test]
    fn preview_markdown_complex_snapshot_covers_codex_style_features() {
        let text = render_snapshot_lines(
            r#"# Review
Intro with **bold**, `code`, and [docs](https://example.com/docs).

> Quote [target](file:///Users/example/code/codex/codex-rs/README.md#L93)
> - nested item
> ```xyzlang
> plain code
> ```
> | Plain check | EARS |
> |---|---|
> | 좁은 이유 | WHEN item falls, THE MODEL SHALL record why. |

1. ordered
   1. child
"#,
            Some(Path::new("/Users/example/code/codex")),
        );

        assert_eq!(
            text,
            vec![
                "Review",
                "Intro with bold, code, and docs (https://example.com/docs).",
                "> Quote codex-rs/README.md:93",
                "> • nested item",
                "> plain code",
                "> Plain check    EARS",
                "> ━━━━━━━━━━━    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
                "> 좁은 이유      WHEN item falls, THE MODEL SHALL record why.",
                "1. ordered",
                "  1. child",
            ]
        );
    }

    #[test]
    fn preview_markdown_deep_snapshot_covers_nested_quote_list_code_table_and_links() {
        let text = render_snapshot_lines(
            r#"- [task](file:///Users/example/code/codex/README.md#L10)
  > quote starts
  > - nested [web](https://example.com/a)
  >   ```rust
  >   fn main() {}
  >   ```
  > | File | Meaning |
  > |---|---|
  > | [readme](file:///Users/example/code/codex/README.md#L10) | narrow reason |

  continuation paragraph
"#,
            Some(Path::new("/Users/example/code/codex")),
        );

        assert_eq!(
            text,
            vec![
                "• README.md:10",
                "  > quote starts",
                "  >   • nested web (https://example.com/a)",
                "  >     fn main() {}",
                "  > File            Meaning",
                "  > ━━━━━━━━━━━━    ━━━━━━━━━━━━━",
                "  > README.md:10    narrow reason",
                "",
                "  continuation paragraph",
            ]
        );
    }

    #[test]
    fn preview_build_leaf_work_includes_status_first_and_intent_when_files_exist() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/02-leaves/preview/00-status.md")
            .write_str(
                "# Leaf Status\n\n\
                 - stage: leaf\n\
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
        let item = inventory.stages[1]
            .items
            .iter()
            .find(|item| item.stage_dir == StageDir::Leaves && item.slug == "preview")
            .expect("item");

        let preview = build_from_source(&item.slug, &item.preview).expect("preview");
        let text = preview_text(&preview);
        let status_index = text
            .iter()
            .position(|line| line.contains("stage: leaf"))
            .expect("stage status line");
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
    fn preview_build_leaf_work_resolves_relative_links_from_source_file_directory() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/02-leaves/preview/00-status.md")
            .write_str("# Leaf Status\n\n- current gate: ① Intent\n")
            .expect("status");
        root.child(".leaf/02-leaves/preview/01-Learn/01-intent.md")
            .write_str("[guide](./notes/guide.md#L3C2-L4C8)\n")
            .expect("intent");

        let inventory = inventory::load(root.path()).expect("inventory");
        let item = inventory.stages[1]
            .items
            .iter()
            .find(|item| item.stage_dir == StageDir::Leaves && item.slug == "preview")
            .expect("item");
        let preview = build_from_source(&item.slug, &item.preview).expect("preview");
        let link = preview
            .lines
            .iter()
            .find_map(|line| match line {
                PreviewLine::Styled(spans) => spans.iter().find_map(|span| match span {
                    PreviewSpan::Link { text, target, .. } => Some((text, target)),
                    _ => None,
                }),
                _ => None,
            })
            .expect("relative source link");

        assert_eq!(link.0, "./notes/guide.md:3:2-4:8");
        assert_eq!(link.1, "./notes/guide.md#L3C2-L4C8");
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
        let item = inventory.stages[1]
            .items
            .iter()
            .find(|item| item.stage_dir == StageDir::Leaves && item.slug == "preview")
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

        let preview = build_pressed_digest(
            "research",
            &root.path().join(".leaf/04-pressed/research.md"),
        );
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

    #[test]
    fn preview_build_pressed_digest_resolves_relative_links_from_digest_directory() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/04-pressed/research.md")
            .write_str(
                "# Research Memo\n\n\
                 ## Citation Summary\n\n\
                 See [source](../leaves/research/00-status.md#L2).\n",
            )
            .expect("digest");

        let preview = build_pressed_digest(
            "research",
            &root.path().join(".leaf/04-pressed/research.md"),
        );
        let link = preview
            .lines
            .iter()
            .find_map(|line| match line {
                PreviewLine::Styled(spans) => spans.iter().find_map(|span| match span {
                    PreviewSpan::Link { text, target, .. } => Some((text, target)),
                    _ => None,
                }),
                _ => None,
            })
            .expect("digest relative link");

        assert_eq!(link.0, "../leaves/research/00-status.md:2");
        assert_eq!(link.1, "../leaves/research/00-status.md#L2");
    }
}
