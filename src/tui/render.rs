use crate::inventory::{Bucket, ParseState};
use crate::preview::{PreviewLine, PreviewSpan};
use crate::tui::app::{AppState, BucketFilter, ListRow, Mode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

const PREVIEW_MIN_HEIGHT: u16 = 14;

pub(crate) fn draw(frame: &mut Frame<'_>, app: &AppState) {
    let area = frame.area();
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

fn draw_header(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let visible_count = app.visible_rows().len();
    let total_count = app.rows().len();
    let filter = if app.filter().is_empty() {
        "none".to_string()
    } else {
        app.filter().to_string()
    };
    let line = Line::from(vec![
        Span::styled("leaf list", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  bucket "),
        Span::styled(bucket_filter_label(app.active_bucket()), chrome_style()),
        Span::raw(format!(
            "  filter {filter}  rows {visible_count}/{total_count}"
        )),
    ]);
    frame.render_widget(Paragraph::new(line), area);
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
        .map(|(index, row)| table_row(row).style(row_style(app, index)));

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Length(14),
            Constraint::Length(18),
            Constraint::Min(18),
            Constraint::Length(8),
        ],
    )
    .header(Row::new(["BUCKET", "STATE", "PHASE", "GATE", "SLUG", "STATUS"]).style(chrome_style()))
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
    let status = if app.mode() == Mode::FilterInput {
        format!(
            "filter: {}  Esc list  Backspace delete  {}",
            app.filter(),
            app.status_line()
        )
    } else {
        format!(
            "j/k up/down  h/l bucket  / filter  p preview  q quit  {}",
            app.status_line()
        )
    };
    frame.render_widget(Paragraph::new(Line::styled(status, dim_style())), area);
}

fn table_row(row: &ListRow) -> Row<'_> {
    Row::new(vec![
        Cell::from(row.bucket_label().to_string()).style(bucket_style(row.bucket())),
        Cell::from(row.state().to_string()),
        Cell::from(row.phase().to_string()),
        Cell::from(row.gate().to_string()),
        Cell::from(row.slug().to_string()),
        Cell::from(parse_state_label(row.parse_state()).to_string())
            .style(parse_state_style(row.parse_state())),
    ])
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
    if app.selected_index() == index {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    } else {
        Style::default()
    }
}

fn preview_height(area: Rect) -> u16 {
    area.height.saturating_sub(8).clamp(5, 9)
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
        PreviewLine::Code(text) => Line::styled(text.clone(), code_style()),
        PreviewLine::Styled(spans) => Line::from(
            spans
                .iter()
                .map(|span| match span {
                    PreviewSpan::Plain(text) => Span::raw(text.clone()),
                    PreviewSpan::Bold(text) => Span::styled(text.clone(), strong_style()),
                    PreviewSpan::Code(text) => Span::styled(text.clone(), code_style()),
                })
                .collect::<Vec<_>>(),
        ),
        PreviewLine::Plain(text) => Line::from(text.clone()),
    }
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
    use crate::tui::app::{AppState, KeyInput};
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

        let buffer = render_buffer(110, 24, &app);
        let text = buffer_to_text(&buffer, 110, 24);

        assert!(text.contains("leaf list"));
        assert!(text.contains("BUCKET"));
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
    fn empty_inventory_renders_without_panicking() {
        let fixture = RenderFixture::new();
        let inventory = fixture.inventory_with_items(Vec::new());
        let app = AppState::from_inventory(&inventory);

        let text = buffer_text(50, 6, &app);

        assert!(text.contains("No leaf items"));
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

        fn leaf_item(&self, bucket: Bucket, slug: &str, status: StatusSummary) -> InventoryItem {
            let path = self
                .root
                .path()
                .join(".leaf")
                .join(bucket_dir(bucket))
                .join(slug);
            let status_path = path.join("00-status.md");
            std::fs::create_dir_all(status_path.parent().unwrap()).unwrap();
            std::fs::write(
                &status_path,
                "# 상태\n\n- next action: 다음 행동을 정리한다.\n",
            )
            .unwrap();

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
