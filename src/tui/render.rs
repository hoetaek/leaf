use crate::inventory::{Bucket, ParseState};
use crate::list_columns::{ColumnWidth, LIST_COLUMNS, ListColumn};
use crate::preview::{PreviewLine, PreviewSpan};
use crate::review::{ReviewDocument, ReviewLine};
use crate::tui::app::{AppState, BucketFilter, ListRow, Mode};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

const PREVIEW_MIN_HEIGHT: u16 = 14;
const LIST_HEADER_HEIGHT: u16 = 2;
const HEADER_SPLIT_MIN_WIDTH: u16 = 60;
const HEADER_SUMMARY_WIDTH: u16 = 24;
const RIGHT_PREVIEW_RATIO: f32 = 0.45;
const BOTTOM_PREVIEW_RATIO: f32 = 0.40;
const MIN_TABLE_WIDTH_FOR_RIGHT_PREVIEW: u16 = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewPlacement {
    Hidden,
    Right,
    Bottom,
}

pub(crate) fn draw(frame: &mut Frame<'_>, app: &AppState) {
    let area = frame.area();
    if app.mode() == Mode::Review {
        draw_review(frame, area, app);
        return;
    }

    let chunks = Layout::vertical([
        Constraint::Length(LIST_HEADER_HEIGHT),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);
    draw_header(frame, chunks[0], app);

    match preview_placement(area, app) {
        PreviewPlacement::Hidden => draw_table(frame, chunks[1], app),
        PreviewPlacement::Bottom => {
            let body_chunks = Layout::vertical([
                Constraint::Min(1),
                Constraint::Length(bottom_preview_height(chunks[1])),
            ])
            .split(chunks[1]);
            draw_table(frame, body_chunks[0], app);
            draw_preview(frame, body_chunks[1], app);
        }
        PreviewPlacement::Right => {
            let preview_width = right_preview_width(area);
            let body_chunks = Layout::horizontal([
                Constraint::Min(MIN_TABLE_WIDTH_FOR_RIGHT_PREVIEW),
                Constraint::Length(preview_width),
            ])
            .split(chunks[1]);
            draw_table(frame, body_chunks[0], app);
            draw_preview(frame, body_chunks[1], app);
        }
    }
    draw_status(frame, chunks[2], app);
}

fn draw_review(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let Some(review) = app.review_state() else {
        frame.render_widget(Paragraph::new("No review document loaded."), area);
        return;
    };

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);
    app.set_review_body_size(chunks[3].height as usize, chunks[3].width as usize);

    let document = &review.document;
    let header = Line::from(vec![
        Span::styled("leaf review", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(document.title.clone(), strong_style()),
        Span::raw("  "),
        Span::styled(document.root_relative_path.clone(), dim_style()),
    ]);
    frame.render_widget(Paragraph::new(header), chunks[0]);

    frame.render_widget(
        Paragraph::new(Line::styled(
            "READ ONLY - edit originals",
            strong_style().fg(Color::Yellow),
        )),
        chunks[1],
    );

    let rendered_body_lines = wrapped_review_lines(document, chunks[3].width);
    let scroll_offset =
        clamped_review_scroll_offset(&rendered_body_lines, chunks[3].height, review.scroll_offset);
    let current_source = current_source_index(&rendered_body_lines, scroll_offset);
    let scroll_percent =
        review_scroll_percent(&rendered_body_lines, chunks[3].height, scroll_offset);
    let meta = format!(
        "source {}/{}  scroll {}%  {}",
        current_source, document.source_count, scroll_percent, review.status_message
    );
    frame.render_widget(Paragraph::new(Line::styled(meta, dim_style())), chunks[2]);

    let body_lines = rendered_body_lines
        .iter()
        .skip(scroll_offset)
        .take(chunks[3].height as usize)
        .map(|line| line.line.clone())
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(body_lines), chunks[3]);

    frame.render_widget(
        Paragraph::new(Line::styled(
            "↑/↓/wheel scroll  d/u half  PgUp/PgDn  g/G top/bottom  r refresh  Esc/q back",
            dim_style(),
        )),
        chunks[4],
    );
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
            Span::styled(bucket_filter_label(app.active_bucket()), chrome_style()),
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

    let visible_rows = app.visible_rows();
    if visible_rows.is_empty() {
        let empty = Paragraph::new("No leaf items match the current view.")
            .block(chrome_block().title("Inventory"));
        frame.render_widget(empty, area);
        return;
    }

    let row_capacity = table_row_capacity(area);
    let offset = row_viewport_offset(app.selected_index(), row_capacity);
    let rows = visible_rows
        .into_iter()
        .enumerate()
        .skip(offset)
        .take(row_capacity)
        .map(|(index, row)| table_row(row, row_is_active(app, index)).style(row_style(app, index)));

    let table = Table::new(rows, table_constraints())
        .header(Row::new(table_header()).style(chrome_style()))
        .column_spacing(1)
        .block(chrome_block().title("Inventory"));
    frame.render_widget(table, area);
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
            lines.extend(preview.lines.iter().map(preview_line));
            (format!("Preview {}", row.slug()), lines)
        }
        None => (
            "Preview".to_string(),
            vec![Line::from("No leaf item selected.")],
        ),
    };

    frame.render_widget(
        Paragraph::new(lines).block(chrome_block().title(title)),
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
        Mode::ConfirmPromote => app.status_line().to_string(),
        Mode::RangeSelect => format!(
            "range {selected_count} selected  j/k extend  v/Esc done  y copy  q quit  {}",
            app.status_line()
        ),
        Mode::Review => {
            "↑/↓/wheel scroll  d/u half  PgUp/PgDn  g/G top/bottom  r refresh  Esc/q back"
                .to_string()
        }
        Mode::List if selected_count > 0 => format!(
            "{selected_count} selected  Space toggle  v range  a all  y copy  Esc clear  q quit  {}",
            app.status_line()
        ),
        Mode::List => format!(
            "j/k up/down  h/l bucket  y copy  P promote  Space select  v range  a all  / filter  p preview  r refresh  q quit  mouse drag  {}",
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
        ListColumn::Bucket => cell.style(bucket_style(row.bucket())),
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
    let chunks = Layout::vertical([
        Constraint::Length(LIST_HEADER_HEIGHT),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);
    let body = chunks[1];
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

    let inner_left = table.x + 1;
    let inner_right = table.x + table.width - 2;
    if column < inner_left || column > inner_right {
        return None;
    }

    let first_data_row = table.y + 2;
    if row < first_data_row {
        return None;
    }
    let data_row_index = (row - first_data_row) as usize;
    let row_capacity = table_row_capacity(table);
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
    area.height.saturating_sub(3) as usize
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
        (true, true) => Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
        (true, false) => Style::default().bg(Color::DarkGray).fg(Color::White),
        (false, true) => Style::default().bg(Color::Blue).fg(Color::White),
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
    match line {
        PreviewLine::Heading(text) => Line::styled(text.clone(), strong_style()),
        PreviewLine::Checkbox { checked, text } => {
            let marker = if *checked { "[x]" } else { "[ ]" };
            Line::from(vec![
                Span::styled(marker, chrome_style()),
                Span::raw(" "),
                Span::raw(text.clone()),
            ])
        }
        PreviewLine::ListItem { marker, spans } => {
            let mut rendered = vec![Span::styled(marker.clone(), chrome_style()), Span::raw(" ")];
            rendered.extend(spans.iter().map(preview_span));
            Line::from(rendered)
        }
        PreviewLine::Code(text) => Line::styled(text.clone(), code_style()),
        PreviewLine::Styled(spans) => {
            Line::from(spans.iter().map(preview_span).collect::<Vec<_>>())
        }
        PreviewLine::SourceBoundary {
            phase,
            gate,
            source,
        } => source_boundary_line(None, phase, gate, source),
        PreviewLine::TableHeader { .. } => Line::styled(
            crate::preview::table_line_text(line).expect("table line text"),
            strong_style(),
        ),
        PreviewLine::TableDivider { .. } => Line::styled(
            crate::preview::table_line_text(line).expect("table line text"),
            chrome_style(),
        ),
        PreviewLine::TableRow { .. } => {
            Line::from(crate::preview::table_line_text(line).expect("table line text"))
        }
        PreviewLine::Plain(text) => Line::from(text.clone()),
    }
}

fn preview_span(span: &PreviewSpan) -> Span<'static> {
    match span {
        PreviewSpan::Plain(text) => Span::raw(text.clone()),
        PreviewSpan::Bold(text) => Span::styled(text.clone(), strong_style()),
        PreviewSpan::Code(text) => Span::styled(text.clone(), code_style()),
    }
}

fn review_line(line: &ReviewLine) -> Line<'static> {
    match line {
        ReviewLine::Separator {
            relative_path,
            phase,
            gate,
        } => source_boundary_line(Some("FILE"), phase, gate, relative_path),
        ReviewLine::MissingSource { relative_path } => Line::from(vec![
            Span::styled("MISSING SOURCE", Style::default().fg(Color::Red)),
            Span::raw(" "),
            Span::styled(relative_path.clone(), dim_style()),
        ]),
        ReviewLine::Markdown(line) => preview_line(line),
        ReviewLine::Message(text) => Line::from(text.clone()),
    }
}

#[derive(Debug, Clone)]
struct RenderedReviewLine {
    source_index: usize,
    line: Line<'static>,
}

fn wrapped_review_lines(document: &ReviewDocument, width: u16) -> Vec<RenderedReviewLine> {
    let width = usize::from(width.max(1));
    let mut rendered = Vec::new();
    let mut section_index = 0;

    for (line_index, line) in document.lines.iter().enumerate() {
        while section_index + 1 < document.sections.len()
            && document.sections[section_index + 1].start_line <= line_index
        {
            section_index += 1;
        }
        let source_index = if document.sections.is_empty() {
            0
        } else {
            section_index + 1
        };
        if let Some(table_lines) = review_table_lines(line, width) {
            rendered.extend(
                table_lines
                    .into_iter()
                    .map(|line| RenderedReviewLine { source_index, line }),
            );
        } else {
            rendered.extend(
                wrap_line(review_line(line), width)
                    .into_iter()
                    .map(|line| RenderedReviewLine { source_index, line }),
            );
        }
    }

    rendered
}

fn review_table_lines(line: &ReviewLine, width: usize) -> Option<Vec<Line<'static>>> {
    let ReviewLine::Markdown(line) = line else {
        return None;
    };
    let table_lines = crate::preview::wrapped_table_line_texts(line, width)?;
    let style = match line {
        PreviewLine::TableHeader { .. } => strong_style(),
        PreviewLine::TableDivider { .. } => chrome_style(),
        PreviewLine::TableRow { .. } => Style::default(),
        _ => return None,
    };
    Some(
        table_lines
            .into_iter()
            .map(|line| Line::styled(line, style))
            .collect(),
    )
}

fn wrap_line(line: Line<'static>, width: usize) -> Vec<Line<'static>> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut current = Vec::new();
    let mut current_width = 0;

    for span in line.spans {
        let style = span.style;
        let mut chunk = String::new();
        for ch in span.content.chars() {
            let char_width = crate::review::terminal_char_width(ch);
            if current_width > 0 && current_width + char_width > width {
                if !chunk.is_empty() {
                    current.push(Span::styled(std::mem::take(&mut chunk), style));
                }
                lines.push(Line::from(std::mem::take(&mut current)));
                current_width = 0;
            }
            chunk.push(ch);
            current_width += char_width;
        }
        if !chunk.is_empty() {
            current.push(Span::styled(chunk, style));
        }
    }

    lines.push(Line::from(current));
    lines
}

fn source_boundary_line(
    prefix: Option<&str>,
    phase: &str,
    gate: &str,
    source: &str,
) -> Line<'static> {
    let mut spans = Vec::new();
    if let Some(prefix) = prefix {
        spans.push(Span::styled(
            format!("{prefix} "),
            strong_style().fg(Color::Yellow),
        ));
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

fn current_source_index(lines: &[RenderedReviewLine], scroll_offset: usize) -> usize {
    lines
        .get(scroll_offset)
        .or_else(|| lines.last())
        .map(|line| line.source_index)
        .unwrap_or(0)
}

fn review_scroll_percent(
    lines: &[RenderedReviewLine],
    body_height: u16,
    scroll_offset: usize,
) -> usize {
    let max_scroll = max_review_scroll_for_body(lines, body_height);
    scroll_offset
        .min(max_scroll)
        .saturating_mul(100)
        .checked_div(max_scroll)
        .unwrap_or(0)
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

fn bucket_filter_label(filter: BucketFilter) -> &'static str {
    match filter {
        BucketFilter::All => "all",
        BucketFilter::Bucket(Bucket::Seeds) => "seeds",
        BucketFilter::Bucket(Bucket::Leaves) => "leaves",
        BucketFilter::Bucket(Bucket::Fallen) => "fallen",
        BucketFilter::Bucket(Bucket::Pressed) => "pressed",
    }
}

fn parse_state_style(state: ParseState) -> Style {
    match state {
        ParseState::Ok => Style::default().fg(Color::Green),
        ParseState::Partial => Style::default().fg(Color::Yellow),
        ParseState::Error => Style::default().fg(Color::Red),
    }
}

fn bucket_style(bucket: Bucket) -> Style {
    match bucket {
        Bucket::Seeds => Style::default().fg(Color::Cyan),
        Bucket::Leaves => Style::default().fg(Color::Green),
        Bucket::Fallen => Style::default().fg(Color::Magenta),
        Bucket::Pressed => Style::default().fg(Color::Blue),
    }
}

fn chrome_style() -> Style {
    Style::default().fg(Color::Gray)
}

fn dim_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn strong_style() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

fn code_style() -> Style {
    Style::default().fg(Color::Cyan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{
        Bucket, BucketInventory, Inventory, InventoryItem, ItemKind, ParseState, PreviewSource,
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

    fn buffer_line(buffer: &Buffer, width: u16, y: u16) -> String {
        (0..width)
            .map(|x| buffer[(x, y)].symbol().to_string())
            .collect::<String>()
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

    fn text_has_style(
        buffer: &Buffer,
        width: u16,
        height: u16,
        text: &str,
        fg: Color,
        bg: Color,
    ) -> bool {
        (0..height).any(|y| {
            (0..width).any(|x| {
                let mut cursor = x;
                for ch in text.chars() {
                    if cursor >= width {
                        return false;
                    }
                    let cell = &buffer[(cursor, y)];
                    if cell.symbol() != ch.to_string() || cell.fg != fg || cell.bg != bg {
                        return false;
                    }
                    cursor = cursor.saturating_add(cell_width(ch));
                }
                true
            })
        })
    }

    fn cell_width(ch: char) -> u16 {
        if ch.is_ascii() { 1 } else { 2 }
    }

    #[test]
    fn renders_header_table_preview_and_status() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            "korean-preview",
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("intent"),
            ),
        )]);
        let app = AppState::from_inventory(&inventory);

        let buffer = render_buffer(120, 24, &app);
        let text = buffer_to_text(&buffer, 120, 24);

        assert!(text.contains("leaf list"));
        assert!(text.contains("BUCKET"));
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
            Bucket::Leaves,
            "section-header",
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("intent"),
            ),
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
            !first_line.contains("bucket "),
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
    fn wide_terminal_places_preview_on_the_right() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            "wide-preview",
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("intent"),
            ),
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
            Bucket::Leaves,
            "bottom-preview",
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("intent"),
            ),
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
            Bucket::Leaves,
            "compact",
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("intent"),
            ),
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
            Bucket::Leaves,
            "alpha",
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("intent"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        app.handle_key(KeyInput::Char('/'));
        app.handle_key(KeyInput::Char('a'));

        let text = buffer_text(80, 12, &app);

        assert!(text.contains("filter: a"));
    }

    #[test]
    fn normal_status_renders_promote_hint() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Seeds,
            "draft",
            status(ParseState::Ok, Some("seed"), Some("Learn"), Some("-")),
        )]);
        let app = AppState::from_inventory(&inventory);

        let text = buffer_text(90, 12, &app);

        assert!(text.contains("P promote"));
    }

    #[test]
    fn normal_status_renders_refresh_hint() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Seeds,
            "draft",
            status(ParseState::Ok, Some("seed"), Some("Learn"), Some("-")),
        )]);
        let app = AppState::from_inventory(&inventory);

        let text = buffer_text(110, 12, &app);

        assert!(text.contains("r refresh"));
    }

    #[test]
    fn normal_status_renders_copy_hint() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            "alpha",
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("intent"),
            ),
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
                line.line
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
    fn normal_status_renders_mouse_drag_hint() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            "alpha",
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("intent"),
            ),
        )]);
        let app = AppState::from_inventory(&inventory);

        let text = buffer_text(140, 12, &app);

        assert!(text.contains("mouse drag"));
    }

    #[test]
    fn normal_status_renders_multi_select_hints() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            "alpha",
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("intent"),
            ),
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
            Bucket::Leaves,
            "alpha",
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("intent"),
            ),
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
            Bucket::Leaves,
            "alpha",
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("intent"),
            ),
        )]);
        let app = AppState::from_inventory(&inventory);

        let buffer = render_buffer(100, 12, &app);

        assert!(text_has_style(
            &buffer,
            100,
            12,
            "leaf",
            Color::White,
            Color::DarkGray
        ));
        assert!(text_has_style(
            &buffer,
            100,
            12,
            "ok",
            Color::White,
            Color::DarkGray
        ));
    }

    #[test]
    fn range_mode_status_renders_extend_copy_and_quit_hints() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            "alpha",
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("intent"),
            ),
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
    fn confirm_promote_status_renders_selected_seed_and_choices() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Seeds,
            "draft",
            status(ParseState::Ok, Some("seed"), Some("Learn"), Some("-")),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        app.handle_key(KeyInput::Char('P'));

        let text = buffer_text(100, 12, &app);

        assert!(text.contains("Promote seed draft?"));
        assert!(text.contains("y confirm"));
        assert!(text.contains("n/Esc cancel"));
    }

    #[test]
    fn table_mouse_target_maps_data_rows_without_selection_column() {
        let fixture = RenderFixture::new();
        let app = AppState::from_inventory(&fixture.inventory_with_items(vec![
            fixture.plain_leaf("alpha"),
            fixture.plain_leaf("beta"),
            fixture.plain_leaf("gamma"),
        ]));

        // Terminal Rect(0,0,80,10): header y=0..1, table y=2 height=7, status y=9.
        // Data rows begin at table.y + 2 = 4.
        let area = Rect::new(0, 0, 80, 10);

        assert_eq!(table_mouse_target(area, &app, 20, 4), Some(0));
        assert_eq!(table_mouse_target(area, &app, 2, 5), Some(1));
        assert_eq!(table_mouse_target(area, &app, 1, 6), Some(2));
        assert_eq!(table_mouse_target(area, &app, 3, 6), Some(2));
        assert_eq!(table_mouse_target(area, &app, 4, 6), Some(2));
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

        // table height 7 -> capacity 4 -> offset = 9 - 3 = 6.
        let area = Rect::new(0, 0, 80, 10);
        assert_eq!(table_mouse_target(area, &app, 20, 4), Some(6));
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

        assert_eq!(table_mouse_target(area, &app, 20, 4), Some(0));
        assert_eq!(table_mouse_target(area, &app, 110, 4), None);
    }

    #[test]
    fn table_mouse_target_ignores_header_border_and_out_of_range() {
        let fixture = RenderFixture::new();
        let app = AppState::from_inventory(&fixture.inventory_with_items(vec![
            fixture.plain_leaf("alpha"),
            fixture.plain_leaf("beta"),
        ]));
        let area = Rect::new(0, 0, 80, 10);

        assert_eq!(table_mouse_target(area, &app, 20, 2), None); // top border
        assert_eq!(table_mouse_target(area, &app, 20, 3), None); // header
        assert_eq!(table_mouse_target(area, &app, 20, 8), None); // bottom border
        assert_eq!(table_mouse_target(area, &app, 20, 9), None); // status line
        assert_eq!(table_mouse_target(area, &app, 20, 6), None); // beyond last row
        assert_eq!(table_mouse_target(area, &app, 0, 4), None); // left border
        assert_eq!(table_mouse_target(area, &app, 79, 4), None); // right border
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
            .join(Bucket::Leaves.dir_name())
            .join(slug)
            .join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("leaf dir");
        std::fs::write(&status_path, "# Leaf 상태\n\n- current gate: ① Intent\n").expect("status");
        let intent_path = root
            .join(".leaf")
            .join(Bucket::Leaves.dir_name())
            .join(slug)
            .join("01-Learn/01-intent.md");
        std::fs::create_dir_all(intent_path.parent().unwrap()).expect("intent dir");
        std::fs::write(&intent_path, "# Intent\n\n- rendered item\n").expect("intent");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("active"),
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
        assert!(text.contains("• rendered item"));
        assert!(!text.contains("# Intent"));
    }

    #[test]
    fn review_mode_renders_markdown_table_as_aligned_terminal_lines() {
        let fixture = RenderFixture::new();
        let slug = "table-demo";
        let root = fixture.root.path();
        let leaf_path = root
            .join(".leaf")
            .join(Bucket::Leaves.dir_name())
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
",
        )
        .expect("criteria");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("active"),
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
        assert!(!text.contains("| Plain check | EARS |"));
    }

    #[test]
    fn review_mode_wraps_markdown_table_cells_to_terminal_width() {
        let fixture = RenderFixture::new();
        let slug = "narrow-table";
        let root = fixture.root.path();
        let leaf_path = root
            .join(".leaf")
            .join(Bucket::Leaves.dir_name())
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
            Bucket::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("active"),
                Some("Example"),
                Some("③ Criteria"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let width = 64;
        let buffer = render_buffer(width, 34, &app);
        let text = buffer_to_text(&buffer, width, 34);
        let table_lines = text
            .lines()
            .filter(|line| {
                line.contains("fallen reason")
                    || line.contains("WHEN an item")
                    || line.contains("SHALL record")
                    || line.contains("lifecycl")
            })
            .collect::<Vec<_>>();

        assert!(table_lines.len() >= 3, "{text}");
        assert!(text.contains("WHEN an item"));
        assert!(text.contains("lifecycl"));
        assert!(
            table_lines
                .iter()
                .all(|line| crate::preview::display_width(line.trim_end()) <= usize::from(width)),
            "{text}"
        );
    }

    #[test]
    fn review_mode_footer_omits_non_interactive_auto_watch_hint() {
        let fixture = RenderFixture::new();
        let slug = "demo";
        let status_path = fixture
            .root
            .path()
            .join(".leaf")
            .join(Bucket::Leaves.dir_name())
            .join(slug)
            .join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("leaf dir");
        std::fs::write(&status_path, "# Status\n\n- current gate: ① Intent\n").expect("status");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("active"),
                Some("Learn"),
                Some("① Intent"),
            ),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let text = buffer_text(140, 18, &app);

        assert!(text.contains("r refresh"));
        assert!(text.contains("Esc/q back"));
        assert!(!text.contains("q quit"));
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
            .join(Bucket::Leaves.dir_name())
            .join(slug)
            .join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("leaf dir");
        std::fs::write(&status_path, "# Status\n\n- current gate: ① Intent\n").expect("status");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            slug,
            status(
                ParseState::Ok,
                Some("active"),
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
            .join(Bucket::Leaves.dir_name())
            .join(slug)
            .join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("leaf dir");
        std::fs::write(&status_path, "# Status\n\nshort body\n").expect("status");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            slug,
            status(ParseState::Ok, Some("active"), Some("Learn"), Some("-")),
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
            .join(Bucket::Leaves.dir_name())
            .join(slug)
            .join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("leaf dir");
        let body = (1..=16)
            .map(|line| format!("status line {line:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&status_path, format!("# Status\n\n{body}\n")).expect("status");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            slug,
            status(ParseState::Ok, Some("active"), Some("Learn"), Some("-")),
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
            .join(Bucket::Leaves.dir_name())
            .join(slug)
            .join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("leaf dir");
        let body = (1..=16)
            .map(|line| format!("status line {line:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&status_path, format!("# Status\n\n{body}\n")).expect("status");
        let inventory = fixture.inventory_with_items(vec![fixture.leaf_item(
            Bucket::Leaves,
            slug,
            status(ParseState::Ok, Some("active"), Some("Learn"), Some("-")),
        )]);
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let _ = buffer_text(80, 9, &app);
        assert_eq!(app.handle_key(KeyInput::Char('G')), Outcome::Continue);
        let bottom = buffer_text(80, 9, &app);
        assert!(bottom.contains("status line 16"));

        assert_eq!(app.handle_key(KeyInput::Up), Outcome::Continue);
        let after_up = buffer_text(80, 9, &app);

        assert!(after_up.contains("status line 11"));
        assert!(!after_up.contains("status line 16"));
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
            let mut seeds = Vec::new();
            let mut leaves = Vec::new();
            let mut fallen = Vec::new();
            let mut pressed = Vec::new();

            for item in items {
                match item.bucket {
                    Bucket::Seeds => seeds.push(item),
                    Bucket::Leaves => leaves.push(item),
                    Bucket::Fallen => fallen.push(item),
                    Bucket::Pressed => pressed.push(item),
                }
            }

            Inventory {
                leaf_root: self.root.path().join(".leaf"),
                buckets: vec![
                    BucketInventory {
                        bucket: Bucket::Seeds,
                        items: seeds,
                    },
                    BucketInventory {
                        bucket: Bucket::Leaves,
                        items: leaves,
                    },
                    BucketInventory {
                        bucket: Bucket::Fallen,
                        items: fallen,
                    },
                    BucketInventory {
                        bucket: Bucket::Pressed,
                        items: pressed,
                    },
                ],
            }
        }

        fn plain_leaf(&self, slug: &str) -> InventoryItem {
            self.leaf_item(
                Bucket::Leaves,
                slug,
                status(
                    ParseState::Ok,
                    Some("active"),
                    Some("Learn"),
                    Some("intent"),
                ),
            )
        }

        fn leaf_item(&self, bucket: Bucket, slug: &str, status: StatusSummary) -> InventoryItem {
            let path = self
                .root
                .path()
                .join(".leaf")
                .join(bucket_dir(bucket))
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
                bucket,
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
                    root_relative_path: format!(".leaf/{}/{slug}", bucket_dir(bucket)),
                }),
            }
        }
    }

    fn status(
        parse_state: ParseState,
        state: Option<&str>,
        current_phase: Option<&str>,
        current_gate: Option<&str>,
    ) -> StatusSummary {
        StatusSummary {
            parse_state,
            state: state.map(str::to_string),
            current_phase: current_phase.map(str::to_string),
            current_gate: current_gate.map(str::to_string),
            first_missing_gate: None,
            next_action: None,
            missing_fields: Vec::new(),
        }
    }

    fn bucket_dir(bucket: Bucket) -> &'static str {
        bucket.dir_name()
    }
}
