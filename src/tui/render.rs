use crate::inventory::{Bucket, ParseState};
use crate::preview::{PreviewLine, PreviewSpan};
use crate::review::{ReviewDocument, ReviewLine};
use crate::tui::app::{AppState, BucketFilter, ListRow, Mode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

const PREVIEW_MIN_HEIGHT: u16 = 14;

pub(crate) fn draw(frame: &mut Frame<'_>, app: &AppState) {
    let area = frame.area();
    if app.mode() == Mode::Review {
        draw_review(frame, area, app);
        return;
    }

    let show_preview = app.preview_open() && area.height >= PREVIEW_MIN_HEIGHT;
    let mut constraints = vec![Constraint::Length(1), Constraint::Min(1)];
    if show_preview {
        constraints.push(Constraint::Length(preview_height(area)));
    }
    constraints.push(Constraint::Length(1));

    let chunks = Layout::vertical(constraints).split(area);
    let mut index = 0;
    draw_header(frame, chunks[index], app);
    index += 1;
    draw_table(frame, chunks[index], app);
    index += 1;
    if show_preview {
        draw_preview(frame, chunks[index], app);
        index += 1;
    }
    draw_status(frame, chunks[index], app);
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
    app.set_review_body_height(chunks[3].height as usize);

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

    let scroll_offset =
        clamped_review_scroll_offset(document, chunks[3].height, review.scroll_offset);
    let current_source = current_source_index(document, scroll_offset);
    let scroll_percent = review_scroll_percent(document, chunks[3].height, scroll_offset);
    let meta = format!(
        "source {}/{}  scroll {}%  {}",
        current_source, document.source_count, scroll_percent, review.status_message
    );
    frame.render_widget(Paragraph::new(Line::styled(meta, dim_style())), chunks[2]);

    let body_lines = document
        .lines
        .iter()
        .skip(scroll_offset)
        .take(chunks[3].height as usize)
        .map(review_line)
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(body_lines), chunks[3]);

    frame.render_widget(
        Paragraph::new(Line::styled(
            "↑/↓ scroll  d/u half  PgUp/PgDn  g/G top/bottom  r refresh  Esc/q back",
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
    let mut spans = vec![
        Span::styled("leaf list", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  bucket "),
        Span::styled(bucket_filter_label(app.active_bucket()), chrome_style()),
        Span::raw(format!(
            "  filter {filter}  rows {visible_count}/{total_count}"
        )),
    ];
    let selected_count = app.selected_row_count();
    if selected_count > 0 {
        spans.push(Span::styled(
            format!("  selected {selected_count}"),
            strong_style(),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
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
        .map(|(index, row)| {
            table_row(row, app.visible_row_is_marked(index)).style(row_style(app, index))
        });

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(18),
            Constraint::Min(18),
            Constraint::Length(8),
        ],
    )
    .header(Row::new(["SEL", "BUCKET", "PHASE", "GATE", "SLUG", "STATUS"]).style(chrome_style()))
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
            "↑/↓ scroll  d/u half  PgUp/PgDn  g/G top/bottom  r refresh  Esc/q back".to_string()
        }
        Mode::List if selected_count > 0 => format!(
            "{selected_count} selected  Space toggle  v range  a all  y copy  Esc clear  q quit  {}",
            app.status_line()
        ),
        Mode::List => format!(
            "j/k up/down  h/l bucket  y copy  P promote  Space select  v range  a all  / filter  p preview  r refresh  q quit  mouse select  {}",
            app.status_line()
        ),
    };
    frame.render_widget(Paragraph::new(Line::styled(status, dim_style())), area);
}

fn table_row(row: &ListRow, marked: bool) -> Row<'_> {
    let marker = if marked { "*" } else { " " };
    Row::new(vec![
        Cell::from(marker).style(strong_style()),
        Cell::from(row.bucket_label().to_string()).style(bucket_style(row.bucket())),
        Cell::from(row.phase().to_string()),
        Cell::from(row.gate().to_string()),
        Cell::from(row.slug().to_string()),
        Cell::from(parse_state_label(row.parse_state()).to_string())
            .style(parse_state_style(row.parse_state())),
    ])
}

/// Computes the table chunk for a full terminal `Rect`, mirroring the layout
/// `draw` uses so mouse hit-testing maps onto the same rows `draw_table` renders.
fn table_chunk(area: Rect, app: &AppState) -> Rect {
    let show_preview = app.preview_open() && area.height >= PREVIEW_MIN_HEIGHT;
    let mut constraints = vec![Constraint::Length(1), Constraint::Min(1)];
    if show_preview {
        constraints.push(Constraint::Length(preview_height(area)));
    }
    constraints.push(Constraint::Length(1));
    Layout::vertical(constraints).split(area)[1]
}

/// Maps a terminal `(column, row)` click onto the visible row it covers.
///
/// Returns `Some((visible_index, is_sel_column))` when the coordinate lands on a
/// data row inside the table, or `None` for the header, borders, status line, or
/// any coordinate outside the rendered rows. Data rows start at `table.y + 2`
/// (top border + header); the `SEL` column spans `table.x + 1..=table.x + 3`.
pub(crate) fn table_mouse_target(
    area: Rect,
    app: &AppState,
    column: u16,
    row: u16,
) -> Option<(usize, bool)> {
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

    // The `SEL` column spans `table.x + 1..=table.x + 3`; the lower bound is
    // already guaranteed by the `inner_left` check above.
    let is_sel_column = column <= table.x + 3;
    Some((visible_index, is_sel_column))
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

fn preview_height(area: Rect) -> u16 {
    (area.height.saturating_sub(2) / 2).clamp(6, 18)
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
        ReviewLine::Separator(path) => Line::from(vec![
            Span::styled("FILE ", strong_style().fg(Color::Yellow)),
            Span::styled(path.clone(), strong_style()),
        ]),
        ReviewLine::MissingSource { relative_path } => Line::from(vec![
            Span::styled("MISSING SOURCE", Style::default().fg(Color::Red)),
            Span::raw(" "),
            Span::styled(relative_path.clone(), dim_style()),
        ]),
        ReviewLine::Markdown(line) => preview_line(line),
        ReviewLine::Message(text) => Line::from(text.clone()),
    }
}

fn current_source_index(document: &ReviewDocument, scroll_offset: usize) -> usize {
    document
        .sections
        .iter()
        .rposition(|section| section.start_line <= scroll_offset)
        .map(|index| index + 1)
        .unwrap_or(0)
}

fn review_scroll_percent(
    document: &ReviewDocument,
    body_height: u16,
    scroll_offset: usize,
) -> usize {
    let max_scroll = max_review_scroll_for_body(document, body_height);
    if max_scroll == 0 {
        0
    } else {
        scroll_offset.min(max_scroll) * 100 / max_scroll
    }
}

fn clamped_review_scroll_offset(
    document: &ReviewDocument,
    body_height: u16,
    scroll_offset: usize,
) -> usize {
    scroll_offset.min(max_review_scroll_for_body(document, body_height))
}

fn max_review_scroll_for_body(document: &ReviewDocument, body_height: u16) -> usize {
    document.lines.len().saturating_sub(body_height as usize)
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

fn parse_state_label(state: ParseState) -> &'static str {
    match state {
        ParseState::Ok => "ok",
        ParseState::Partial => "partial",
        ParseState::Error => "error",
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
    fn large_terminal_allocates_more_space_to_preview() {
        let area = Rect::new(0, 0, 120, 32);

        assert_eq!(preview_height(area), 15);
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
    fn review_separator_renders_as_prominent_file_boundary() {
        let line = review_line(&ReviewLine::Separator(
            ".leaf/02-leaves/demo/01-Learn/01-intent.md".to_string(),
        ));
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert!(rendered.starts_with("FILE "));
        assert!(rendered.contains(".leaf/02-leaves/demo/01-Learn/01-intent.md"));
        assert!(
            line.spans
                .iter()
                .any(|span| span.style.add_modifier.contains(Modifier::BOLD))
        );
    }

    #[test]
    fn normal_status_renders_mouse_select_hint() {
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

        assert!(text.contains("mouse select"));
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
    fn marked_rows_render_a_selection_marker_and_selected_status() {
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
        assert!(before.contains("SEL"));

        app.handle_key(KeyInput::Char(' '));
        let after = buffer_text(120, 14, &app);
        assert!(after.contains('*'));
        assert!(after.contains("1 selected"));
        assert!(after.contains("Space toggle"));
        assert!(after.contains("Esc clear"));
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
    fn table_mouse_target_maps_data_rows_and_sel_column() {
        let fixture = RenderFixture::new();
        let app = AppState::from_inventory(&fixture.inventory_with_items(vec![
            fixture.plain_leaf("alpha"),
            fixture.plain_leaf("beta"),
            fixture.plain_leaf("gamma"),
        ]));

        // Terminal Rect(0,0,80,10): header y=0, table y=1 height=8, status y=9.
        // Data rows begin at table.y + 2 = 3. SEL column spans x=1..=3.
        let area = Rect::new(0, 0, 80, 10);

        assert_eq!(table_mouse_target(area, &app, 20, 3), Some((0, false)));
        assert_eq!(table_mouse_target(area, &app, 2, 4), Some((1, true)));
        assert_eq!(table_mouse_target(area, &app, 1, 5), Some((2, true)));
        assert_eq!(table_mouse_target(area, &app, 3, 5), Some((2, true)));
        assert_eq!(table_mouse_target(area, &app, 4, 5), Some((2, false)));
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

        // table height 8 -> capacity 5 -> offset = 9 - 4 = 5.
        let area = Rect::new(0, 0, 80, 10);
        assert_eq!(table_mouse_target(area, &app, 20, 3), Some((5, false)));
        assert_eq!(table_mouse_target(area, &app, 20, 7), Some((9, false)));
    }

    #[test]
    fn table_mouse_target_ignores_header_border_and_out_of_range() {
        let fixture = RenderFixture::new();
        let app = AppState::from_inventory(&fixture.inventory_with_items(vec![
            fixture.plain_leaf("alpha"),
            fixture.plain_leaf("beta"),
        ]));
        let area = Rect::new(0, 0, 80, 10);

        assert_eq!(table_mouse_target(area, &app, 20, 1), None); // top border
        assert_eq!(table_mouse_target(area, &app, 20, 2), None); // header
        assert_eq!(table_mouse_target(area, &app, 20, 8), None); // bottom border
        assert_eq!(table_mouse_target(area, &app, 20, 9), None); // status line
        assert_eq!(table_mouse_target(area, &app, 20, 5), None); // beyond last row
        assert_eq!(table_mouse_target(area, &app, 0, 3), None); // left border
        assert_eq!(table_mouse_target(area, &app, 79, 3), None); // right border
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
