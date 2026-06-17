use crate::inventory::{ParseState, StageDir};
use crate::list_columns::{ColumnWidth, LIST_COLUMNS, ListColumn};
use crate::preview::{PreviewColor, PreviewLine, PreviewSpan, PreviewStyle};
use crate::review::{ReviewDocument, ReviewLine};
use crate::tui::app::{AppState, ListRow, Mode, ReviewState, StageFilter};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table};
use std::ops::Range;
use std::sync::OnceLock;
use url::Url;

const PREVIEW_MIN_HEIGHT: u16 = 14;
const LIST_HEADER_HEIGHT: u16 = 2;
const HEADER_SPLIT_MIN_WIDTH: u16 = 60;
const HEADER_SUMMARY_WIDTH: u16 = 24;
const RIGHT_PREVIEW_RATIO: f32 = 0.45;
const BOTTOM_PREVIEW_RATIO: f32 = 0.40;
const MIN_TABLE_WIDTH_FOR_RIGHT_PREVIEW: u16 = 80;
const REVIEW_BODY_HORIZONTAL_PADDING: u16 = 1;
const MIN_REVIEW_BODY_WIDTH_FOR_HORIZONTAL_PADDING: u16 = 48;

static MARKDOWN_STYLE_PROFILE: OnceLock<MarkdownStyleProfile> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewPlacement {
    Hidden,
    Right,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
struct ListChunks {
    header: Rect,
    notice: Rect,
    body: Rect,
    status: Rect,
}

pub(crate) fn draw(frame: &mut Frame<'_>, app: &AppState) {
    let area = frame.area();
    if app.mode() == Mode::Review {
        draw_review(frame, area, app);
        return;
    }
    if app.mode() == Mode::ReferenceRead {
        draw_reference_read(frame, area, app);
        return;
    }
    if app.mode() == Mode::ReferencePicker {
        draw_review(frame, area, app);
        draw_reference_modal(frame, area, app);
        return;
    }

    let chunks = list_chunks(area);
    draw_header(frame, chunks.header, app);
    draw_notice(frame, chunks.notice, app);

    match preview_placement(area, app) {
        PreviewPlacement::Hidden => draw_table(frame, chunks.body, app),
        PreviewPlacement::Bottom => {
            let body_chunks = Layout::vertical([
                Constraint::Min(1),
                Constraint::Length(bottom_preview_height(chunks.body)),
            ])
            .split(chunks.body);
            draw_table(frame, body_chunks[0], app);
            draw_preview(frame, body_chunks[1], app);
        }
        PreviewPlacement::Right => {
            let preview_width = right_preview_width(area);
            let body_chunks = Layout::horizontal([
                Constraint::Min(MIN_TABLE_WIDTH_FOR_RIGHT_PREVIEW),
                Constraint::Length(preview_width),
            ])
            .split(chunks.body);
            draw_table(frame, body_chunks[0], app);
            draw_preview(frame, body_chunks[1], app);
        }
    }
    draw_status(frame, chunks.status, app);

    if app.mode() == Mode::FallInput {
        draw_fall_modal(frame, area, app);
    }
}

fn draw_fall_modal(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let targets = app.fall_targets();
    let count = targets.len();
    let preview = if targets.len() > 3 {
        format!("{}, …", targets[..3].join(", "))
    } else {
        targets.join(", ")
    };

    let lines = vec![
        Line::styled(
            format!("Fall {count} {}", if count == 1 { "item" } else { "items" }),
            strong_style(),
        ),
        Line::styled(preview, dim_style()),
        Line::raw(""),
        Line::raw(format!("reason: {}", app.fall_reason())),
        Line::raw(""),
        Line::styled("Enter confirm   ·   Esc cancel", dim_style()),
    ];

    let modal = centered_rect(area, 60, lines.len() as u16 + 2);
    frame.render_widget(Clear, modal);
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Fall")
                .border_style(strong_style().fg(Color::Yellow)),
        ),
        modal,
    );
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

fn list_chunks(area: Rect) -> ListChunks {
    let chunks = Layout::vertical([
        Constraint::Length(LIST_HEADER_HEIGHT),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);

    ListChunks {
        header: chunks[0],
        notice: chunks[1],
        body: chunks[2],
        status: chunks[3],
    }
}

fn draw_review(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let Some(review) = app.review_state() else {
        frame.render_widget(Paragraph::new("No review document loaded."), area);
        return;
    };
    draw_review_state(
        frame,
        area,
        app,
        review,
        "↑/↓ scroll  d/u half  PgUp/PgDn  g/G top/bottom  r refresh  R references  drag select/copy text  Esc/q back",
    );
}

/// Render a `ReviewState` (the canonical review, or a single reference file in
/// read mode) into the standard header/notice/body/footer layout. The body
/// width/height it measures drives the matching scroll clamp in `AppState`.
fn draw_review_state(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &AppState,
    review: &ReviewState,
    footer: &str,
) {
    let layout = review_layout(area);
    app.set_review_body_size(layout.body.height as usize, layout.body.width as usize);

    let document = &review.document;
    let header = Line::from(vec![
        Span::styled("leaf review", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(document.title.clone(), strong_style()),
        Span::raw("  "),
        Span::styled(document.root_relative_path.clone(), dim_style()),
    ]);
    frame.render_widget(Paragraph::new(header), layout.header);

    let notice = if review.status_message.is_empty() {
        "READ ONLY - edit originals".to_string()
    } else {
        format!("READ ONLY - edit originals  {}", review.status_message)
    };
    frame.render_widget(
        Paragraph::new(Line::styled(notice, strong_style().fg(Color::Yellow))),
        layout.notice,
    );

    let rendered_body_lines = review.rendered_body_lines(document, layout.body.width);
    let scroll_offset = clamped_review_scroll_offset(
        &rendered_body_lines,
        layout.body.height,
        review.scroll_offset,
    );

    frame.render_widget(review_body_block(), layout.body_shell);

    let body_lines =
        visible_review_body_lines(&rendered_body_lines, scroll_offset, layout.body.height)
            .into_iter()
            .map(|line| line.content.line.clone())
            .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(body_lines), layout.body);

    frame.render_widget(
        Paragraph::new(Line::styled(footer, dim_style())),
        layout.footer,
    );
}

/// Full-screen read of one reference file, reusing the review body renderer.
fn draw_reference_read(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let Some(read) = app.reference_read() else {
        frame.render_widget(Paragraph::new("No reference loaded."), area);
        return;
    };
    draw_review_state(
        frame,
        area,
        app,
        read,
        "↑/↓ scroll  d/u half  PgUp/PgDn  g/G top/bottom  Esc back to references",
    );
}

/// Centered modal listing the current leaf's references, drawn over the review.
fn draw_reference_modal(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let Some(picker) = app.reference_picker() else {
        return;
    };

    let width = (area.width.saturating_mul(60) / 100).clamp(20, area.width);
    let height = (area.height.saturating_mul(60) / 100).clamp(3, area.height);
    let modal = centered_rect(area, width, height);
    frame.render_widget(Clear, modal);

    let title = if picker.search_active() {
        format!(" references — /{} ", picker.query())
    } else {
        format!(" references ({}) ", picker.total())
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(dim_style());
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let rows = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(inner);

    if picker.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::styled("no references", dim_style())),
            rows[0],
        );
    } else {
        let filtered = picker.filtered();
        let selected = picker.selected();
        let max_name = rows[0].width.saturating_sub(2) as usize;
        let lines: Vec<Line> = filtered
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let chosen = selected == Some(index);
                let marker = if chosen { "> " } else { "  " };
                let name = truncate_display(&entry.name, max_name);
                let style = if chosen {
                    strong_style()
                } else {
                    Style::default()
                };
                Line::styled(format!("{marker}{name}"), style)
            })
            .collect();
        frame.render_widget(Paragraph::new(lines), rows[0]);
    }

    let hint = if picker.is_empty() {
        "Esc close".to_string()
    } else if picker.search_active() {
        format!(
            "{}/{} matched  Enter open  Esc cancel search",
            picker.filtered_count(),
            picker.total()
        )
    } else {
        "j/k move  / search  Enter open  Esc close".to_string()
    };
    frame.render_widget(Paragraph::new(Line::styled(hint, dim_style())), rows[1]);
}

/// Truncate a display string to `max` columns, appending an ellipsis.
fn truncate_display(text: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if text.chars().count() <= max {
        return text.to_string();
    }
    let keep = max.saturating_sub(1);
    let truncated: String = text.chars().take(keep).collect();
    format!("{truncated}…")
}

#[derive(Debug, Clone, Copy)]
struct ReviewLayout {
    header: Rect,
    notice: Rect,
    body_shell: Rect,
    body: Rect,
    footer: Rect,
}

fn review_layout(area: Rect) -> ReviewLayout {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);
    let body_shell = chunks[2];
    ReviewLayout {
        header: chunks[0],
        notice: chunks[1],
        body_shell,
        body: review_body_inner(body_shell),
        footer: chunks[3],
    }
}

fn review_body_inner(area: Rect) -> Rect {
    let inner = review_body_block().inner(area);
    padded_content_inner(inner)
}

fn padded_content_inner(inner: Rect) -> Rect {
    if inner.width < MIN_REVIEW_BODY_WIDTH_FOR_HORIZONTAL_PADDING {
        return inner;
    }
    Rect {
        x: inner.x.saturating_add(REVIEW_BODY_HORIZONTAL_PADDING),
        width: inner
            .width
            .saturating_sub(REVIEW_BODY_HORIZONTAL_PADDING.saturating_mul(2)),
        ..inner
    }
}

#[cfg(test)]
fn rect_contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

#[cfg(test)]
pub(crate) fn review_hyperlink_target(
    area: Rect,
    app: &AppState,
    column: u16,
    row: u16,
) -> Option<String> {
    if app.mode() != Mode::Review {
        return None;
    }
    let review = app.review_state()?;
    let layout = review_layout(area);
    if !rect_contains(layout.body, column, row) {
        return None;
    }

    let rendered = review.rendered_body_lines(&review.document, layout.body.width);
    let scroll_offset =
        clamped_review_scroll_offset(&rendered, layout.body.height, review.scroll_offset);
    let relative_row = usize::from(row.saturating_sub(layout.body.y));
    let relative_column = usize::from(column.saturating_sub(layout.body.x));
    let line = visible_review_body_lines(&rendered, scroll_offset + relative_row, 1)
        .into_iter()
        .next()?;

    line.content
        .hyperlinks
        .iter()
        .find(|link| link.columns.contains(&relative_column))
        .and_then(|link| safe_hyperlink_destination(&link.destination))
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let visible_count = app.visible_rows().len();
    let total_count = app.row_count();
    let filter = if app.filter().is_empty() {
        "none".to_string()
    } else {
        app.filter().to_string()
    };
    let header_rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);
    draw_header_row(
        frame,
        header_rows[0],
        vec![
            Span::styled("Inventory", strong_style()),
            Span::raw("  "),
            Span::styled("leaf list", dim_style()),
        ],
        Some(vec![
            Span::styled(stage_filter_label(app.active_stage()), chrome_style()),
            Span::raw(format!(" {visible_count}/{total_count}")),
        ]),
    );

    let selected_count = app.selected_row_count();
    let selected = (selected_count > 0).then(|| {
        vec![Span::styled(
            format!("selected {selected_count}"),
            strong_style(),
        )]
    });
    draw_header_row(
        frame,
        header_rows[1],
        vec![
            Span::styled("filter ", chrome_style()),
            Span::styled(filter, dim_style()),
        ],
        selected,
    );
}

fn draw_header_row(
    frame: &mut Frame<'_>,
    area: Rect,
    left: Vec<Span<'static>>,
    right: Option<Vec<Span<'static>>>,
) {
    if area.height == 0 {
        return;
    }
    let Some(right) = right else {
        frame.render_widget(Paragraph::new(Line::from(left)), area);
        return;
    };

    if area.width < HEADER_SPLIT_MIN_WIDTH {
        let mut spans = left;
        spans.push(Span::raw("  "));
        spans.extend(right);
        frame.render_widget(Paragraph::new(Line::from(spans)), area);
        return;
    }

    let summary_width = HEADER_SUMMARY_WIDTH.min(area.width);
    let chunks =
        Layout::horizontal([Constraint::Min(1), Constraint::Length(summary_width)]).split(area);
    frame.render_widget(Paragraph::new(Line::from(left)), chunks[0]);
    frame.render_widget(
        Paragraph::new(Line::from(right)).alignment(Alignment::Right),
        chunks[1],
    );
}

fn draw_table(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    if area.height == 0 {
        return;
    }

    let block = chrome_block().title("Inventory");
    frame.render_widget(block.clone(), area);
    let content_area = padded_content_inner(block.inner(area));

    let visible_rows = app.visible_rows();
    if visible_rows.is_empty() {
        let empty = Paragraph::new("No leaf items match the current view.");
        let empty_area = if content_area.height == 0 {
            area
        } else {
            content_area
        };
        frame.render_widget(empty, empty_area);
        return;
    }

    let row_capacity = table_row_capacity(content_area);
    let offset = row_viewport_offset(app.selected_index(), row_capacity);
    let rows = visible_rows
        .into_iter()
        .enumerate()
        .skip(offset)
        .take(row_capacity)
        .map(|(index, row)| table_row(row, row_is_active(app, index)).style(row_style(app, index)));

    let table = Table::new(rows, table_constraints())
        .header(Row::new(table_header()).style(chrome_style()))
        .column_spacing(1);
    frame.render_widget(table, content_area);
}

fn draw_preview(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let (title, lines) = match app.selected_row() {
        Some(row) => {
            let preview = app
                .selected_preview()
                .unwrap_or_else(|| crate::preview::Preview {
                    title: row.slug().to_string(),
                    lines: vec![PreviewLine::Plain("No preview available.".to_string())],
                });
            let mut lines = vec![
                Line::from(vec![
                    Span::styled(preview.title.clone(), strong_style()),
                    Span::raw("  "),
                    Span::styled(row.relative_path().to_string(), dim_style()),
                ]),
                Line::from(""),
            ];
            lines.extend(preview_lines_with_rhythm(&preview.lines));
            (format!("Preview {}", row.slug()), lines)
        }
        None => (
            "Preview".to_string(),
            vec![Line::from("No leaf item selected.")],
        ),
    };

    let block = chrome_block().title(title);
    frame.render_widget(block.clone(), area);
    frame.render_widget(
        Paragraph::new(lines),
        padded_content_inner(block.inner(area)),
    );
}

fn draw_notice(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    if app.notice().is_empty() {
        return;
    }

    frame.render_widget(
        Paragraph::new(Line::styled(
            app.notice().to_string(),
            strong_style().fg(Color::Yellow),
        )),
        area,
    );
}

fn draw_status(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let selected_count = app.selected_row_count();
    let status = match app.mode() {
        Mode::FilterInput => format!(
            "filter: {}  Esc list  Backspace delete  {}",
            app.filter(),
            app.status_line()
        ),
        Mode::RangeSelect => format!(
            "range {selected_count} selected  j/k extend  v/Esc done  y copy  q quit  {}",
            app.status_line()
        ),
        Mode::FallInput => format!(
            "fall reason: {}  Enter confirm  Esc cancel  Backspace delete  {}",
            app.fall_reason(),
            app.status_line()
        ),
        Mode::Review => {
            "↑/↓ scroll  d/u half  PgUp/PgDn  g/G top/bottom  r refresh  R references  drag select/copy text  Esc/q back"
                .to_string()
        }
        Mode::ReferencePicker => {
            "references  j/k move  / search  Enter open  Esc close".to_string()
        }
        Mode::ReferenceRead => {
            "↑/↓ scroll  d/u half  g/G top/bottom  Esc back to references".to_string()
        }
        Mode::List if selected_count > 0 => format!(
            "{selected_count} selected  Space toggle  v range  a all  y copy  F fall  Esc clear  q quit  {}",
            app.status_line()
        ),
        Mode::List => format!(
            "j/k up/down  h/l stage  y copy  F fall  Space select  v range  a all  / filter  p preview  r refresh  q quit  mouse drag  {}",
            app.status_line()
        ),
    };
    frame.render_widget(Paragraph::new(Line::styled(status, dim_style())), area);
}

fn table_row(row: &ListRow, active: bool) -> Row<'_> {
    Row::new(
        LIST_COLUMNS
            .iter()
            .copied()
            .map(|column| table_cell(column, row, active))
            .collect::<Vec<_>>(),
    )
}

fn table_header() -> Vec<Cell<'static>> {
    LIST_COLUMNS
        .iter()
        .map(|column| Cell::from(column.header()))
        .collect()
}

fn table_constraints() -> Vec<Constraint> {
    LIST_COLUMNS
        .iter()
        .map(|column| match column.tui_width() {
            ColumnWidth::Fixed(width) => Constraint::Length(width),
            ColumnWidth::Min(width) => Constraint::Min(width),
        })
        .collect()
}

fn table_cell(column: ListColumn, row: &ListRow, active: bool) -> Cell<'static> {
    let cell = Cell::from(column.value(row));
    if active {
        return cell;
    }
    match column {
        ListColumn::Stage => cell.style(stage_style(row.stage_dir())),
        ListColumn::Status => cell.style(parse_state_style(row.parse_state())),
        ListColumn::Phase | ListColumn::Gate | ListColumn::Slug => cell,
    }
}

fn row_is_active(app: &AppState, index: usize) -> bool {
    app.selected_index() == index || app.visible_row_is_marked(index)
}

/// Computes the table chunk for a full terminal `Rect`, mirroring the layout
/// `draw` uses so mouse hit-testing maps onto the same rows `draw_table` renders.
fn table_chunk(area: Rect, app: &AppState) -> Rect {
    let body = list_chunks(area).body;
    match preview_placement(area, app) {
        PreviewPlacement::Hidden => body,
        PreviewPlacement::Bottom => Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(bottom_preview_height(body)),
        ])
        .split(body)[0],
        PreviewPlacement::Right => {
            let preview_width = right_preview_width(area);
            Layout::new(
                Direction::Horizontal,
                [
                    Constraint::Min(MIN_TABLE_WIDTH_FOR_RIGHT_PREVIEW),
                    Constraint::Length(preview_width),
                ],
            )
            .split(body)[0]
        }
    }
}

/// Maps a terminal `(column, row)` click onto the visible row it covers.
///
/// Returns `Some(visible_index)` when the coordinate lands on a data row inside
/// the table, or `None` for the header, borders, status line, or any coordinate
/// outside the rendered rows. Data rows start at `table.y + 2` (top border +
/// header).
pub(crate) fn table_mouse_target(
    area: Rect,
    app: &AppState,
    column: u16,
    row: u16,
) -> Option<usize> {
    let table = table_chunk(area, app);
    if table.height == 0 || table.width < 2 {
        return None;
    }
    let table_content = padded_content_inner(chrome_block().inner(table));
    if table_content.height == 0 || table_content.width == 0 {
        return None;
    }

    if column < table_content.x || column >= table_content.x.saturating_add(table_content.width) {
        return None;
    }

    let first_data_row = table_content.y + 1;
    if row < first_data_row {
        return None;
    }
    let data_row_index = (row - first_data_row) as usize;
    let row_capacity = table_row_capacity(table_content);
    if data_row_index >= row_capacity {
        return None;
    }

    let offset = row_viewport_offset(app.selected_index(), row_capacity);
    let visible_index = offset + data_row_index;
    if visible_index >= app.visible_rows().len() {
        return None;
    }

    Some(visible_index)
}

fn table_row_capacity(area: Rect) -> usize {
    area.height.saturating_sub(1) as usize
}

fn row_viewport_offset(selected_index: usize, row_capacity: usize) -> usize {
    if row_capacity == 0 {
        selected_index
    } else {
        selected_index.saturating_sub(row_capacity - 1)
    }
}

fn row_style(app: &AppState, index: usize) -> Style {
    match (
        app.selected_index() == index,
        app.visible_row_is_marked(index),
    ) {
        (true, true) => Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD),
        (true, false) => Style::default().add_modifier(Modifier::REVERSED),
        (false, true) => Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
        (false, false) => Style::default(),
    }
}

fn preview_placement(area: Rect, app: &AppState) -> PreviewPlacement {
    if !app.preview_open() || area.height < PREVIEW_MIN_HEIGHT {
        return PreviewPlacement::Hidden;
    }
    let preview_width = right_preview_width(area);
    let table_width = area.width.saturating_sub(preview_width);
    if table_width < MIN_TABLE_WIDTH_FOR_RIGHT_PREVIEW {
        PreviewPlacement::Bottom
    } else {
        PreviewPlacement::Right
    }
}

fn right_preview_width(area: Rect) -> u16 {
    ((area.width as f32) * RIGHT_PREVIEW_RATIO).floor() as u16
}

fn bottom_preview_height(area: Rect) -> u16 {
    ((area.height as f32) * BOTTOM_PREVIEW_RATIO)
        .floor()
        .clamp(6.0, 18.0) as u16
}

fn preview_line(line: &PreviewLine) -> Line<'static> {
    preview_hyperlink_line(line).line
}

fn preview_lines_with_rhythm(lines: &[PreviewLine]) -> Vec<Line<'static>> {
    let mut rendered = Vec::new();
    let mut previous = None;
    for line in lines {
        let current = markdown_block_kind_for_preview(line);
        if should_insert_markdown_rhythm(previous, current) && !rendered_ends_with_blank(&rendered)
        {
            rendered.push(Line::default());
        }
        rendered.push(preview_line(line));
        previous = Some(current);
    }
    rendered
}

fn preview_hyperlink_line(line: &PreviewLine) -> HyperlinkLine {
    match line {
        PreviewLine::BlockQuote { prefix, line, .. } => {
            prepend_blockquote_prefix(prefix, preview_hyperlink_line(line))
        }
        PreviewLine::Heading { level, text } => HyperlinkLine::new(Line::from(Span::styled(
            heading_display_text(*level, text),
            heading_style(*level),
        ))),
        PreviewLine::Checkbox {
            marker,
            checked,
            text,
        } => {
            let checkbox = if *checked { "[x]" } else { "[ ]" };
            HyperlinkLine::new(Line::from(vec![
                Span::styled(marker.clone(), list_marker_style()),
                Span::raw(" "),
                Span::styled(checkbox, list_marker_style()),
                Span::raw(" "),
                Span::raw(text.clone()),
            ]))
        }
        PreviewLine::ListItem { marker, spans } => {
            let rendered = vec![
                Span::styled(marker.clone(), list_marker_style()),
                Span::raw(" "),
            ];
            let mut line = HyperlinkLine::new(Line::from(rendered));
            push_preview_spans(&mut line, spans);
            line
        }
        PreviewLine::Code(text) => HyperlinkLine::new(Line::styled(text.clone(), code_style())),
        PreviewLine::CodeSpans(spans) => {
            let mut line = HyperlinkLine::default();
            push_preview_spans(&mut line, spans);
            line
        }
        PreviewLine::Styled(spans) => {
            let mut line = HyperlinkLine::default();
            push_preview_spans(&mut line, spans);
            line
        }
        PreviewLine::SourceBoundary {
            phase,
            gate,
            source,
        } => HyperlinkLine::new(source_boundary_line(None, phase, gate, source)),
        PreviewLine::TableHeader { .. } => HyperlinkLine::new(Line::styled(
            crate::preview::table_line_text(line).expect("table line text"),
            table_header_style(),
        )),
        PreviewLine::TableDivider { .. } => HyperlinkLine::new(Line::styled(
            crate::preview::table_line_text(line).expect("table line text"),
            table_rule_style(),
        )),
        PreviewLine::TableRow { .. } => HyperlinkLine::new(Line::from(
            crate::preview::table_line_text(line).expect("table line text"),
        )),
        PreviewLine::Plain(text) => HyperlinkLine::new(Line::from(text.clone())),
    }
}

fn push_preview_spans(line: &mut HyperlinkLine, spans: &[PreviewSpan]) {
    for span in spans {
        let destination = match span {
            PreviewSpan::Link { target, .. } => Some(target.as_str()),
            _ => None,
        };
        line.push_span(preview_span(span), destination);
    }
}

fn preview_span(span: &PreviewSpan) -> Span<'static> {
    match span {
        PreviewSpan::Plain(text) => Span::raw(text.clone()),
        PreviewSpan::Bold(text) => Span::styled(text.clone(), strong_style()),
        PreviewSpan::StyledText { text, style } => {
            Span::styled(text.clone(), preview_text_style(*style))
        }
        PreviewSpan::Code(text) => Span::styled(text.clone(), code_style()),
        PreviewSpan::Link { text, .. } => Span::styled(text.clone(), link_style()),
        PreviewSpan::Syntax { text, style } => Span::styled(text.clone(), preview_style(*style)),
    }
}

fn preview_text_style(style: crate::preview::PreviewTextStyle) -> Style {
    let mut rendered = Style::default();
    if style.bold {
        rendered = rendered.add_modifier(Modifier::BOLD);
    }
    if style.italic {
        rendered = rendered.add_modifier(Modifier::ITALIC);
    }
    if style.underline {
        rendered = rendered.add_modifier(Modifier::UNDERLINED);
    }
    if style.strikethrough {
        rendered = rendered.add_modifier(Modifier::CROSSED_OUT);
    }
    rendered
}

fn preview_style(style: PreviewStyle) -> Style {
    let mut rendered = Style::default();
    if let Some(color) = style.fg {
        rendered = rendered.fg(preview_color(color));
    }
    if let Some(color) = style.bg {
        rendered = rendered.bg(preview_color(color));
    }
    if style.text_style.bold {
        rendered = rendered.add_modifier(Modifier::BOLD);
    }
    if style.text_style.italic {
        rendered = rendered.add_modifier(Modifier::ITALIC);
    }
    if style.text_style.underline {
        rendered = rendered.add_modifier(Modifier::UNDERLINED);
    }
    if style.text_style.strikethrough {
        rendered = rendered.add_modifier(Modifier::CROSSED_OUT);
    }
    rendered
}

#[allow(clippy::disallowed_methods)]
fn preview_color(color: PreviewColor) -> Color {
    match color {
        PreviewColor::Rgb(red, green, blue) => Color::Rgb(red, green, blue),
        PreviewColor::Ansi(index) => match index {
            0x00 => Color::Black,
            0x01 => Color::Red,
            0x02 => Color::Green,
            0x03 => Color::Yellow,
            0x04 => Color::Blue,
            0x05 => Color::Magenta,
            0x06 => Color::Cyan,
            0x07 => Color::Gray,
            index => Color::Indexed(index),
        },
    }
}

#[cfg(test)]
fn review_line(line: &ReviewLine) -> Line<'static> {
    review_hyperlink_line(line).line
}

fn review_hyperlink_line(line: &ReviewLine) -> HyperlinkLine {
    match line {
        ReviewLine::Separator {
            relative_path,
            phase,
            gate,
        } => HyperlinkLine::new(source_boundary_line(
            Some("FILE"),
            phase,
            gate,
            relative_path,
        )),
        ReviewLine::MissingSource { relative_path } => HyperlinkLine::new(Line::from(vec![
            Span::styled("MISSING SOURCE", Style::default().fg(Color::Red)),
            Span::raw(" "),
            Span::styled(relative_path.clone(), dim_style()),
        ])),
        ReviewLine::Markdown(line) => preview_hyperlink_line(line),
        ReviewLine::Message(text) => HyperlinkLine::new(Line::from(text.clone())),
    }
}

const USER_REVIEW_MARKER: &str = "USER REVIEW NEEDED";

fn highlight_user_review_markers(line: HyperlinkLine) -> HyperlinkLine {
    let text = line_text(&line.line);
    let ranges = text
        .match_indices(USER_REVIEW_MARKER)
        .map(|(start, marker)| {
            let mut end = start + marker.len();
            if text.as_bytes().get(end) == Some(&b':') {
                end += 1;
            }
            start..end
        })
        .collect::<Vec<_>>();
    if ranges.is_empty() {
        return line;
    }

    let mut highlighted_spans = Vec::new();
    let mut span_start = 0;
    for span in line.line.spans {
        let content = span.content.into_owned();
        let span_end = span_start + content.len();
        let mut offset = 0;
        while offset < content.len() {
            let absolute = span_start + offset;
            let range = ranges
                .iter()
                .find(|range| range.start <= absolute && absolute < range.end);
            let next_boundary = ranges
                .iter()
                .filter_map(|range| {
                    if range.start > absolute && range.start < span_end {
                        Some(range.start)
                    } else if range.end > absolute && range.end < span_end {
                        Some(range.end)
                    } else {
                        None
                    }
                })
                .min()
                .unwrap_or(span_end);
            let relative_end = next_boundary - span_start;
            let mut style = span.style;
            if range.is_some() {
                style = style.patch(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
            }
            highlighted_spans.push(Span::styled(
                content[offset..relative_end].to_string(),
                style,
            ));
            offset = relative_end;
        }
        span_start = span_end;
    }

    HyperlinkLine {
        line: Line {
            style: line.line.style,
            alignment: line.line.alignment,
            spans: highlighted_spans,
        },
        hyperlinks: line.hyperlinks,
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RenderedReviewLine {
    content: HyperlinkLine,
}

#[derive(Debug, Clone)]
pub(crate) struct ReviewRenderCache {
    width: u16,
    lines: Vec<RenderedReviewLine>,
}

impl ReviewRenderCache {
    pub(crate) fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub(crate) fn lines(&self) -> &[RenderedReviewLine] {
        &self.lines
    }
}

pub(crate) fn build_review_render_cache(
    document: &ReviewDocument,
    width: usize,
) -> ReviewRenderCache {
    let width = width.try_into().unwrap_or(u16::MAX);
    ReviewRenderCache {
        width,
        lines: review_body_lines(document, width),
    }
}

pub(crate) fn review_render_cache_matches(cache: &ReviewRenderCache, width: usize) -> bool {
    cache.width == width.try_into().unwrap_or(u16::MAX)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TerminalHyperlink {
    columns: Range<usize>,
    destination: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct HyperlinkLine {
    line: Line<'static>,
    hyperlinks: Vec<TerminalHyperlink>,
}

impl HyperlinkLine {
    fn new(line: Line<'static>) -> Self {
        Self {
            line,
            hyperlinks: Vec::new(),
        }
    }

    fn width(&self) -> usize {
        line_width(&self.line)
    }

    fn push_span(&mut self, span: Span<'static>, destination: Option<&str>) {
        let start = self.width();
        let end = start + span_width(&span);
        self.line.push_span(span);
        if end > start
            && let Some(destination) = destination
        {
            push_hyperlink_range(&mut self.hyperlinks, start..end, destination);
        }
    }
}

#[allow(dead_code)]
fn terminal_hyperlink_text(line: &HyperlinkLine) -> String {
    let text = line_text(&line.line);
    if line.hyperlinks.is_empty() {
        return text;
    }

    let mut rendered = String::new();
    let mut column = 0;
    let mut active_destination: Option<String> = None;
    for ch in text.chars() {
        let width = crate::review::terminal_char_width(ch);
        let destination = line
            .hyperlinks
            .iter()
            .find(|link| link.columns.contains(&column))
            .and_then(|link| safe_hyperlink_destination(&link.destination));
        if active_destination != destination {
            if active_destination.is_some() {
                rendered.push_str("\x1b]8;;\x07");
            }
            if let Some(destination) = &destination {
                rendered.push_str(&format!("\x1b]8;;{destination}\x07"));
            }
            active_destination = destination;
        }
        rendered.push(ch);
        column += width;
    }
    if active_destination.is_some() {
        rendered.push_str("\x1b]8;;\x07");
    }
    rendered
}

fn wrapped_review_lines(document: &ReviewDocument, width: u16) -> Vec<RenderedReviewLine> {
    let width = usize::from(width.max(1));
    let mut rendered: Vec<RenderedReviewLine> = Vec::new();
    let mut previous_kind = None;

    for line in &document.lines {
        let current = markdown_block_kind_for_review(line);
        if should_insert_markdown_rhythm(previous_kind, current)
            && rendered.last().is_none_or(|line| line.content.width() != 0)
        {
            rendered.push(RenderedReviewLine {
                content: HyperlinkLine::default(),
            });
        }

        if let Some(table_lines) = review_table_lines(line, width) {
            rendered.extend(
                table_lines
                    .into_iter()
                    .map(|content| RenderedReviewLine { content }),
            );
        } else if let Some(blockquote_lines) = review_blockquote_lines(line, width) {
            rendered.extend(
                blockquote_lines
                    .into_iter()
                    .map(|content| RenderedReviewLine { content }),
            );
        } else if let Some(list_lines) = review_list_lines(line, width) {
            rendered.extend(
                list_lines
                    .into_iter()
                    .map(|content| RenderedReviewLine { content }),
            );
        } else {
            let content = review_hyperlink_line(line);
            let content = if matches!(
                line,
                ReviewLine::Markdown(PreviewLine::Code(_) | PreviewLine::CodeSpans(_))
            ) {
                content
            } else {
                highlight_user_review_markers(content)
            };
            rendered.extend(
                wrap_hyperlink_line(content, width)
                    .into_iter()
                    .map(|content| RenderedReviewLine { content }),
            );
        }
        previous_kind = Some(current);
    }

    rendered
}

fn review_body_lines(document: &ReviewDocument, width: u16) -> Vec<RenderedReviewLine> {
    let mut rendered = wrapped_review_lines(document, width);
    if rendered.is_empty() {
        return rendered;
    }
    if rendered.last().is_none_or(|line| line.content.width() != 0) {
        rendered.push(RenderedReviewLine {
            content: HyperlinkLine::default(),
        });
    }
    rendered.push(RenderedReviewLine {
        content: review_end_marker_line(usize::from(width.max(1))),
    });
    rendered
}

fn review_end_marker_line(width: usize) -> HyperlinkLine {
    let marker = "-- END --";
    let padding = width.saturating_sub(text_width(marker)) / 2;
    HyperlinkLine::new(Line::from(vec![
        Span::raw(" ".repeat(padding)),
        Span::styled(marker, dim_style()),
    ]))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkdownBlockKind {
    Blank,
    Boundary,
    Source,
    Heading1,
    Heading,
    Code,
    Table,
    List,
    Quote,
    Paragraph,
}

fn markdown_block_kind_for_review(line: &ReviewLine) -> MarkdownBlockKind {
    match line {
        ReviewLine::Separator { .. } => MarkdownBlockKind::Boundary,
        ReviewLine::MissingSource { .. } => MarkdownBlockKind::Paragraph,
        ReviewLine::Markdown(line) => markdown_block_kind_for_preview(line),
        ReviewLine::Message(text) if text.is_empty() => MarkdownBlockKind::Blank,
        ReviewLine::Message(_) => MarkdownBlockKind::Paragraph,
    }
}

fn markdown_block_kind_for_preview(line: &PreviewLine) -> MarkdownBlockKind {
    match line {
        PreviewLine::BlockQuote { line, .. } => match markdown_block_kind_for_preview(line) {
            MarkdownBlockKind::Code | MarkdownBlockKind::Table => {
                markdown_block_kind_for_preview(line)
            }
            _ => MarkdownBlockKind::Quote,
        },
        PreviewLine::Heading { level: 1, .. } => MarkdownBlockKind::Heading1,
        PreviewLine::Heading { .. } => MarkdownBlockKind::Heading,
        PreviewLine::Code(_) | PreviewLine::CodeSpans(_) => MarkdownBlockKind::Code,
        PreviewLine::TableHeader { .. }
        | PreviewLine::TableDivider { .. }
        | PreviewLine::TableRow { .. } => MarkdownBlockKind::Table,
        PreviewLine::Checkbox { .. } | PreviewLine::ListItem { .. } => MarkdownBlockKind::List,
        PreviewLine::SourceBoundary { .. } => MarkdownBlockKind::Source,
        PreviewLine::Plain(text) if text.is_empty() => MarkdownBlockKind::Blank,
        PreviewLine::Plain(_) | PreviewLine::Styled(_) => MarkdownBlockKind::Paragraph,
    }
}

fn should_insert_markdown_rhythm(
    previous: Option<MarkdownBlockKind>,
    current: MarkdownBlockKind,
) -> bool {
    let Some(previous) = previous else {
        return false;
    };
    if previous == MarkdownBlockKind::Blank
        || current == MarkdownBlockKind::Blank
        || current == MarkdownBlockKind::Boundary
    {
        return false;
    }
    if previous == MarkdownBlockKind::Boundary && current != MarkdownBlockKind::Heading1 {
        return false;
    }
    match (previous, current) {
        (
            _,
            MarkdownBlockKind::Source | MarkdownBlockKind::Heading1 | MarkdownBlockKind::Heading,
        ) => true,
        (
            MarkdownBlockKind::Source | MarkdownBlockKind::Heading1 | MarkdownBlockKind::Heading,
            _,
        ) => true,
        (_, MarkdownBlockKind::Quote) if previous != current => true,
        (MarkdownBlockKind::Quote, _) if previous != current => true,
        (_, MarkdownBlockKind::Code | MarkdownBlockKind::Table)
            if previous != current && previous != MarkdownBlockKind::Quote =>
        {
            true
        }
        (MarkdownBlockKind::Code | MarkdownBlockKind::Table, _) if previous != current => true,
        _ => false,
    }
}

fn rendered_ends_with_blank(lines: &[Line<'_>]) -> bool {
    lines.last().is_some_and(|line| line_width(line) == 0)
}

fn visible_review_body_lines(
    lines: &[RenderedReviewLine],
    scroll_offset: usize,
    height: u16,
) -> Vec<RenderedReviewLine> {
    lines
        .iter()
        .skip(scroll_offset)
        .take(height as usize)
        .cloned()
        .collect()
}

fn review_table_lines(line: &ReviewLine, width: usize) -> Option<Vec<HyperlinkLine>> {
    let ReviewLine::Markdown(line) = line else {
        return None;
    };
    let (prefix, table_line) = blockquote_context(line);
    let prefix_width = blockquote_display_prefix_width(prefix);
    let table_width = width.saturating_sub(prefix_width).max(1);
    let table_lines = crate::preview::wrapped_table_line_texts(table_line, table_width)?;
    let style = match table_line {
        PreviewLine::TableHeader { .. } => table_header_style(),
        PreviewLine::TableDivider { .. } => table_rule_style(),
        PreviewLine::TableRow { .. } => Style::default(),
        _ => return None,
    };
    Some(
        annotate_table_link_fragments(table_line, table_lines, style)
            .into_iter()
            .map(highlight_user_review_markers)
            .map(|line| prepend_blockquote_prefix(prefix, line))
            .collect(),
    )
}

fn review_blockquote_lines(line: &ReviewLine, width: usize) -> Option<Vec<HyperlinkLine>> {
    let ReviewLine::Markdown(PreviewLine::BlockQuote { prefix, line, .. }) = line else {
        return None;
    };
    let prefix_width = blockquote_display_prefix_width(prefix);
    let inner_width = width.saturating_sub(prefix_width).max(1);
    Some(
        wrap_preview_line(line, inner_width)
            .into_iter()
            .map(|line| prepend_blockquote_prefix(prefix, line))
            .collect(),
    )
}

fn review_list_lines(line: &ReviewLine, width: usize) -> Option<Vec<HyperlinkLine>> {
    let ReviewLine::Markdown(line) = line else {
        return None;
    };
    preview_list_lines(line, width)
}

// Review-only caller; preview panel rendering goes through preview_line/preview_lines_with_rhythm.
fn wrap_preview_line(line: &PreviewLine, width: usize) -> Vec<HyperlinkLine> {
    preview_list_lines(line, width).unwrap_or_else(|| {
        let content = preview_hyperlink_line(line);
        let content = if matches!(line, PreviewLine::Code(_) | PreviewLine::CodeSpans(_)) {
            content
        } else {
            highlight_user_review_markers(content)
        };
        wrap_hyperlink_line(content, width)
    })
}

// Review-only caller; preview panel rendering goes through preview_line/preview_lines_with_rhythm.
fn preview_list_lines(line: &PreviewLine, width: usize) -> Option<Vec<HyperlinkLine>> {
    let continuation_width = match line {
        PreviewLine::ListItem { marker, .. } => text_width(marker) + 1,
        PreviewLine::Checkbox { marker, .. } => text_width(marker) + 1 + "[x]".len() + 1,
        _ => return None,
    };
    Some(wrap_hyperlink_line_with_continuation(
        highlight_user_review_markers(preview_hyperlink_line(line)),
        width,
        continuation_width,
    ))
}

fn blockquote_context(line: &PreviewLine) -> (&str, &PreviewLine) {
    match line {
        PreviewLine::BlockQuote { prefix, line, .. } => (prefix.as_str(), line),
        _ => ("", line),
    }
}

fn prepend_blockquote_prefix(prefix: &str, mut line: HyperlinkLine) -> HyperlinkLine {
    if prefix.is_empty() {
        return line;
    }
    let display_prefix = blockquote_display_prefix(prefix);
    let shift = text_width(&display_prefix);
    let mut spans = vec![Span::styled(display_prefix, quote_style())];
    spans.extend(line.line.spans);
    line.line = Line::from(spans);
    for hyperlink in &mut line.hyperlinks {
        hyperlink.columns = hyperlink.columns.start + shift..hyperlink.columns.end + shift;
    }
    line
}

fn blockquote_display_prefix_width(prefix: &str) -> usize {
    text_width(&blockquote_display_prefix(prefix))
}

fn blockquote_display_prefix(prefix: &str) -> String {
    prefix
        .chars()
        .map(|ch| if ch == '>' { '│' } else { ch })
        .collect()
}

fn wrap_hyperlink_line(line: HyperlinkLine, width: usize) -> Vec<HyperlinkLine> {
    wrap_hyperlink_line_with_prefix(line, width, None)
}

fn wrap_hyperlink_line_with_continuation(
    line: HyperlinkLine,
    width: usize,
    continuation_width: usize,
) -> Vec<HyperlinkLine> {
    let continuation_prefix = (continuation_width > 0)
        .then(|| Span::styled(" ".repeat(continuation_width), Style::default()));
    wrap_hyperlink_line_with_prefix(line, width, continuation_prefix)
}

fn wrap_hyperlink_line_with_prefix(
    line: HyperlinkLine,
    width: usize,
    continuation_prefix: Option<Span<'static>>,
) -> Vec<HyperlinkLine> {
    let width = width.max(1);
    let units = hyperlink_line_units(&line);
    let mut lines = Vec::new();
    let mut start = 0;
    let mut use_prefix = false;

    while start < units.len() {
        let prefix = use_prefix.then(|| continuation_prefix.clone()).flatten();
        let prefix_width = prefix.as_ref().map(span_width).unwrap_or(0);
        let available_width = width.saturating_sub(prefix_width).max(1);
        let token_end = non_whitespace_token_end(&units, start);
        if token_end > start
            && units_width(&units[start..token_end]) > available_width
            && is_url_like_units(&units[start..token_end])
        {
            lines.push(hyperlink_line_from_units(prefix, &units[start..token_end]));
            start = skip_whitespace_units(&units, token_end);
            use_prefix = true;
            continue;
        }

        let mut current_width = 0;
        let mut end = start;
        let mut last_space_after = None;

        while end < units.len() {
            let unit = &units[end];
            if current_width > 0 && current_width + unit.width > available_width {
                break;
            }
            if current_width == 0 && unit.width > available_width {
                end += 1;
                break;
            }
            current_width += unit.width;
            end += 1;
            if unit.ch.is_whitespace() {
                last_space_after = Some(end);
            }
        }

        let mut next_start = end;
        let mut line_end = end;
        if end < units.len() {
            if units[end].ch.is_whitespace() {
                line_end = trim_trailing_whitespace_units(&units, end);
                next_start = skip_whitespace_units(&units, end);
            } else if let Some(space_after) = last_space_after
                && space_after > start
            {
                line_end = trim_trailing_whitespace_units(&units, space_after);
                next_start = skip_whitespace_units(&units, space_after);
            }
        }
        if line_end <= start {
            line_end = end.max(start + 1).min(units.len());
            next_start = line_end;
        }

        lines.push(hyperlink_line_from_units(prefix, &units[start..line_end]));
        start = next_start;
        use_prefix = true;
    }

    if lines.is_empty() {
        lines.push(HyperlinkLine::new(Line::default()));
    }
    lines
}

#[derive(Debug, Clone)]
struct HyperlinkUnit {
    ch: char,
    width: usize,
    style: Style,
    destination: Option<String>,
}

fn hyperlink_line_units(line: &HyperlinkLine) -> Vec<HyperlinkUnit> {
    let mut units = Vec::new();
    let mut source_column = 0;
    for span in &line.line.spans {
        for ch in span.content.chars() {
            let width = crate::review::terminal_char_width(ch);
            let destination = line
                .hyperlinks
                .iter()
                .find(|link| link.columns.contains(&source_column))
                .map(|link| link.destination.clone());
            units.push(HyperlinkUnit {
                ch,
                width,
                style: line.line.style.patch(span.style),
                destination,
            });
            source_column += width;
        }
    }
    units
}

fn hyperlink_line_from_units(
    prefix: Option<Span<'static>>,
    units: &[HyperlinkUnit],
) -> HyperlinkLine {
    let mut line = HyperlinkLine::default();
    if let Some(prefix) = prefix {
        line.push_span(prefix, None);
    }
    for unit in units {
        let span = Span::styled(unit.ch.to_string(), unit.style);
        line.push_span(span, unit.destination.as_deref());
    }
    line
}

fn skip_whitespace_units(units: &[HyperlinkUnit], mut index: usize) -> usize {
    while index < units.len() && units[index].ch.is_whitespace() {
        index += 1;
    }
    index
}

fn non_whitespace_token_end(units: &[HyperlinkUnit], mut index: usize) -> usize {
    while index < units.len() && !units[index].ch.is_whitespace() {
        index += 1;
    }
    index
}

fn units_width(units: &[HyperlinkUnit]) -> usize {
    units.iter().map(|unit| unit.width).sum()
}

fn is_url_like_units(units: &[HyperlinkUnit]) -> bool {
    let text = units.iter().map(|unit| unit.ch).collect::<String>();
    is_url_like_token(&text)
}

fn is_url_like_token(text: &str) -> bool {
    if text.contains("://") {
        return true;
    }
    if !(text.contains('/') || text.contains('\\')) {
        return false;
    }
    text.contains('.')
        || text.contains('?')
        || text.contains('#')
        || text.matches('/').count() + text.matches('\\').count() >= 2
}

fn trim_trailing_whitespace_units(units: &[HyperlinkUnit], mut end: usize) -> usize {
    while end > 0 && units[end - 1].ch.is_whitespace() {
        end -= 1;
    }
    end
}

fn push_hyperlink_range(
    hyperlinks: &mut Vec<TerminalHyperlink>,
    columns: Range<usize>,
    destination: &str,
) {
    if columns.is_empty() {
        return;
    }
    if hyperlinks
        .iter()
        .any(|link| link.destination == destination && link.columns == columns)
    {
        return;
    }
    if let Some(previous) = hyperlinks.last_mut()
        && previous.destination == destination
        && previous.columns.end == columns.start
    {
        previous.columns.end = columns.end;
        return;
    }
    hyperlinks.push(TerminalHyperlink {
        columns,
        destination: destination.to_string(),
    });
}

fn annotate_web_urls(mut line: HyperlinkLine) -> HyperlinkLine {
    let text = line_text(&line.line);
    line.hyperlinks.extend(web_links_in_text(&text));
    line
}

fn annotate_table_link_fragments(
    table_line: &PreviewLine,
    table_lines: Vec<String>,
    style: Style,
) -> Vec<HyperlinkLine> {
    let mut rendered = table_lines
        .into_iter()
        .map(|line| annotate_web_urls(HyperlinkLine::new(Line::styled(line, style))))
        .collect::<Vec<_>>();
    let links = match table_line {
        PreviewLine::TableHeader { links, .. } | PreviewLine::TableRow { links, .. } => links,
        _ => return rendered,
    };

    let mut cursor = TableLinkSearchCursor::default();
    for link in links {
        annotate_table_link_fragment(&mut rendered, &link.text, &link.target, &mut cursor);
    }
    rendered
}

#[derive(Debug, Clone, Copy, Default)]
struct TableLinkSearchCursor {
    line_index: usize,
    byte_start: usize,
}

fn annotate_table_link_fragment(
    lines: &mut [HyperlinkLine],
    link_text: &str,
    destination: &str,
    cursor: &mut TableLinkSearchCursor,
) {
    if link_text.is_empty() {
        return;
    }

    let mut remaining_start = 0;
    for (line_index, line) in lines.iter_mut().enumerate().skip(cursor.line_index) {
        if remaining_start >= link_text.len() {
            break;
        }
        let line_text = line_text(&line.line);
        let search_from = if line_index == cursor.line_index {
            cursor.byte_start.min(line_text.len())
        } else {
            0
        };
        let remaining = &link_text[remaining_start..];
        if let Some(relative_start) = line_text[search_from..].find(remaining) {
            let byte_start = search_from + relative_start;
            if let Some(columns) = display_range_in_text(&line_text, remaining, byte_start) {
                push_hyperlink_range(&mut line.hyperlinks, columns, destination);
            }
            cursor.line_index = line_index;
            cursor.byte_start = byte_start + remaining.len();
            break;
        }

        let Some((relative_start, fragment_len)) =
            longest_link_fragment_in_line(&line_text[search_from..], remaining)
        else {
            continue;
        };
        let byte_start = search_from + relative_start;
        if let Some(columns) =
            display_range_in_text(&line_text, &remaining[..fragment_len], byte_start)
        {
            push_hyperlink_range(&mut line.hyperlinks, columns, destination);
        }
        remaining_start += fragment_len;
        cursor.line_index = line_index + 1;
        cursor.byte_start = 0;
    }
}

fn longest_link_fragment_in_line(line: &str, remaining: &str) -> Option<(usize, usize)> {
    let mut best = None;
    for (offset, _) in remaining.char_indices() {
        let suffix = &remaining[offset..];
        if suffix.trim().is_empty() {
            continue;
        }
        let mut end = suffix.len();
        while end > 0 && !suffix.is_char_boundary(end) {
            end -= 1;
        }
        while end > 0 {
            let candidate = &suffix[..end];
            if !candidate.trim().is_empty()
                && let Some(byte_start) = line.find(candidate)
            {
                let score = text_width(candidate);
                if best
                    .as_ref()
                    .is_none_or(|(_, length, best_score)| score > *best_score && *length < end)
                {
                    best = Some((byte_start, offset + end, score));
                }
                break;
            }
            end = previous_char_boundary(suffix, end);
        }
    }
    best.map(|(byte_start, consumed, _)| (byte_start, consumed))
}

fn previous_char_boundary(text: &str, mut index: usize) -> usize {
    if index == 0 {
        return 0;
    }
    index -= 1;
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn display_range_in_text(text: &str, needle: &str, byte_start: usize) -> Option<Range<usize>> {
    if byte_start > text.len() || !text[byte_start..].starts_with(needle) {
        return None;
    }
    let start = text_width(&text[..byte_start]);
    Some(start..start + text_width(needle))
}

fn web_links_in_text(text: &str) -> Vec<TerminalHyperlink> {
    let mut links = Vec::new();
    let mut search_from = 0;
    for raw_token in text.split_whitespace() {
        let Some(relative_start) = text[search_from..].find(raw_token) else {
            continue;
        };
        let raw_start = search_from + relative_start;
        search_from = raw_start + raw_token.len();

        let trimmed_start = raw_token
            .find(|ch: char| !matches!(ch, '(' | '[' | '{' | '<' | '"' | '\''))
            .unwrap_or(raw_token.len());
        let trimmed_end = trailing_web_url_end(&raw_token[trimmed_start..]) + trimmed_start;
        if trimmed_start >= trimmed_end {
            continue;
        }

        let candidate = &raw_token[trimmed_start..trimmed_end];
        let Some(destination) = web_destination(candidate) else {
            continue;
        };
        let start = text_width(&text[..raw_start + trimmed_start]);
        let end = start + text_width(candidate);
        links.push(TerminalHyperlink {
            columns: start..end,
            destination,
        });
    }
    links
}

fn trailing_web_url_end(candidate: &str) -> usize {
    let mut end = candidate.len();
    while end > 0 {
        let remaining = &candidate[..end];
        let Some(ch) = remaining.chars().next_back() else {
            break;
        };
        if !matches!(
            ch,
            ',' | '.' | ';' | '!' | '"' | '\'' | ')' | ']' | '}' | '>'
        ) {
            break;
        }
        end -= ch.len_utf8();
    }
    end
}

fn web_destination(destination: &str) -> Option<String> {
    let safe = safe_hyperlink_destination(destination)?;
    let parsed = Url::parse(&safe).ok()?;
    matches!(parsed.scheme(), "http" | "https")
        .then(|| parsed.host_str())
        .flatten()?;
    Some(safe)
}

fn safe_hyperlink_destination(destination: &str) -> Option<String> {
    let safe = destination
        .chars()
        .filter(|ch| !ch.is_control())
        .collect::<String>();
    if safe.is_empty() {
        return None;
    }

    if let Ok(parsed) = Url::parse(&safe) {
        return matches!(parsed.scheme(), "http" | "https" | "file").then_some(safe);
    }

    is_local_path_like_destination(&safe).then_some(safe)
}

fn is_local_path_like_destination(destination: &str) -> bool {
    destination.starts_with('/')
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

fn line_width(line: &Line<'_>) -> usize {
    line.spans.iter().map(span_width).sum()
}

fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn span_width(span: &Span<'_>) -> usize {
    span.content
        .chars()
        .map(crate::review::terminal_char_width)
        .sum()
}

fn text_width(text: &str) -> usize {
    text.chars().map(crate::review::terminal_char_width).sum()
}

fn source_boundary_line(
    prefix: Option<&str>,
    phase: &str,
    gate: &str,
    source: &str,
) -> Line<'static> {
    let mut spans = Vec::new();
    if let Some(prefix) = prefix {
        spans.push(Span::styled(format!("{prefix} "), source_label_style()));
    }
    spans.extend([
        Span::styled(phase.to_string(), phase_style(phase)),
        Span::styled(" / ".to_string(), chrome_style()),
        Span::styled(gate.to_string(), strong_style()),
        Span::raw("  "),
        Span::styled(source.to_string(), dim_style()),
    ]);
    Line::from(spans)
}

fn phase_style(phase: &str) -> Style {
    let color = match phase {
        "Learn" => Color::Cyan,
        "Example" => Color::Green,
        "Architect" => Color::Magenta,
        "Feedback" => Color::Yellow,
        "Status" => Color::Gray,
        _ => Color::White,
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

fn clamped_review_scroll_offset(
    lines: &[RenderedReviewLine],
    body_height: u16,
    scroll_offset: usize,
) -> usize {
    scroll_offset.min(max_review_scroll_for_body(lines, body_height))
}

fn max_review_scroll_for_body(lines: &[RenderedReviewLine], body_height: u16) -> usize {
    lines.len().saturating_sub(body_height as usize)
}

fn chrome_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(chrome_style())
        .title_style(chrome_style())
}

fn review_body_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(chrome_style())
}

fn stage_filter_label(filter: StageFilter) -> &'static str {
    match filter {
        StageFilter::All => "all",
        StageFilter::Stage(StageDir::Sprouts) => "sprouts",
        StageFilter::Stage(StageDir::Leaves) => "leaves",
        StageFilter::Stage(StageDir::Fallen) => "fallen",
        StageFilter::Stage(StageDir::Pressed) => "pressed",
    }
}

fn parse_state_style(state: ParseState) -> Style {
    match state {
        ParseState::Ok => Style::default().fg(Color::Green),
        ParseState::Partial => Style::default().fg(Color::Yellow),
        ParseState::Error => Style::default().fg(Color::Red),
    }
}

fn stage_style(stage_dir: StageDir) -> Style {
    match stage_dir {
        StageDir::Sprouts => Style::default().fg(Color::Cyan),
        StageDir::Leaves => Style::default().fg(Color::Green),
        StageDir::Fallen => Style::default().fg(Color::Magenta),
        StageDir::Pressed => Style::default().fg(Color::Blue),
    }
}

fn chrome_style() -> Style {
    Style::default().fg(markdown_style_profile().chrome)
}

fn dim_style() -> Style {
    Style::default().fg(markdown_style_profile().dim)
}

fn strong_style() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

fn heading_style(level: u8) -> Style {
    let base = strong_style();
    match level {
        1 => base.add_modifier(Modifier::UNDERLINED),
        2 => base,
        3 => base.add_modifier(Modifier::ITALIC),
        _ => Style::default().add_modifier(Modifier::ITALIC),
    }
}

fn heading_display_text(level: u8, text: &str) -> String {
    let level = level.clamp(1, 6);
    let prefix = "#".repeat(level as usize);
    format!("{prefix} {text}")
}

fn table_header_style() -> Style {
    Style::default()
        .fg(markdown_style_profile().table_header)
        .add_modifier(Modifier::BOLD)
}

fn table_rule_style() -> Style {
    Style::default().fg(markdown_style_profile().table_rule)
}

fn quote_style() -> Style {
    Style::default().fg(markdown_style_profile().quote)
}

fn list_marker_style() -> Style {
    Style::default().fg(markdown_style_profile().list_marker)
}

fn source_label_style() -> Style {
    Style::default()
        .fg(markdown_style_profile().source_label)
        .add_modifier(Modifier::BOLD)
}

fn code_style() -> Style {
    Style::default().fg(markdown_style_profile().code)
}

fn link_style() -> Style {
    Style::default()
        .fg(markdown_style_profile().link)
        .add_modifier(Modifier::UNDERLINED)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MarkdownStyleProfile {
    chrome: Color,
    dim: Color,
    heading: Color,
    table_header: Color,
    table_rule: Color,
    quote: Color,
    list_marker: Color,
    code: Color,
    link: Color,
    source_label: Color,
}

impl MarkdownStyleProfile {
    fn from_markdown_settings(settings: &crate::codex_config::CodexMarkdownRenderSettings) -> Self {
        if !settings.use_theme_colors {
            return Self::from_codex_theme_name(settings.theme.as_deref(), false);
        }
        Self::from_codex_theme_name_with_accents(
            settings.theme.as_deref(),
            true,
            crate::syntax::theme_accent_colors(),
        )
    }

    fn from_codex_theme_name(theme_name: Option<&str>, use_theme_colors: bool) -> Self {
        Self::from_codex_theme_name_with_accents(
            theme_name,
            use_theme_colors,
            crate::syntax::ThemeAccentColors::default(),
        )
    }

    fn from_codex_theme_name_with_accents(
        theme_name: Option<&str>,
        use_theme_colors: bool,
        accents: crate::syntax::ThemeAccentColors,
    ) -> Self {
        let appearance = theme_appearance(theme_name);
        if !use_theme_colors {
            return match appearance {
                ThemeAppearance::Light => Self {
                    chrome: Color::DarkGray,
                    dim: Color::Gray,
                    heading: Color::Black,
                    table_header: Color::Black,
                    table_rule: Color::Gray,
                    quote: Color::DarkGray,
                    list_marker: Color::DarkGray,
                    code: Color::DarkGray,
                    link: Color::Black,
                    source_label: Color::DarkGray,
                },
                ThemeAppearance::Dark => Self {
                    chrome: Color::Gray,
                    dim: Color::DarkGray,
                    heading: Color::White,
                    table_header: Color::White,
                    table_rule: Color::DarkGray,
                    quote: Color::Gray,
                    list_marker: Color::Gray,
                    code: Color::Gray,
                    link: Color::White,
                    source_label: Color::Gray,
                },
            };
        }

        match appearance {
            ThemeAppearance::Light => {
                let primary = syntax_accent_color(accents.primary, Color::Blue);
                let secondary = syntax_accent_color(accents.secondary, primary);
                let muted = syntax_accent_color(accents.muted, Color::DarkGray);
                Self {
                    chrome: Color::DarkGray,
                    dim: Color::Gray,
                    heading: Color::Black,
                    table_header: primary,
                    table_rule: Color::Gray,
                    quote: secondary,
                    list_marker: primary,
                    code: primary,
                    link: primary,
                    source_label: muted,
                }
            }
            ThemeAppearance::Dark => {
                let primary = syntax_accent_color(accents.primary, Color::Cyan);
                let secondary = syntax_accent_color(accents.secondary, Color::Yellow);
                let muted = syntax_accent_color(accents.muted, Color::DarkGray);
                Self {
                    chrome: Color::Gray,
                    dim: Color::DarkGray,
                    heading: Color::White,
                    table_header: secondary,
                    table_rule: muted,
                    quote: primary,
                    list_marker: primary,
                    code: primary,
                    link: primary,
                    source_label: secondary,
                }
            }
        }
    }
}

fn syntax_accent_color(color: Option<PreviewColor>, fallback: Color) -> Color {
    color.map(preview_color).unwrap_or(fallback)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeAppearance {
    Dark,
    Light,
}

fn markdown_style_profile() -> &'static MarkdownStyleProfile {
    MARKDOWN_STYLE_PROFILE.get_or_init(|| {
        let settings = crate::codex_config::CodexSettings::load();
        MarkdownStyleProfile::from_markdown_settings(&settings.markdown_render_settings())
    })
}

fn theme_appearance(theme_name: Option<&str>) -> ThemeAppearance {
    let Some(theme_name) = theme_name else {
        return ThemeAppearance::Dark;
    };
    let normalized = normalized_theme_name(theme_name);
    if normalized.ends_with("-light")
        || matches!(
            normalized.as_str(),
            "catppuccin-latte" | "coldark-cold" | "github" | "inspired-github"
        )
    {
        ThemeAppearance::Light
    } else {
        ThemeAppearance::Dark
    }
}

fn normalized_theme_name(name: &str) -> String {
    let mut normalized = String::new();
    for ch in name.trim().chars() {
        match ch {
            ' ' | '_' => normalized.push('-'),
            _ => normalized.extend(ch.to_lowercase()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{
        Inventory, InventoryItem, ItemKind, ParseState, PreviewSource, StageDir, StageInventory,
        StatusSummary,
    };
    use crate::tui::app::{AppState, KeyInput, Outcome};
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

    fn render_buffer(width: u16, height: u16, app: &AppState) -> Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn buffer_text(width: u16, height: u16, app: &AppState) -> String {
        let buffer = render_buffer(width, height, app);
        buffer_to_text(&buffer, width, height)
    }

    fn buffer_to_text(buffer: &Buffer, width: u16, height: u16) -> String {
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn review_body_inner_adds_horizontal_padding_when_roomy() {
        let body_shell = Rect::new(0, 0, 80, 10);
        let body = review_body_inner(body_shell);

        assert_eq!(body.x, 2);
        assert_eq!(body.width, 76);
        assert_eq!(body.y, 1);
        assert_eq!(body.height, 8);
    }

    #[test]
    fn review_body_inner_omits_horizontal_padding_when_cramped() {
        let body_shell = Rect::new(0, 0, 40, 10);
        let body = review_body_inner(body_shell);

        assert_eq!(body.x, 1);
        assert_eq!(body.width, 38);
        assert_eq!(body.y, 1);
        assert_eq!(body.height, 8);
    }

    #[test]
    fn padded_content_inner_adds_horizontal_padding_when_roomy() {
        let inner = Rect::new(1, 1, 78, 8);
        let padded = padded_content_inner(inner);

        assert_eq!(padded.x, 2);
        assert_eq!(padded.width, 76);
        assert_eq!(padded.y, 1);
        assert_eq!(padded.height, 8);
    }

    #[test]
    fn fall_input_mode_renders_centered_modal_with_reason_and_hints() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Sprouts,
            "draft",
            status(
                ParseState::Ok,
                Some("sprout"),
                Some("learn"),
                Some("intent"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        app.handle_key(KeyInput::Char('F'));
        app.handle_key(KeyInput::Char('h'));
        app.handle_key(KeyInput::Char('i'));

        let text = buffer_text(120, 24, &app);
        assert!(text.contains("Fall"), "modal title missing: {text}");
        assert!(text.contains("reason:"), "reason line missing: {text}");
        assert!(text.contains("hi"), "typed reason missing: {text}");
        assert!(
            text.contains("Enter") && text.contains("Esc"),
            "confirm/cancel hint missing: {text}"
        );
        assert!(
            text.contains("draft"),
            "target slug preview missing: {text}"
        );
    }

    #[test]
    fn list_status_hint_includes_fall_shortcut() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Sprouts,
            "draft",
            status(
                ParseState::Ok,
                Some("sprout"),
                Some("learn"),
                Some("intent"),
            ),
        )]);
        let app = AppState::from_inventory(&inventory);

        let text = buffer_text(200, 24, &app);
        assert!(
            text.contains("F fall"),
            "status hint missing F fall: {text}"
        );
    }

    fn strip_osc8(text: &str) -> String {
        let bytes = text.as_bytes();
        let mut stripped = String::with_capacity(text.len());
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index..].starts_with(b"\x1b]8;;") {
                index += 5;
                while index < bytes.len() {
                    if bytes[index] == b'\x07' {
                        index += 1;
                        break;
                    }
                    index += 1;
                }
                continue;
            }
            let ch = text[index..]
                .chars()
                .next()
                .expect("index starts a character");
            stripped.push(ch);
            index += ch.len_utf8();
        }
        stripped
    }

    fn buffer_line(buffer: &Buffer, width: u16, y: u16) -> String {
        (0..width)
            .map(|x| buffer[(x, y)].symbol().to_string())
            .collect::<String>()
    }

    fn row_text(buffer: &Buffer, width: u16, y: u16) -> String {
        buffer_line(buffer, width, y)
    }

    fn line_contains_text(buffer: &Buffer, width: u16, height: u16, text: &str) -> bool {
        (0..height).any(|y| {
            (0..width).any(|x| {
                let mut cursor = x;
                for ch in text.chars() {
                    if cursor >= width || buffer[(cursor, y)].symbol() != ch.to_string() {
                        return false;
                    }
                    cursor = cursor.saturating_add(cell_width(ch));
                }
                true
            })
        })
    }

    fn text_position(buffer: &Buffer, width: u16, height: u16, text: &str) -> Option<(u16, u16)> {
        (0..height).find_map(|y| {
            (0..width).find_map(|x| {
                let mut cursor = x;
                for ch in text.chars() {
                    if cursor >= width || buffer[(cursor, y)].symbol() != ch.to_string() {
                        return None;
                    }
                    cursor = cursor.saturating_add(cell_width(ch));
                }
                Some((x, y))
            })
        })
    }

    fn text_cell_style(
        buffer: &Buffer,
        width: u16,
        height: u16,
        text: &str,
    ) -> Option<(Color, Color, Modifier)> {
        let position = (2..height.saturating_sub(1)).find_map(|y| {
            (0..width).find_map(|x| {
                let mut cursor = x;
                for ch in text.chars() {
                    if cursor >= width || buffer[(cursor, y)].symbol() != ch.to_string() {
                        return None;
                    }
                    cursor = cursor.saturating_add(cell_width(ch));
                }
                Some((x, y))
            })
        });

        position
            .or_else(|| text_position(buffer, width, height, text))
            .map(|(x, y)| {
                let cell = &buffer[(x, y)];
                (cell.fg, cell.bg, cell.modifier)
            })
    }

    fn cell_width(ch: char) -> u16 {
        if ch.is_ascii() { 1 } else { 2 }
    }

    #[test]
    fn renders_header_table_preview_and_status() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            "korean-preview",
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
        )]);
        let app = AppState::from_inventory(&inventory);

        let buffer = render_buffer(120, 24, &app);
        let text = buffer_to_text(&buffer, 120, 24);

        assert!(text.contains("leaf list"));
        assert!(text.contains("STAGE"));
        assert!(!text.contains("STATE"));
        assert!(text.contains("korean-preview"));
        assert!(text.contains(".leaf/02-leaves/korean-preview"));
        assert!(line_contains_text(
            &buffer,
            110,
            24,
            "다음 행동을 정리한다."
        ));
        assert!(text.contains("q quit"));
    }

    #[test]
    fn list_header_uses_two_line_section_header() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            "section-header",
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        app.handle_key(KeyInput::Right);
        app.handle_key(KeyInput::Right);

        let buffer = render_buffer(120, 24, &app);
        let first_line = buffer_line(&buffer, 120, 0);
        let second_line = buffer_line(&buffer, 120, 1);

        assert!(
            first_line.contains("Inventory"),
            "first line: {first_line:?}"
        );
        assert!(
            first_line.contains("leaf list"),
            "first line: {first_line:?}"
        );
        assert!(first_line.contains("leaves"), "first line: {first_line:?}");
        assert!(first_line.contains("1/1"), "first line: {first_line:?}");
        assert!(
            !first_line.contains("stage_dir "),
            "first line: {first_line:?}"
        );
        assert!(
            second_line.contains("filter none"),
            "second line: {second_line:?}"
        );
        assert!(
            !second_line.contains("selected 0"),
            "second line: {second_line:?}"
        );
    }

    #[test]
    fn list_header_shows_selected_count_only_when_rows_are_marked() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![
            fixture.plain_leaf("alpha"),
            fixture.plain_leaf("beta"),
        ]);
        let mut app = AppState::from_inventory(&inventory);

        let initial = render_buffer(120, 24, &app);
        assert!(!buffer_line(&initial, 120, 1).contains("selected"));

        app.handle_key(KeyInput::Char(' '));
        let selected = render_buffer(120, 24, &app);

        assert!(buffer_line(&selected, 120, 1).contains("selected 1"));
    }

    #[test]
    fn list_notice_renders_below_header_in_strong_yellow() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.plain_leaf("alpha")]);
        let mut app = AppState::from_inventory(&inventory);
        app.set_notice("copied row alpha");

        let buffer = render_buffer(80, 12, &app);

        assert!(row_text(&buffer, 80, 2).contains("copied row alpha"));
        assert!(!row_text(&buffer, 80, 11).contains("copied row alpha"));
        let position =
            text_position(&buffer, 80, 12, "copied row alpha").expect("notice should render");
        let first_cell = &buffer[(position.0, position.1)];
        assert_eq!(first_cell.fg, Color::Yellow);
        assert!(first_cell.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn list_notice_line_is_blank_when_no_notice() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.plain_leaf("alpha")]);
        let app = AppState::from_inventory(&inventory);

        let buffer = render_buffer(80, 12, &app);

        assert_eq!(row_text(&buffer, 80, 2).trim(), "");
    }

    #[test]
    fn small_terminal_render_with_notice_does_not_panic() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.plain_leaf("alpha")]);
        let mut app = AppState::from_inventory(&inventory);
        app.set_notice("refreshed");

        let buffer = render_buffer(40, 6, &app);
        let text = buffer_to_text(&buffer, 40, 6);
        assert!(text.contains("refreshed"));
    }

    #[test]
    fn wide_terminal_places_preview_on_the_right() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            "wide-preview",
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
        )]);
        let app = AppState::from_inventory(&inventory);

        let buffer = render_buffer(160, 24, &app);
        let preview_position = text_position(&buffer, 160, 24, ".leaf/02-leaves/wide-preview")
            .expect("preview path should render");

        assert!(
            preview_position.0 >= 88,
            "right preview should start after the table, got {preview_position:?}"
        );
        assert!(
            preview_position.1 < 6,
            "right preview should align near the top content, got {preview_position:?}"
        );
    }

    #[test]
    fn medium_terminal_falls_back_to_bottom_preview() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            "bottom-preview",
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
        )]);
        let app = AppState::from_inventory(&inventory);

        let buffer = render_buffer(120, 24, &app);
        let preview_position = text_position(&buffer, 120, 24, ".leaf/02-leaves/bottom-preview")
            .expect("preview path should render");

        assert!(
            preview_position.0 < 40,
            "bottom preview should use the full width from the left, got {preview_position:?}"
        );
        assert!(
            preview_position.1 >= 12,
            "bottom preview should sit below the table, got {preview_position:?}"
        );
    }

    #[test]
    fn small_terminal_hides_preview_without_hiding_rows() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            "compact",
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
        )]);
        let app = AppState::from_inventory(&inventory);

        let text = buffer_text(80, 8, &app);

        assert!(text.contains("compact"));
        assert!(!text.contains("다음 행동을 정리한다."));
    }

    #[test]
    fn tall_bottom_layout_allocates_bounded_preview_height() {
        let area = Rect::new(0, 0, 120, 30);

        assert_eq!(bottom_preview_height(area), 12);
    }

    #[test]
    fn filter_mode_status_shows_filter_text() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            "alpha",
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        app.handle_key(KeyInput::Char('/'));
        app.handle_key(KeyInput::Char('a'));

        let text = buffer_text(80, 12, &app);

        assert!(text.contains("filter: a"));
    }

    #[test]
    fn normal_status_omits_promote_hint() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Sprouts,
            "draft",
            status(ParseState::Ok, Some("sprout"), Some("Learn"), Some("-")),
        )]);
        let app = AppState::from_inventory(&inventory);

        let text = buffer_text(90, 12, &app);

        assert!(!text.contains("P promote"));
    }

    #[test]
    fn normal_status_renders_refresh_hint() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Sprouts,
            "draft",
            status(ParseState::Ok, Some("sprout"), Some("Learn"), Some("-")),
        )]);
        let app = AppState::from_inventory(&inventory);

        let text = buffer_text(110, 12, &app);

        assert!(text.contains("r refresh"));
    }

    #[test]
    fn normal_status_renders_copy_hint() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            "alpha",
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
        )]);
        let app = AppState::from_inventory(&inventory);

        let text = buffer_text(100, 12, &app);

        assert!(text.contains("y copy"));
    }

    #[test]
    fn preview_line_renders_list_item_marker_and_text() {
        let line = preview_line(&PreviewLine::ListItem {
            marker: "•".to_string(),
            spans: vec![PreviewSpan::Plain("source item".to_string())],
        });

        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert!(rendered.contains("•"));
        assert!(rendered.contains("source item"));
    }

    #[test]
    fn preview_line_renders_source_boundary_with_phase_color() {
        let line = preview_line(&PreviewLine::SourceBoundary {
            phase: "Example".to_string(),
            gate: "③ Criteria".to_string(),
            source: "02-Example/03-criteria.md".to_string(),
        });
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert!(rendered.contains("Example"));
        assert!(rendered.contains("③ Criteria"));
        assert!(rendered.contains("02-Example/03-criteria.md"));
        let phase_span = line
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "Example")
            .expect("phase span");
        assert_eq!(phase_span.style.fg, Some(Color::Green));
        assert!(phase_span.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn preview_lines_add_vertical_rhythm_between_markdown_blocks() {
        let rendered = preview_lines_with_rhythm(&[
            PreviewLine::SourceBoundary {
                phase: "Learn".to_string(),
                gate: "① Intent".to_string(),
                source: "01-Learn/01-intent.md".to_string(),
            },
            PreviewLine::Heading {
                level: 1,
                text: "Intent".to_string(),
            },
            PreviewLine::Plain("body".to_string()),
            PreviewLine::Code("fn main() {}".to_string()),
            PreviewLine::Plain("after".to_string()),
            PreviewLine::ListItem {
                marker: "•".to_string(),
                spans: vec![PreviewSpan::Plain("item".to_string())],
            },
            PreviewLine::ListItem {
                marker: "•".to_string(),
                spans: vec![PreviewSpan::Plain("next".to_string())],
            },
        ]);
        let text = rendered.iter().map(line_text).collect::<Vec<_>>();

        assert_eq!(
            text,
            vec![
                "Learn / ① Intent  01-Learn/01-intent.md",
                "",
                "# Intent",
                "",
                "body",
                "",
                "fn main() {}",
                "",
                "after",
                "• item",
                "• next",
            ]
        );
    }

    #[test]
    fn preview_lines_add_vertical_rhythm_around_quote_blocks() {
        let rendered = preview_lines_with_rhythm(&[
            PreviewLine::Plain("before".to_string()),
            PreviewLine::BlockQuote {
                depth: 1,
                prefix: "> ".to_string(),
                line: Box::new(PreviewLine::Plain("quoted one".to_string())),
            },
            PreviewLine::BlockQuote {
                depth: 1,
                prefix: "> ".to_string(),
                line: Box::new(PreviewLine::Plain("quoted two".to_string())),
            },
            PreviewLine::Plain("after".to_string()),
        ]);
        let text = rendered.iter().map(line_text).collect::<Vec<_>>();

        assert_eq!(
            text,
            vec!["before", "", "│ quoted one", "│ quoted two", "", "after"]
        );
    }

    #[test]
    fn preview_line_renders_syntax_span_style() {
        let line = preview_line(&PreviewLine::CodeSpans(vec![PreviewSpan::Syntax {
            text: "fn".to_string(),
            style: crate::preview::PreviewStyle {
                fg: Some(crate::preview::PreviewColor::Rgb(1, 2, 3)),
                bg: Some(crate::preview::PreviewColor::Rgb(4, 5, 6)),
                text_style: crate::preview::PreviewTextStyle {
                    bold: true,
                    underline: true,
                    ..Default::default()
                },
            },
        }]));

        assert_eq!(line.spans[0].content.as_ref(), "fn");
        assert_eq!(line.spans[0].style.fg, Some(Color::Rgb(1, 2, 3)));
        assert_eq!(line.spans[0].style.bg, Some(Color::Rgb(4, 5, 6)));
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert!(
            line.spans[0]
                .style
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
    }

    #[test]
    fn preview_line_renders_heading_level_styles() {
        let h1 = preview_line(&PreviewLine::Heading {
            level: 1,
            text: "Title".to_string(),
        });
        let h3 = preview_line(&PreviewLine::Heading {
            level: 3,
            text: "Subhead".to_string(),
        });
        let h6 = preview_line(&PreviewLine::Heading {
            level: 6,
            text: "Minor".to_string(),
        });

        assert_eq!(line_text(&h1), "# Title");
        assert_eq!(line_text(&h3), "### Subhead");
        assert_eq!(line_text(&h6), "###### Minor");
        assert!(h1.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert!(
            h1.spans[0]
                .style
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
        assert!(h3.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert!(h3.spans[0].style.add_modifier.contains(Modifier::ITALIC));
        assert!(!h6.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert!(h6.spans[0].style.add_modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn preview_line_renders_inline_text_style_modifiers() {
        let line = preview_line(&PreviewLine::Styled(vec![PreviewSpan::StyledText {
            text: "styled".to_string(),
            style: crate::preview::PreviewTextStyle {
                bold: true,
                italic: true,
                underline: true,
                strikethrough: true,
            },
        }]));

        assert_eq!(line.spans[0].content.as_ref(), "styled");
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert!(line.spans[0].style.add_modifier.contains(Modifier::ITALIC));
        assert!(
            line.spans[0]
                .style
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
        assert!(
            line.spans[0]
                .style
                .add_modifier
                .contains(Modifier::CROSSED_OUT)
        );
    }

    #[test]
    fn markdown_style_profile_tracks_codex_theme_appearance() {
        let dark = MarkdownStyleProfile::from_codex_theme_name(Some("catppuccin_mocha"), true);
        let light = MarkdownStyleProfile::from_codex_theme_name(Some("Gruvbox Light"), true);

        assert_eq!(dark.link, Color::Cyan);
        assert_eq!(dark.code, Color::Cyan);
        assert_eq!(dark.heading, Color::White);
        assert_eq!(dark.table_header, Color::Yellow);
        assert_eq!(dark.table_rule, Color::DarkGray);
        assert_eq!(dark.quote, Color::Cyan);
        assert_eq!(dark.list_marker, Color::Cyan);
        assert_eq!(dark.source_label, Color::Yellow);
        assert_eq!(light.link, Color::Blue);
        assert_eq!(light.code, Color::Blue);
        assert_eq!(light.heading, Color::Black);
        assert_eq!(light.table_header, Color::Blue);
        assert_eq!(light.table_rule, Color::Gray);
        assert_eq!(light.quote, Color::Blue);
        assert_eq!(light.list_marker, Color::Blue);
        assert_eq!(light.source_label, Color::DarkGray);
        assert_eq!(theme_appearance(Some("github")), ThemeAppearance::Light);
        assert_eq!(theme_appearance(Some("dracula")), ThemeAppearance::Dark);
    }

    #[test]
    fn markdown_style_profile_can_disable_theme_accent_colors_from_codex_config() {
        let colored = MarkdownStyleProfile::from_codex_theme_name(Some("catppuccin_mocha"), true);
        let neutral = MarkdownStyleProfile::from_codex_theme_name(Some("catppuccin_mocha"), false);

        assert_eq!(colored.table_header, Color::Yellow);
        assert_eq!(colored.link, Color::Cyan);
        assert_eq!(neutral.table_header, Color::White);
        assert_eq!(neutral.link, Color::White);
        assert_eq!(neutral.code, Color::Gray);
        assert_eq!(neutral.list_marker, Color::Gray);
    }

    #[test]
    fn markdown_style_profile_uses_syntax_theme_accent_colors_when_available() {
        let accents = crate::syntax::ThemeAccentColors {
            primary: Some(PreviewColor::Rgb(1, 2, 3)),
            secondary: Some(PreviewColor::Rgb(4, 5, 6)),
            muted: Some(PreviewColor::Ansi(8)),
        };

        let profile = MarkdownStyleProfile::from_codex_theme_name_with_accents(
            Some("dracula"),
            true,
            accents,
        );

        assert_eq!(profile.link, Color::Rgb(1, 2, 3));
        assert_eq!(profile.code, Color::Rgb(1, 2, 3));
        assert_eq!(profile.quote, Color::Rgb(1, 2, 3));
        assert_eq!(profile.list_marker, Color::Rgb(1, 2, 3));
        assert_eq!(profile.table_header, Color::Rgb(4, 5, 6));
        assert_eq!(profile.source_label, Color::Rgb(4, 5, 6));
        assert_eq!(profile.table_rule, Color::Indexed(8));
    }

    #[test]
    fn markdown_style_profile_uses_codex_config_theme() {
        let temp = assert_fs::TempDir::new().expect("temp codex home");
        let config = temp.path().join("config.toml");
        std::fs::write(
            &config,
            "[tui]\ntheme = \"github\"\nstatus_line_use_colors = false\n",
        )
        .expect("config");
        let settings = crate::codex_config::CodexSettings::from_config_path(&config);

        let render_settings = settings.markdown_render_settings();
        let profile = MarkdownStyleProfile::from_markdown_settings(&render_settings);

        assert_eq!(settings.tui.theme.as_deref(), Some("github"));
        assert!(!settings.tui.status_line_use_colors);
        assert_eq!(render_settings.theme.as_deref(), Some("github"));
        assert!(!render_settings.use_theme_colors);
        assert_eq!(profile.link, Color::Black);
        assert_eq!(profile.code, Color::DarkGray);
        assert_eq!(profile.chrome, Color::DarkGray);
        assert_eq!(profile.heading, Color::Black);
        assert_eq!(profile.table_header, Color::Black);
        assert_eq!(profile.list_marker, Color::DarkGray);
    }

    #[test]
    fn markdown_element_styles_use_separate_profile_tokens() {
        let header = preview_line(&PreviewLine::TableHeader {
            cells: vec!["Plain check".to_string(), "EARS".to_string()],
            links: Vec::new(),
            widths: vec![11, 4],
            alignments: vec![
                crate::preview::TableAlignment::None,
                crate::preview::TableAlignment::None,
            ],
        });
        let divider = preview_line(&PreviewLine::TableDivider {
            widths: vec![11, 4],
            kind: crate::preview::TableDividerKind::Header,
        });
        let list_item = preview_line(&PreviewLine::ListItem {
            marker: "•".to_string(),
            spans: vec![PreviewSpan::Plain("item".to_string())],
        });
        let quoted = review_blockquote_lines(
            &ReviewLine::Markdown(PreviewLine::BlockQuote {
                depth: 1,
                prefix: "> ".to_string(),
                line: Box::new(PreviewLine::Plain("quoted".to_string())),
            }),
            80,
        )
        .expect("blockquote lines");

        assert_eq!(header.style.fg, Some(markdown_style_profile().table_header));
        assert!(header.style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(divider.style.fg, Some(markdown_style_profile().table_rule));
        assert_eq!(
            list_item.spans[0].style.fg,
            Some(markdown_style_profile().list_marker)
        );
        assert_eq!(
            quoted[0].line.spans[0].style.fg,
            Some(markdown_style_profile().quote)
        );
    }

    #[test]
    fn blockquote_prefix_renders_as_quote_gutter() {
        let quoted = review_blockquote_lines(
            &ReviewLine::Markdown(PreviewLine::BlockQuote {
                depth: 1,
                prefix: "> ".to_string(),
                line: Box::new(PreviewLine::Plain("quoted".to_string())),
            }),
            80,
        )
        .expect("blockquote lines");

        assert_eq!(line_text(&quoted[0].line), "│ quoted");
    }

    #[test]
    fn nested_blockquote_prefix_renders_as_nested_quote_gutter() {
        let quoted = review_blockquote_lines(
            &ReviewLine::Markdown(PreviewLine::BlockQuote {
                depth: 2,
                prefix: "> > ".to_string(),
                line: Box::new(PreviewLine::Plain("quoted".to_string())),
            }),
            80,
        )
        .expect("blockquote lines");

        assert_eq!(line_text(&quoted[0].line), "│ │ quoted");
    }

    #[test]
    fn review_separator_renders_as_prominent_file_boundary() {
        let line = review_line(&ReviewLine::Separator {
            relative_path: ".leaf/02-leaves/demo/01-Learn/01-intent.md".to_string(),
            phase: "Learn".to_string(),
            gate: "① Intent".to_string(),
        });
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert!(rendered.starts_with("FILE "));
        assert!(rendered.contains("Learn"));
        assert!(rendered.contains("① Intent"));
        assert!(rendered.contains(".leaf/02-leaves/demo/01-Learn/01-intent.md"));
        let phase_span = line
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "Learn")
            .expect("phase span");
        assert_eq!(phase_span.style.fg, Some(Color::Cyan));
        assert!(
            line.spans
                .iter()
                .any(|span| span.style.add_modifier.contains(Modifier::BOLD))
        );
    }

    #[test]
    fn highlight_user_review_markers_styles_marker_span_red_bold() {
        let line = HyperlinkLine::new(Line::from(
            "USER REVIEW NEEDED: 강조 marker 범위를 정해야 한다.",
        ));

        let highlighted = highlight_user_review_markers(line.clone());

        assert_eq!(line_text(&highlighted.line), line_text(&line.line));
        let marker_span = &highlighted.line.spans[0];
        assert_eq!(marker_span.content.as_ref(), "USER REVIEW NEEDED:");
        assert_eq!(marker_span.style.fg, Some(Color::Red));
        assert!(marker_span.style.add_modifier.contains(Modifier::BOLD));
        let rest = &highlighted.line.spans[1];
        assert_ne!(rest.style.fg, Some(Color::Red));
    }

    #[test]
    fn highlight_user_review_markers_highlights_every_occurrence() {
        let line = HyperlinkLine::new(Line::from(
            "USER REVIEW NEEDED: surface와 USER REVIEW NEEDED 강조 방식",
        ));

        let highlighted = highlight_user_review_markers(line);

        let red: Vec<&str> = highlighted
            .line
            .spans
            .iter()
            .filter(|span| span.style.fg == Some(Color::Red))
            .map(|span| span.content.as_ref())
            .collect();
        assert_eq!(red, vec!["USER REVIEW NEEDED:", "USER REVIEW NEEDED"]);
    }

    #[test]
    fn highlight_user_review_markers_matches_across_span_boundaries() {
        let line = HyperlinkLine::new(Line::from(vec![
            Span::raw("USER REVIEW "),
            Span::styled("NEEDED", strong_style()),
            Span::raw(" 확인"),
        ]));

        let highlighted = highlight_user_review_markers(line);

        let red_text: String = highlighted
            .line
            .spans
            .iter()
            .filter(|span| {
                span.style.fg == Some(Color::Red)
                    && span.style.add_modifier.contains(Modifier::BOLD)
            })
            .map(|span| span.content.as_ref())
            .collect();
        assert_eq!(red_text, "USER REVIEW NEEDED");
    }

    #[test]
    fn highlight_user_review_markers_no_marker_is_identity() {
        let line = HyperlinkLine::new(Line::from("일반 본문 줄입니다."));
        assert_eq!(highlight_user_review_markers(line.clone()), line);
    }

    #[test]
    fn highlight_user_review_markers_ignores_lowercase() {
        let line = HyperlinkLine::new(Line::from("user review needed: 소문자"));
        assert_eq!(highlight_user_review_markers(line.clone()), line);
    }

    #[test]
    fn highlight_user_review_markers_preserves_hyperlinks() {
        let mut line = HyperlinkLine::new(Line::from(vec![
            Span::raw("USER REVIEW NEEDED: see "),
            Span::styled("docs", link_style()),
        ]));
        line.hyperlinks.push(TerminalHyperlink {
            columns: 24..28,
            destination: "https://example.com".to_string(),
        });

        let highlighted = highlight_user_review_markers(line.clone());

        assert_eq!(highlighted.hyperlinks, line.hyperlinks);
        assert_eq!(line_text(&highlighted.line), line_text(&line.line));
    }

    #[test]
    fn wrapped_review_lines_keep_marker_highlight_across_wrap() {
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/01-sprouts/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/01-sprouts/demo/00-status.md".to_string(),
                start_line: 0,
            }],
            lines: vec![ReviewLine::Markdown(PreviewLine::Plain(
                "aaaa aaaa aaaa USER REVIEW NEEDED: tail".to_string(),
            ))],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 20);

        let red_chars: String = rendered
            .iter()
            .flat_map(|line| line.content.line.spans.iter())
            .filter(|span| {
                span.style.fg == Some(Color::Red)
                    && span.style.add_modifier.contains(Modifier::BOLD)
            })
            .map(|span| span.content.as_ref().to_string())
            .collect();
        assert_eq!(red_chars.replace(' ', ""), "USERREVIEWNEEDED:");
    }

    #[test]
    fn wrapped_review_lines_do_not_highlight_markers_in_code_blocks() {
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/01-sprouts/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/01-sprouts/demo/00-status.md".to_string(),
                start_line: 0,
            }],
            lines: vec![ReviewLine::Markdown(PreviewLine::Code(
                "USER REVIEW NEEDED: code sample".to_string(),
            ))],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 80);

        assert!(
            rendered
                .iter()
                .flat_map(|line| line.content.line.spans.iter())
                .all(|span| span.style.fg != Some(Color::Red))
        );
    }

    #[test]
    fn review_table_lines_highlight_marker_cells() {
        let line = ReviewLine::Markdown(PreviewLine::TableRow {
            headers: vec!["Status".to_string()],
            cells: vec!["USER REVIEW NEEDED".to_string()],
            links: vec![],
            widths: vec![18],
            alignments: vec![crate::preview::TableAlignment::None],
        });

        let rendered = review_table_lines(&line, 80).expect("table row");

        assert!(rendered[0].line.spans.iter().any(|span| {
            span.content.as_ref() == "USER REVIEW NEEDED"
                && span.style.fg == Some(Color::Red)
                && span.style.add_modifier.contains(Modifier::BOLD)
        }));
    }

    #[test]
    fn review_blockquote_lines_highlight_marker_but_not_inner_code() {
        let quoted = ReviewLine::Markdown(PreviewLine::BlockQuote {
            depth: 1,
            prefix: "> ".to_string(),
            line: Box::new(PreviewLine::Plain(
                "USER REVIEW NEEDED: 인용 본문".to_string(),
            )),
        });
        let rendered = review_blockquote_lines(&quoted, 80).expect("blockquote");
        let red_text: String = rendered[0]
            .line
            .spans
            .iter()
            .filter(|span| {
                span.style.fg == Some(Color::Red)
                    && span.style.add_modifier.contains(Modifier::BOLD)
            })
            .map(|span| span.content.as_ref())
            .collect();
        assert_eq!(red_text, "USER REVIEW NEEDED:");

        let quoted_code = ReviewLine::Markdown(PreviewLine::BlockQuote {
            depth: 1,
            prefix: "> ".to_string(),
            line: Box::new(PreviewLine::Code("USER REVIEW NEEDED: code".to_string())),
        });
        let rendered = review_blockquote_lines(&quoted_code, 80).expect("blockquote");
        assert!(
            rendered
                .iter()
                .flat_map(|line| line.line.spans.iter())
                .all(|span| span.style.fg != Some(Color::Red))
        );
    }

    #[test]
    fn review_list_lines_highlight_marker_items() {
        let line = ReviewLine::Markdown(PreviewLine::ListItem {
            marker: "-".to_string(),
            spans: vec![PreviewSpan::Plain(
                "USER REVIEW NEEDED: 항목 판단".to_string(),
            )],
        });

        let rendered = review_list_lines(&line, 80).expect("list item");

        let red_text: String = rendered[0]
            .line
            .spans
            .iter()
            .filter(|span| {
                span.style.fg == Some(Color::Red)
                    && span.style.add_modifier.contains(Modifier::BOLD)
            })
            .map(|span| span.content.as_ref())
            .collect();
        assert_eq!(red_text, "USER REVIEW NEEDED:");
    }

    #[test]
    fn preview_line_does_not_highlight_user_review_marker() {
        let line = PreviewLine::Plain("USER REVIEW NEEDED: preview는 불변".to_string());

        let rendered = preview_line(&line);

        assert!(
            rendered
                .spans
                .iter()
                .all(|span| span.style.fg != Some(Color::Red))
        );
    }

    #[test]
    fn review_body_wraps_long_lines_to_terminal_width() {
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/00-status.md".to_string(),
                start_line: 0,
            }],
            lines: vec![ReviewLine::Markdown(PreviewLine::Plain(
                "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string(),
            ))],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 20);
        let text = rendered
            .iter()
            .map(|line| {
                line.content
                    .line
                    .spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>();

        assert_eq!(text.len(), 4);
        assert_eq!(text[0], "abcdefghijklmnopqrst");
        assert_eq!(text[1], "uvwxyz0123456789ABCD");
        assert_eq!(text[3], "YZ");
    }

    #[test]
    fn review_body_wraps_plain_text_at_word_boundaries() {
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/00-status.md".to_string(),
                start_line: 0,
            }],
            lines: vec![ReviewLine::Markdown(PreviewLine::Plain(
                "This is a simple sentence that should wrap.".to_string(),
            ))],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 16);
        let text = rendered
            .iter()
            .map(|line| line_text(&line.content.line))
            .collect::<Vec<_>>();

        assert_eq!(
            text,
            vec!["This is a simple", "sentence that", "should wrap."]
        );
    }

    #[test]
    fn review_lines_add_vertical_rhythm_without_changing_link_targets() {
        let destination = "https://example.com/docs";
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/00-status.md".to_string(),
                start_line: 0,
            }],
            lines: vec![
                ReviewLine::Markdown(PreviewLine::Heading {
                    level: 1,
                    text: "Title".to_string(),
                }),
                ReviewLine::Markdown(PreviewLine::Styled(vec![PreviewSpan::Link {
                    text: "docs".to_string(),
                    target: destination.to_string(),
                    source_range: crate::preview::PreviewSourceRange { start: 0, end: 4 },
                    local: false,
                }])),
                ReviewLine::Markdown(PreviewLine::Code("fn main() {}".to_string())),
                ReviewLine::Markdown(PreviewLine::Plain("after".to_string())),
            ],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 80);
        let text = rendered
            .iter()
            .map(|line| line_text(&line.content.line))
            .collect::<Vec<_>>();

        assert_eq!(
            text,
            vec!["# Title", "", "docs", "", "fn main() {}", "", "after"]
        );
        assert!(
            rendered[0].content.line.spans[0]
                .style
                .add_modifier
                .contains(Modifier::BOLD)
        );
        assert!(
            rendered[0].content.line.spans[0]
                .style
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
        assert_eq!(
            rendered[2].content.hyperlinks,
            vec![TerminalHyperlink {
                columns: 0..4,
                destination: destination.to_string(),
            }]
        );
    }

    #[test]
    fn review_lines_add_vertical_rhythm_around_quote_blocks() {
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/00-status.md".to_string(),
                start_line: 0,
            }],
            lines: vec![
                ReviewLine::Markdown(PreviewLine::Plain("before".to_string())),
                ReviewLine::Markdown(PreviewLine::BlockQuote {
                    depth: 1,
                    prefix: "> ".to_string(),
                    line: Box::new(PreviewLine::Plain("quoted one".to_string())),
                }),
                ReviewLine::Markdown(PreviewLine::BlockQuote {
                    depth: 1,
                    prefix: "> ".to_string(),
                    line: Box::new(PreviewLine::Plain("quoted two".to_string())),
                }),
                ReviewLine::Markdown(PreviewLine::Plain("after".to_string())),
            ],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 80);
        let text = rendered
            .iter()
            .map(|line| line_text(&line.content.line))
            .collect::<Vec<_>>();

        assert_eq!(
            text,
            vec!["before", "", "│ quoted one", "│ quoted two", "", "after"]
        );
    }

    #[test]
    fn review_lines_add_vertical_rhythm_before_h1_after_file_boundary() {
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/00-status.md".to_string(),
                start_line: 1,
            }],
            lines: vec![
                ReviewLine::Separator {
                    relative_path: ".leaf/02-leaves/demo/00-status.md".to_string(),
                    phase: "Status".to_string(),
                    gate: "Status".to_string(),
                },
                ReviewLine::Markdown(PreviewLine::Heading {
                    level: 1,
                    text: "Leaf Status".to_string(),
                }),
                ReviewLine::Markdown(PreviewLine::Plain("body".to_string())),
            ],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 80);
        let text = rendered
            .iter()
            .map(|line| line_text(&line.content.line))
            .collect::<Vec<_>>();

        assert_eq!(
            text,
            vec![
                "FILE Status / Status  .leaf/02-leaves/demo/00-status.md",
                "",
                "# Leaf Status",
                "",
                "body"
            ]
        );
    }

    #[test]
    fn review_body_lines_add_centered_end_marker() {
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: Vec::new(),
            lines: vec![ReviewLine::Markdown(PreviewLine::Plain("body".to_string()))],
            source_count: 1,
        };

        let rendered = review_body_lines(&document, 20);
        let text = rendered
            .iter()
            .map(|line| line_text(&line.content.line))
            .collect::<Vec<_>>();

        assert_eq!(text, vec!["body", "", "     -- END --"]);
    }

    #[test]
    fn review_body_does_not_split_long_url_like_token_without_scheme() {
        let url_like =
            "example.test/api/v1/projects/alpha-team/releases/2026-02-17/builds/1234567890";
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/00-status.md".to_string(),
                start_line: 0,
            }],
            lines: vec![ReviewLine::Markdown(PreviewLine::Plain(
                url_like.to_string(),
            ))],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 24);
        let text = rendered
            .iter()
            .map(|line| line_text(&line.content.line))
            .collect::<Vec<_>>();

        assert_eq!(text, vec![url_like]);
    }

    #[test]
    fn review_wrapping_keeps_long_url_hyperlink_token_intact() {
        let destination =
            "https://example.com/a-very-long-path-with-many-segments-and-query?x=1&y=2";
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/00-status.md".to_string(),
                start_line: 0,
            }],
            lines: vec![ReviewLine::Markdown(PreviewLine::Styled(vec![
                PreviewSpan::Plain("see ".to_string()),
                PreviewSpan::Link {
                    text: destination.to_string(),
                    target: destination.to_string(),
                    source_range: crate::preview::PreviewSourceRange {
                        start: 4,
                        end: 4 + destination.len(),
                    },
                    local: false,
                },
            ]))],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 24);
        let text = rendered
            .iter()
            .map(|line| line_text(&line.content.line))
            .collect::<Vec<_>>();

        assert_eq!(text, vec!["see", destination]);
        assert_eq!(
            rendered[1].content.hyperlinks,
            vec![TerminalHyperlink {
                columns: 0..destination.len(),
                destination: destination.to_string(),
            }]
        );
    }

    #[test]
    fn review_wrapping_preserves_list_item_continuation_indent() {
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/01-Learn/01-intent.md".to_string(),
                start_line: 0,
            }],
            lines: vec![ReviewLine::Markdown(PreviewLine::ListItem {
                marker: "•".to_string(),
                spans: vec![PreviewSpan::Plain("first second third fourth".to_string())],
            })],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 16);
        let text = rendered
            .iter()
            .map(|line| line_text(&line.content.line))
            .collect::<Vec<_>>();

        assert_eq!(text, vec!["• first second", "  third fourth"]);
    }

    #[test]
    fn review_wrapping_preserves_checkbox_continuation_indent() {
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/01-Learn/01-intent.md".to_string(),
                start_line: 0,
            }],
            lines: vec![ReviewLine::Markdown(PreviewLine::Checkbox {
                marker: "  •".to_string(),
                checked: true,
                text: "first second third".to_string(),
            })],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 18);
        let text = rendered
            .iter()
            .map(|line| line_text(&line.content.line))
            .collect::<Vec<_>>();

        assert_eq!(
            text,
            vec!["  • [x] first", "        second", "        third"]
        );
    }

    #[test]
    fn review_wrapping_preserves_link_ranges_across_wrapped_lines() {
        let destination = "https://example.com/very/long/path";
        let label = "abcdefghijklmnopqrstuvwxyz0123456789";
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/00-status.md".to_string(),
                start_line: 0,
            }],
            lines: vec![ReviewLine::Markdown(PreviewLine::Styled(vec![
                PreviewSpan::Plain("See ".to_string()),
                PreviewSpan::Link {
                    text: label.to_string(),
                    target: destination.to_string(),
                    source_range: crate::preview::PreviewSourceRange {
                        start: 4,
                        end: 4 + label.len(),
                    },
                    local: false,
                },
                PreviewSpan::Plain(" now".to_string()),
            ]))],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 14);
        let linked_lines = rendered
            .iter()
            .filter(|line| {
                line.content
                    .hyperlinks
                    .iter()
                    .any(|link| link.destination == destination)
            })
            .collect::<Vec<_>>();

        assert!(
            linked_lines.len() >= 2,
            "expected wrapped link fragments: {rendered:?}"
        );
        for line in linked_lines {
            let width = line.content.width();
            for link in &line.content.hyperlinks {
                assert_eq!(link.destination, destination);
                assert!(link.columns.start < link.columns.end);
                assert!(link.columns.end <= width);
            }
        }
    }

    #[test]
    fn visible_review_body_lines_preserve_hyperlink_metadata_after_scroll_windowing() {
        let destination = "https://example.com/docs";
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/00-status.md".to_string(),
                start_line: 0,
            }],
            lines: vec![
                ReviewLine::Markdown(PreviewLine::Plain("before".to_string())),
                ReviewLine::Markdown(PreviewLine::Styled(vec![PreviewSpan::Link {
                    text: "docs".to_string(),
                    target: destination.to_string(),
                    source_range: crate::preview::PreviewSourceRange { start: 0, end: 4 },
                    local: false,
                }])),
                ReviewLine::Markdown(PreviewLine::Plain("after".to_string())),
            ],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 80);
        let visible = visible_review_body_lines(&rendered, 1, 1);

        assert_eq!(visible.len(), 1);
        assert_eq!(line_text(&visible[0].content.line), "docs");
        assert_eq!(
            visible[0].content.hyperlinks,
            vec![TerminalHyperlink {
                columns: 0..4,
                destination: destination.to_string(),
            }]
        );
    }

    #[test]
    fn review_blockquote_wrapping_shifts_link_ranges_by_prefix_width() {
        let destination = "https://example.com";
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/00-status.md".to_string(),
                start_line: 0,
            }],
            lines: vec![ReviewLine::Markdown(PreviewLine::BlockQuote {
                depth: 1,
                prefix: "> ".to_string(),
                line: Box::new(PreviewLine::Styled(vec![PreviewSpan::Link {
                    text: "docs".to_string(),
                    target: destination.to_string(),
                    source_range: crate::preview::PreviewSourceRange { start: 0, end: 4 },
                    local: false,
                }])),
            })],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 20);

        assert_eq!(
            rendered[0].content.hyperlinks,
            vec![TerminalHyperlink {
                columns: 2..6,
                destination: destination.to_string(),
            }]
        );
    }

    #[test]
    fn review_wrapping_complex_snapshot_preserves_nested_link_ranges() {
        let destination = "file:///Users/example/code/codex/README.md#L10";
        let document = ReviewDocument {
            title: "demo".to_string(),
            root_relative_path: ".leaf/02-leaves/demo".to_string(),
            sections: vec![crate::review::ReviewSection {
                relative_path: ".leaf/02-leaves/demo/00-status.md".to_string(),
                start_line: 0,
            }],
            lines: vec![
                ReviewLine::Markdown(PreviewLine::BlockQuote {
                    depth: 1,
                    prefix: "> ".to_string(),
                    line: Box::new(PreviewLine::ListItem {
                        marker: "•".to_string(),
                        spans: vec![
                            PreviewSpan::Plain("see ".to_string()),
                            PreviewSpan::Link {
                                text: "README.md:10".to_string(),
                                target: destination.to_string(),
                                source_range: crate::preview::PreviewSourceRange {
                                    start: 0,
                                    end: 12,
                                },
                                local: true,
                            },
                        ],
                    }),
                }),
                ReviewLine::Markdown(PreviewLine::BlockQuote {
                    depth: 1,
                    prefix: "> ".to_string(),
                    line: Box::new(PreviewLine::Code("  fn main() {}".to_string())),
                }),
            ],
            source_count: 1,
        };

        let rendered = wrapped_review_lines(&document, 14);
        let text = rendered
            .iter()
            .map(|line| line_text(&line.content.line))
            .collect::<Vec<_>>();
        let linked_lines = rendered
            .iter()
            .filter(|line| {
                line.content
                    .hyperlinks
                    .iter()
                    .any(|link| link.destination == destination)
            })
            .collect::<Vec<_>>();

        assert_eq!(
            text,
            vec![
                "│ • see",
                "│   README.md:",
                "│   10",
                "",
                "│   fn main()",
                "│ {}"
            ]
        );
        assert_eq!(linked_lines.len(), 2);
        assert_eq!(
            linked_lines[0].content.hyperlinks,
            vec![TerminalHyperlink {
                columns: 4..14,
                destination: destination.to_string(),
            }]
        );
        assert_eq!(
            linked_lines[1].content.hyperlinks,
            vec![TerminalHyperlink {
                columns: 4..6,
                destination: destination.to_string(),
            }]
        );
    }

    #[test]
    fn terminal_hyperlink_text_adds_osc8_without_changing_visible_text() {
        let destination = "https://example.com/very/long/path";
        let mut line = HyperlinkLine::new(Line::from(vec![
            Span::raw("See "),
            Span::styled(destination.to_string(), link_style()),
            Span::raw(" now."),
        ]));
        line.hyperlinks.push(TerminalHyperlink {
            columns: 4..4 + destination.len(),
            destination: destination.to_string(),
        });

        let raw = terminal_hyperlink_text(&line);
        let visible = strip_osc8(&raw);

        assert!(raw.contains("\x1b]8;;https://example.com/very/long/path\x07"));
        assert_eq!(visible, format!("See {destination} now."));
        assert!(visible.contains("now."));
    }

    #[test]
    fn terminal_hyperlink_text_rejects_unsafe_schemes() {
        let mut line = HyperlinkLine::new(Line::from("open"));
        line.hyperlinks.push(TerminalHyperlink {
            columns: 0..4,
            destination: "javascript:alert(1)".to_string(),
        });

        let raw = terminal_hyperlink_text(&line);

        assert_eq!(raw, "open");
        assert!(!raw.contains("\x1b]8;;"));
        assert_eq!(safe_hyperlink_destination("data:text/plain,hello"), None);
        assert_eq!(
            safe_hyperlink_destination("file:///Users/example/code/codex/README.md").as_deref(),
            Some("file:///Users/example/code/codex/README.md")
        );
        assert_eq!(
            safe_hyperlink_destination("/Users/example/code/codex/README.md").as_deref(),
            Some("/Users/example/code/codex/README.md")
        );
    }

    #[test]
    fn review_table_lines_recover_web_url_hyperlinks_from_rendered_cells() {
        let destination = "https://example.com/docs";
        let line = ReviewLine::Markdown(PreviewLine::TableRow {
            headers: vec!["URL".to_string()],
            cells: vec![destination.to_string()],
            links: vec![],
            widths: vec![destination.len()],
            alignments: vec![crate::preview::TableAlignment::None],
        });

        let rendered = review_table_lines(&line, 80).expect("table row");
        let raw = terminal_hyperlink_text(&rendered[0]);
        let visible = strip_osc8(&raw);

        assert_eq!(line_text(&rendered[0].line), destination);
        assert_eq!(
            rendered[0].hyperlinks,
            vec![TerminalHyperlink {
                columns: 0..destination.len(),
                destination: destination.to_string(),
            }]
        );
        assert!(raw.contains("\x1b]8;;https://example.com/docs\x07"));
        assert_eq!(visible, destination);
    }

    #[test]
    fn review_table_lines_preserve_local_file_hyperlinks_from_table_metadata() {
        let visible = "README.md:12";
        let destination = "file:///Users/example/code/codex/README.md#L12";
        let line = ReviewLine::Markdown(PreviewLine::TableRow {
            headers: vec!["File".to_string()],
            cells: vec![visible.to_string()],
            links: vec![crate::preview::PreviewTableLink {
                cell: 0,
                text: visible.to_string(),
                columns: 0..visible.len(),
                target: destination.to_string(),
                source_range: crate::preview::PreviewSourceRange { start: 0, end: 12 },
                local: true,
            }],
            widths: vec![visible.len()],
            alignments: vec![crate::preview::TableAlignment::None],
        });

        let rendered = review_table_lines(&line, 80).expect("table row");
        let raw = terminal_hyperlink_text(&rendered[0]);

        assert_eq!(line_text(&rendered[0].line), visible);
        assert_eq!(
            rendered[0].hyperlinks,
            vec![TerminalHyperlink {
                columns: 0..visible.len(),
                destination: destination.to_string(),
            }]
        );
        assert!(raw.contains("\x1b]8;;file:///Users/example/code/codex/README.md#L12\x07"));
        assert_eq!(strip_osc8(&raw), visible);
    }

    #[test]
    fn review_table_lines_keep_duplicate_cell_link_text_targets_distinct() {
        let visible = "README.md:1";
        let first = "file:///Users/example/code/codex/a/README.md#L1";
        let second = "file:///Users/example/code/codex/b/README.md#L1";
        let line = ReviewLine::Markdown(PreviewLine::TableRow {
            headers: vec!["A".to_string(), "B".to_string()],
            cells: vec![visible.to_string(), visible.to_string()],
            links: vec![
                crate::preview::PreviewTableLink {
                    cell: 0,
                    text: visible.to_string(),
                    columns: 0..visible.len(),
                    target: first.to_string(),
                    source_range: crate::preview::PreviewSourceRange { start: 0, end: 10 },
                    local: true,
                },
                crate::preview::PreviewTableLink {
                    cell: 1,
                    text: visible.to_string(),
                    columns: 0..visible.len(),
                    target: second.to_string(),
                    source_range: crate::preview::PreviewSourceRange { start: 11, end: 21 },
                    local: true,
                },
            ],
            widths: vec![visible.len(), visible.len()],
            alignments: vec![
                crate::preview::TableAlignment::None,
                crate::preview::TableAlignment::None,
            ],
        });

        let rendered = review_table_lines(&line, 80).expect("table row");

        assert_eq!(
            line_text(&rendered[0].line),
            format!("{visible}    {visible}")
        );
        assert_eq!(
            rendered[0].hyperlinks,
            vec![
                TerminalHyperlink {
                    columns: 0..visible.len(),
                    destination: first.to_string(),
                },
                TerminalHyperlink {
                    columns: visible.len() + crate::preview::TABLE_COLUMN_GAP
                        ..visible.len() + crate::preview::TABLE_COLUMN_GAP + visible.len(),
                    destination: second.to_string(),
                },
            ]
        );
    }

    #[test]
    fn review_table_lines_preserve_wrapped_local_file_hyperlink_fragments() {
        let visible = "codex-rs/tui/src/markdown_render.rs:74:3";
        let destination =
            "file:///Users/example/code/codex/codex-rs/tui/src/markdown_render.rs#L74C3";
        let line = ReviewLine::Markdown(PreviewLine::TableRow {
            headers: vec!["File".to_string()],
            cells: vec![visible.to_string()],
            links: vec![crate::preview::PreviewTableLink {
                cell: 0,
                text: visible.to_string(),
                columns: 0..visible.len(),
                target: destination.to_string(),
                source_range: crate::preview::PreviewSourceRange { start: 0, end: 12 },
                local: true,
            }],
            widths: vec![visible.len()],
            alignments: vec![crate::preview::TableAlignment::None],
        });

        let rendered = review_table_lines(&line, 16).expect("table row");
        let linked_lines = rendered
            .iter()
            .filter(|line| {
                line.hyperlinks
                    .iter()
                    .any(|link| link.destination == destination)
            })
            .collect::<Vec<_>>();

        assert!(
            linked_lines.len() >= 2,
            "expected wrapped linked fragments: {rendered:?}"
        );
        assert_eq!(
            rendered
                .iter()
                .map(|line| line_text(&line.line))
                .collect::<String>(),
            visible
        );
        for line in linked_lines {
            let width = line.width();
            for hyperlink in &line.hyperlinks {
                assert_eq!(hyperlink.destination, destination);
                assert!(hyperlink.columns.start < hyperlink.columns.end);
                assert!(hyperlink.columns.end <= width);
            }
            assert!(terminal_hyperlink_text(line).contains(destination));
        }
    }

    #[test]
    fn normal_status_renders_mouse_drag_hint() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            "alpha",
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
        )]);
        let app = AppState::from_inventory(&inventory);

        let text = buffer_text(140, 12, &app);

        assert!(text.contains("mouse drag"));
    }

    #[test]
    fn normal_status_renders_multi_select_hints() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            "alpha",
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
        )]);
        let app = AppState::from_inventory(&inventory);

        let text = buffer_text(120, 12, &app);

        assert!(text.contains("Space select"));
        assert!(text.contains("v range"));
        assert!(text.contains("a all"));
        assert!(text.contains("y copy"));
    }

    #[test]
    fn marked_rows_use_row_highlight_without_selection_column() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            "alpha",
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
        )]);
        let mut app = AppState::from_inventory(&inventory);

        let before = buffer_text(110, 14, &app);
        assert!(!before.contains('*'));
        assert!(!before.contains("SEL"));

        app.handle_key(KeyInput::Char(' '));
        let after = buffer_text(120, 14, &app);
        assert!(!after.contains('*'));
        assert!(after.contains("1 selected"));
        assert!(after.contains("Space toggle"));
        assert!(after.contains("Esc clear"));
    }

    #[test]
    fn selected_row_semantic_cells_keep_readable_selected_style() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            "alpha",
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
        )]);
        let app = AppState::from_inventory(&inventory);

        let buffer = render_buffer(100, 12, &app);

        for text in ["leaf", "ok", "alpha"] {
            let (fg, bg, modifier) =
                text_cell_style(&buffer, 100, 12, text).expect("cursor row cell");
            assert!(modifier.contains(Modifier::REVERSED), "{text} not reversed");
            assert!(
                !modifier.contains(Modifier::BOLD),
                "{text} unexpectedly bold"
            );
            assert_eq!(fg, Color::Reset, "{text} fg must stay theme-default");
            assert_eq!(bg, Color::Reset, "{text} bg must stay theme-default");
        }
    }

    #[test]
    fn marked_row_uses_blue_bold_text_without_background_fill() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![
            fixture.plain_leaf("alpha"),
            fixture.plain_leaf("beta"),
        ]);
        let mut app = AppState::from_inventory(&inventory);
        app.handle_key(KeyInput::Char(' '));
        app.handle_key(KeyInput::Char('j'));

        let buffer = render_buffer(100, 12, &app);

        let (fg, bg, modifier) =
            text_cell_style(&buffer, 100, 12, "alpha").expect("marked row cell");
        assert_eq!(fg, Color::Blue);
        assert_eq!(bg, Color::Reset);
        assert!(modifier.contains(Modifier::BOLD));
        assert!(!modifier.contains(Modifier::REVERSED));

        let (cursor_fg, cursor_bg, cursor_modifier) =
            text_cell_style(&buffer, 100, 12, "beta").expect("cursor row cell");
        assert!(cursor_modifier.contains(Modifier::REVERSED));
        assert_eq!(cursor_fg, Color::Reset);
        assert_eq!(cursor_bg, Color::Reset);
    }

    #[test]
    fn cursor_on_marked_row_uses_reversed_bold() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![
            fixture.plain_leaf("alpha"),
            fixture.plain_leaf("beta"),
        ]);
        let mut app = AppState::from_inventory(&inventory);
        app.handle_key(KeyInput::Char(' '));

        let buffer = render_buffer(100, 12, &app);

        let (fg, bg, modifier) =
            text_cell_style(&buffer, 100, 12, "alpha").expect("marked cursor row cell");
        assert!(modifier.contains(Modifier::REVERSED));
        assert!(modifier.contains(Modifier::BOLD));
        assert_eq!(fg, Color::Reset);
        assert_eq!(bg, Color::Reset);
    }

    #[test]
    fn range_mode_status_renders_extend_copy_and_quit_hints() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            "alpha",
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        app.handle_key(KeyInput::Char('v'));

        let text = buffer_text(120, 14, &app);

        assert!(text.contains("range 1 selected"));
        assert!(text.contains("j/k extend"));
        assert!(text.contains("v/Esc done"));
        assert!(text.contains("y copy"));
        assert!(text.contains("q quit"));
    }

    #[test]
    fn table_mouse_target_maps_data_rows_without_selection_column() {
        let fixture = RenderFixture::new();
        let app = AppState::from_inventory(&fixture.inventory_with_items(vec![
            fixture.plain_leaf("alpha"),
            fixture.plain_leaf("beta"),
            fixture.plain_leaf("gamma"),
        ]));

        // Terminal Rect(0,0,80,10): header y=0..1, notice y=2, table y=3 height=6, status y=9.
        // Data rows begin below the table border/header, and roomy tables add 1ch content padding.
        let area = Rect::new(0, 0, 80, 10);

        assert_eq!(table_mouse_target(area, &app, 20, 5), Some(0));
        assert_eq!(table_mouse_target(area, &app, 2, 6), Some(1));
        assert_eq!(table_mouse_target(area, &app, 1, 7), None);
        assert_eq!(table_mouse_target(area, &app, 3, 7), Some(2));
        assert_eq!(table_mouse_target(area, &app, 4, 7), Some(2));
    }

    #[test]
    fn table_mouse_target_honors_viewport_offset() {
        let fixture = RenderFixture::new();
        let leaves: Vec<_> = (0..10)
            .map(|index| fixture.plain_leaf(&format!("leaf-{index:02}")))
            .collect();
        let mut app = AppState::from_inventory(&fixture.inventory_with_items(leaves));
        for _ in 0..9 {
            app.handle_key(KeyInput::Down);
        }
        assert_eq!(app.selected_index(), 9);

        // table height 6 -> capacity 3 -> offset = 9 - 2 = 7.
        let area = Rect::new(0, 0, 80, 10);
        assert_eq!(table_mouse_target(area, &app, 20, 5), Some(7));
        assert_eq!(table_mouse_target(area, &app, 20, 7), Some(9));
    }

    #[test]
    fn table_mouse_target_ignores_right_preview_area() {
        let fixture = RenderFixture::new();
        let app = AppState::from_inventory(&fixture.inventory_with_items(vec![
            fixture.plain_leaf("alpha"),
            fixture.plain_leaf("beta"),
            fixture.plain_leaf("gamma"),
        ]));
        let area = Rect::new(0, 0, 160, 24);

        assert_eq!(table_mouse_target(area, &app, 20, 5), Some(0));
        assert_eq!(table_mouse_target(area, &app, 110, 5), None);
    }

    #[test]
    fn table_mouse_target_ignores_header_border_and_out_of_range() {
        let fixture = RenderFixture::new();
        let app = AppState::from_inventory(&fixture.inventory_with_items(vec![
            fixture.plain_leaf("alpha"),
            fixture.plain_leaf("beta"),
        ]));
        let area = Rect::new(0, 0, 80, 10);

        assert_eq!(table_mouse_target(area, &app, 20, 2), None); // notice
        assert_eq!(table_mouse_target(area, &app, 20, 3), None); // top border
        assert_eq!(table_mouse_target(area, &app, 20, 4), None); // header
        assert_eq!(table_mouse_target(area, &app, 20, 8), None); // bottom border
        assert_eq!(table_mouse_target(area, &app, 20, 9), None); // status line
        assert_eq!(table_mouse_target(area, &app, 20, 7), None); // beyond last row
        assert_eq!(table_mouse_target(area, &app, 0, 5), None); // left border
        assert_eq!(table_mouse_target(area, &app, 79, 5), None); // right border
    }

    #[test]
    fn empty_inventory_renders_without_panicking() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(Vec::new());
        let app = AppState::from_inventory(&inventory);

        let text = buffer_text(50, 6, &app);

        assert!(text.contains("No leaf items"));
    }

    #[test]
    fn review_mode_renders_read_only_source_path_and_rendered_markdown() {
        let fixture = RenderFixture::new();
        let slug = "demo";
        let root = fixture.root.path();
        let status_path = root
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug)
            .join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("leaf dir");
        std::fs::write(&status_path, "# Leaf 상태\n\n- current gate: ① Intent\n").expect("status");
        let intent_path = root
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug)
            .join("01-Learn/01-intent.md");
        std::fs::create_dir_all(intent_path.parent().unwrap()).expect("intent dir");
        std::fs::write(&intent_path, "# Intent\n\n- rendered item\n").expect("intent");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("leaf"),
                Some("Learn"),
                Some("① Intent"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let buffer = render_buffer(120, 18, &app);
        let text = buffer_to_text(&buffer, 120, 18);

        assert!(text.contains("leaf review"));
        assert!(text.contains("READ ONLY - edit originals"));
        assert!(text.contains(".leaf/02-leaves/demo/00-status.md"));
        assert!(line_contains_text(&buffer, 120, 18, "Leaf 상태"));
        assert!(text.contains("# Intent"));
        assert!(text.contains("• rendered item"));

        let (heading_x, heading_y) =
            text_position(&buffer, 120, 18, "# Leaf").expect("h1 heading position");
        let heading_cell = &buffer[(heading_x, heading_y)];
        assert!(heading_cell.style().add_modifier.contains(Modifier::BOLD));
        assert!(
            heading_cell
                .style()
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
    }

    #[test]
    fn review_mode_manual_refresh_renders_refreshed_status_message() {
        let fixture = RenderFixture::new();
        let slug = "refresh-demo";
        let leaf_path = fixture
            .root
            .path()
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug);
        let status_path = leaf_path.join("00-status.md");
        std::fs::create_dir_all(&leaf_path).expect("leaf dir");
        std::fs::write(&status_path, "# Status\n\nold status\n").expect("old status");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("-")),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        std::fs::write(&status_path, "# Status\n\nnew status\n").expect("new status");
        assert_eq!(app.handle_key(KeyInput::Char('r')), Outcome::Continue);

        let text = buffer_text(120, 18, &app);

        assert!(text.contains("new status"));
        assert!(text.contains("refreshed from source"));
    }

    #[test]
    fn review_hyperlink_target_maps_visible_body_coordinates_to_safe_destinations() {
        let fixture = RenderFixture::new();
        let slug = "link-demo";
        let root = fixture.root.path();
        let leaf_path = root
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug);
        std::fs::create_dir_all(leaf_path.join("01-Learn")).expect("intent dir");
        std::fs::write(
            leaf_path.join("00-status.md"),
            "# Status\n\n- current gate: ① Intent\n",
        )
        .expect("status");
        std::fs::write(
            leaf_path.join("01-Learn/01-intent.md"),
            "See [docs](https://example.com/docs) today.\n",
        )
        .expect("intent");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("leaf"),
                Some("Learn"),
                Some("① Intent"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let width = 100;
        let height = 18;
        let area = Rect::new(0, 0, width, height);
        let buffer = render_buffer(width, height, &app);
        let (x, y) = text_position(&buffer, width, height, "docs").expect("visible link text");

        assert_eq!(
            review_hyperlink_target(area, &app, x, y).as_deref(),
            Some("https://example.com/docs")
        );
        assert_eq!(
            review_hyperlink_target(area, &app, x + "docs".len() as u16 - 1, y).as_deref(),
            Some("https://example.com/docs")
        );
        assert_eq!(
            review_hyperlink_target(area, &app, x.saturating_sub(1), y),
            None
        );
        assert_eq!(review_hyperlink_target(area, &app, x, 0), None);
    }

    #[test]
    fn review_hyperlink_target_rejects_unsafe_destinations() {
        let fixture = RenderFixture::new();
        let slug = "unsafe-link";
        let root = fixture.root.path();
        let leaf_path = root
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug);
        std::fs::create_dir_all(leaf_path.join("01-Learn")).expect("intent dir");
        std::fs::write(
            leaf_path.join("00-status.md"),
            "# Status\n\n- current gate: ① Intent\n",
        )
        .expect("status");
        std::fs::write(
            leaf_path.join("01-Learn/01-intent.md"),
            "Open [bad](javascript:alert(1)) link.\n",
        )
        .expect("intent");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("leaf"),
                Some("Learn"),
                Some("① Intent"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let width = 80;
        let height = 18;
        let area = Rect::new(0, 0, width, height);
        let buffer = render_buffer(width, height, &app);
        let (x, y) = text_position(&buffer, width, height, "bad").expect("visible link text");

        assert_eq!(review_hyperlink_target(area, &app, x, y), None);
    }

    #[test]
    fn review_mode_renders_complex_markdown_through_pulldown_pipeline() {
        let fixture = RenderFixture::new();
        let slug = "complex-review";
        let root = fixture.root.path();
        let leaf_path = root
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug);
        std::fs::create_dir_all(leaf_path.join("01-Learn")).expect("intent dir");
        std::fs::write(
            leaf_path.join("00-status.md"),
            "# Status\n\n- current gate: ① Intent\n",
        )
        .expect("status");
        std::fs::write(
            leaf_path.join("01-Learn/01-intent.md"),
            r#"# Intent

Intro with **bold**, `code`, and [docs](https://example.com/docs).

> Quote [guide](./notes/guide.md#L3C2-L4C8)
> - nested [web](https://example.com/a)
> ```rust
> fn main() {}
> ```
> | Plain check | EARS |
> | --- | --- |
> | fallen reason | WHEN an item enters fallen, THE MODEL SHALL record a narrow removal reason. |
"#,
        )
        .expect("intent");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("leaf"),
                Some("Learn"),
                Some("① Intent"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let width = 104;
        let height = 34;
        let area = Rect::new(0, 0, width, height);
        let buffer = render_buffer(width, height, &app);
        let text = buffer_to_text(&buffer, width, height);

        assert!(text.contains("┌"), "{text}");
        assert!(line_contains_text(&buffer, width, height, "Intent"));
        assert!(text.contains("Intro with bold, code, and docs"), "{text}");
        assert!(text.contains("│ Quote ./notes/guide.md:3:2-4:8"), "{text}");
        assert!(
            text.contains("│ • nested web (https://example.com/a)"),
            "{text}"
        );
        assert!(text.contains("│ fn main() {}"), "{text}");
        assert!(text.contains("│ Plain check"), "{text}");
        assert!(text.contains("fallen reason"), "{text}");
        assert!(text.contains("WHEN an item enters fallen"), "{text}");
        assert!(!text.contains("**bold**"), "{text}");
        assert!(!text.contains("```rust"), "{text}");
        assert!(!text.contains("| Plain check | EARS |"), "{text}");

        let (docs_x, docs_y) =
            text_position(&buffer, width, height, "docs").expect("visible web link text");
        assert_eq!(
            review_hyperlink_target(area, &app, docs_x, docs_y).as_deref(),
            Some("https://example.com/docs")
        );

        let (guide_x, guide_y) = text_position(&buffer, width, height, "./notes/guide.md:3:2-4:8")
            .expect("visible local link text");
        assert_eq!(
            review_hyperlink_target(area, &app, guide_x, guide_y).as_deref(),
            Some("./notes/guide.md#L3C2-L4C8")
        );
    }

    #[test]
    fn review_mode_renders_markdown_table_as_aligned_terminal_lines() {
        let fixture = RenderFixture::new();
        let slug = "table-demo";
        let root = fixture.root.path();
        let leaf_path = root
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug);
        std::fs::create_dir_all(leaf_path.join("02-Example")).expect("criteria dir");
        std::fs::write(
            leaf_path.join("00-status.md"),
            "# Status\n\n- current gate: ③ Criteria\n",
        )
        .expect("status");
        std::fs::write(
            leaf_path.join("02-Example/03-criteria.md"),
            "\
# Criteria

| Plain check | EARS |
| --- | --- |
| fallen needs narrow reason | WHEN an item enters fallen, THE MODEL SHALL record a narrowly scoped removal reason. |
| active leaf keeps source | WHEN an item is active, THE MODEL SHALL keep source documents reviewable. |
",
        )
        .expect("criteria");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("leaf"),
                Some("Example"),
                Some("③ Criteria"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let buffer = render_buffer(150, 30, &app);
        let text = buffer_to_text(&buffer, 150, 30);

        assert!(text.contains("Plain check                   EARS"));
        assert!(text.contains("─────────────────────────────"));
        assert!(
            text.contains(
                "fallen needs narrow reason    WHEN an item enters fallen, THE MODEL SHALL"
            )
        );
        assert!(
            text.contains("active leaf keeps source      WHEN an item is active, THE MODEL SHALL")
        );
        assert!(!text.contains("| Plain check | EARS |"));
    }

    #[test]
    fn review_mode_wraps_markdown_table_cells_to_terminal_width() {
        let fixture = RenderFixture::new();
        let slug = "narrow-table";
        let root = fixture.root.path();
        let leaf_path = root
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug);
        std::fs::create_dir_all(leaf_path.join("02-Example")).expect("criteria dir");
        std::fs::write(
            leaf_path.join("00-status.md"),
            "# Status\n\n- current gate: ③ Criteria\n",
        )
        .expect("status");
        std::fs::write(
            leaf_path.join("02-Example/03-criteria.md"),
            "\
# Criteria

| Plain check | EARS |
| --- | --- |
| fallen reason is narrow | WHEN an item enters fallen, THE MODEL SHALL record a narrowly scoped removal reason instead of a broad lifecycle outcome term. |
",
        )
        .expect("criteria");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("leaf"),
                Some("Example"),
                Some("③ Criteria"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let width = 64;
        let height = 35;
        let buffer = render_buffer(width, height, &app);
        let text = buffer_to_text(&buffer, width, height);
        let table_lines = text
            .lines()
            .filter(|line| {
                line.contains("fallen reason")
                    || line.contains("WHEN an item")
                    || line.contains("SHALL record")
                    || line.contains("lifecycle")
            })
            .collect::<Vec<_>>();

        assert!(table_lines.len() >= 3, "{text}");
        assert!(
            text.contains(" Plain check  fallen reason is narrow"),
            "{text}"
        );
        assert!(
            text.contains(" EARS         WHEN an item enters fallen"),
            "{text}"
        );
        assert!(text.contains("WHEN an item"));
        assert!(text.contains("lifecycle outcome"));
        assert!(!text.contains("Plain check    EARS"), "{text}");
        assert!(!text.contains("lifecycl\n"));
        assert!(
            table_lines.iter().all(|line| {
                let body = line
                    .trim_end()
                    .strip_prefix('│')
                    .and_then(|line| line.strip_suffix('│'))
                    .unwrap_or(line);
                crate::preview::display_width(body.trim_end()) <= usize::from(width - 2)
            }),
            "{text}"
        );
    }

    #[test]
    fn review_mode_switches_cramped_markdown_table_to_records() {
        let fixture = RenderFixture::new();
        let slug = "record-table";
        let root = fixture.root.path();
        let leaf_path = root
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug);
        std::fs::create_dir_all(leaf_path.join("02-Example")).expect("criteria dir");
        std::fs::write(
            leaf_path.join("00-status.md"),
            "# Status\n\n- current gate: ③ Criteria\n",
        )
        .expect("status");
        std::fs::write(
            leaf_path.join("02-Example/03-criteria.md"),
            "\
# Criteria

| Plain check | EARS |
| --- | --- |
| fallen reason | WHEN an item enters fallen, THE MODEL SHALL record a narrowly scoped removal reason instead of a broad lifecycle outcome term. |
| active source | WHEN an item is active, THE MODEL SHALL keep source documents reviewable. |
",
        )
        .expect("criteria");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("leaf"),
                Some("Example"),
                Some("③ Criteria"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let width = 46;
        let buffer = render_buffer(width, 40, &app);
        let text = buffer_to_text(&buffer, width, 40);

        assert!(text.contains(" Plain check  fallen reason"), "{text}");
        assert!(
            text.contains(" EARS         WHEN an item enters fallen"),
            "{text}"
        );
        assert!(text.contains(" active source"), "{text}");
        assert!(!text.contains("Plain check    EARS"), "{text}");
    }

    #[test]
    fn review_mode_footer_renders_native_drag_copy_contract() {
        let fixture = RenderFixture::new();
        let slug = "demo";
        let status_path = fixture
            .root
            .path()
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug)
            .join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("leaf dir");
        std::fs::write(&status_path, "# Status\n\n- current gate: ① Intent\n").expect("status");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("leaf"),
                Some("Learn"),
                Some("① Intent"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let text = buffer_text(140, 18, &app);

        assert!(text.contains("drag select/copy text"));
        assert!(text.contains("r refresh"));
        assert!(text.contains("Esc/q back"));
        assert!(!text.contains("Shift/Opt-drag copy"));
        assert!(!text.contains("wheel scroll"));
        assert!(!text.contains("auto-watch"));
    }

    #[test]
    fn review_mode_small_terminal_renders_without_panicking() {
        let fixture = RenderFixture::new();
        let slug = "demo";
        let status_path = fixture
            .root
            .path()
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug)
            .join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("leaf dir");
        std::fs::write(&status_path, "# Status\n\n- current gate: ① Intent\n").expect("status");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("leaf"),
                Some("Learn"),
                Some("① Intent"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let text = buffer_text(50, 8, &app);

        assert!(text.contains("READ ONLY"));
    }

    #[test]
    fn review_mode_short_document_scroll_down_keeps_full_first_page_visible() {
        let fixture = RenderFixture::new();
        let slug = "demo";
        let status_path = fixture
            .root
            .path()
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug)
            .join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("leaf dir");
        std::fs::write(&status_path, "# Status\n\nshort body\n").expect("status");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("-")),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        for _ in 0..12 {
            assert_eq!(app.handle_key(KeyInput::PageDown), Outcome::Continue);
        }

        let text = buffer_text(80, 12, &app);

        assert!(text.contains(".leaf/02-leaves/demo/00-status.md"));
        assert!(text.contains("Status"));
        assert!(text.contains("short body"));
    }

    #[test]
    fn review_mode_small_terminal_can_scroll_to_final_line_past_ten_line_page() {
        let fixture = RenderFixture::new();
        let slug = "demo";
        let status_path = fixture
            .root
            .path()
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug)
            .join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("leaf dir");
        let body = (1..=16)
            .map(|line| format!("status line {line:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&status_path, format!("# Status\n\n{body}\n")).expect("status");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("-")),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let _ = buffer_text(80, 9, &app);
        assert_eq!(app.handle_key(KeyInput::Char('G')), Outcome::Continue);

        let text = buffer_text(80, 9, &app);

        assert!(text.contains("status line 16"));
    }

    #[test]
    fn review_mode_small_terminal_up_from_bottom_moves_visible_page() {
        let fixture = RenderFixture::new();
        let slug = "demo";
        let status_path = fixture
            .root
            .path()
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug)
            .join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("leaf dir");
        let body = (1..=16)
            .map(|line| format!("status line {line:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&status_path, format!("# Status\n\n{body}\n")).expect("status");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            StageDir::Leaves,
            slug,
            status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("-")),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let _ = buffer_text(80, 9, &app);
        assert_eq!(app.handle_key(KeyInput::Char('G')), Outcome::Continue);
        let bottom = buffer_text(80, 9, &app);
        assert!(bottom.contains("status line 16"));
        let bottom_offset = app.review_state().unwrap().scroll_offset;

        assert_eq!(app.handle_key(KeyInput::Up), Outcome::Continue);
        let after_up = buffer_text(80, 9, &app);

        assert_eq!(
            app.review_state().unwrap().scroll_offset,
            bottom_offset.saturating_sub(1)
        );
        assert_ne!(after_up, bottom);
    }

    struct RenderFixture {
        root: assert_fs::TempDir,
    }

    impl RenderFixture {
        fn new() -> Self {
            Self {
                root: assert_fs::TempDir::new().expect("temp repo"),
            }
        }

        fn inventory_with_items(&self, items: Vec<InventoryItem>) -> Inventory {
            let mut sprouts = Vec::new();
            let mut leaves = Vec::new();
            let mut fallen = Vec::new();
            let mut pressed = Vec::new();

            for item in items {
                match item.stage_dir {
                    StageDir::Sprouts => sprouts.push(item),
                    StageDir::Leaves => leaves.push(item),
                    StageDir::Fallen => fallen.push(item),
                    StageDir::Pressed => pressed.push(item),
                }
            }

            Inventory {
                leaf_root: self.root.path().join(".leaf"),
                stages: vec![
                    StageInventory {
                        stage_dir: StageDir::Sprouts,
                        items: sprouts,
                    },
                    StageInventory {
                        stage_dir: StageDir::Leaves,
                        items: leaves,
                    },
                    StageInventory {
                        stage_dir: StageDir::Fallen,
                        items: fallen,
                    },
                    StageInventory {
                        stage_dir: StageDir::Pressed,
                        items: pressed,
                    },
                ],
            }
        }

        fn plain_leaf(&self, slug: &str) -> InventoryItem {
            self.leaf_item(
                StageDir::Leaves,
                slug,
                status(ParseState::Ok, Some("leaf"), Some("Learn"), Some("intent")),
            )
        }

        fn leaf_item(
            &self,
            stage_dir: StageDir,
            slug: &str,
            status: StatusSummary,
        ) -> InventoryItem {
            let path = self
                .root
                .path()
                .join(".leaf")
                .join(stage_dir_path(stage_dir))
                .join(slug);
            let status_path = path.join("00-status.md");
            std::fs::create_dir_all(status_path.parent().unwrap()).unwrap();
            if !status_path.exists() {
                std::fs::write(
                    &status_path,
                    "# 상태\n\n- next action: 다음 행동을 정리한다.\n",
                )
                .unwrap();
            }

            InventoryItem {
                stage_dir,
                slug: slug.to_string(),
                kind: ItemKind::LeafWork,
                path: path.clone(),
                status,
                preview: PreviewSource::LeafWork {
                    status_path,
                    intent_path: path.join("01-Learn/01-intent.md"),
                    unknowns_path: path.join("01-Learn/02-unknowns.md"),
                    criteria_path: path.join("02-Example/03-criteria.md"),
                },
                review: Some(crate::review::ReviewSource::LeafWork {
                    root_path: path,
                    root_relative_path: format!(".leaf/{}/{slug}", stage_dir_path(stage_dir)),
                }),
            }
        }
    }

    fn status(
        parse_state: ParseState,
        stage: Option<&str>,
        current_phase: Option<&str>,
        current_gate: Option<&str>,
    ) -> StatusSummary {
        StatusSummary {
            parse_state,
            stage: stage.map(str::to_string),
            legacy_state: None,
            fallen_reason: None,
            current_phase: current_phase.map(str::to_string),
            current_gate: current_gate.map(str::to_string),
            first_missing_gate: None,
            next_action: None,
            missing_fields: Vec::new(),
        }
    }

    fn stage_dir_path(stage_dir: StageDir) -> &'static str {
        stage_dir.dir_name()
    }
}
