use crate::inventory::{Bucket, ParseState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListColumn {
    Bucket,
    Phase,
    Gate,
    Slug,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColumnWidth {
    Fixed(u16),
    Min(u16),
}

pub(crate) const LIST_COLUMNS: [ListColumn; 5] = [
    ListColumn::Bucket,
    ListColumn::Phase,
    ListColumn::Gate,
    ListColumn::Slug,
    ListColumn::Status,
];

pub(crate) trait ListColumnRow {
    fn bucket_label(&self) -> &str;
    fn phase(&self) -> &str;
    fn gate(&self) -> &str;
    fn slug(&self) -> &str;
    fn parse_state(&self) -> ParseState;
}

impl ListColumn {
    pub(crate) fn header(self) -> &'static str {
        match self {
            ListColumn::Bucket => "BUCKET",
            ListColumn::Phase => "PHASE",
            ListColumn::Gate => "GATE",
            ListColumn::Slug => "SLUG",
            ListColumn::Status => "STATUS",
        }
    }

    pub(crate) fn tui_width(self) -> ColumnWidth {
        match self {
            ListColumn::Bucket => ColumnWidth::Fixed(8),
            ListColumn::Phase => ColumnWidth::Fixed(14),
            ListColumn::Gate => ColumnWidth::Fixed(18),
            ListColumn::Slug => ColumnWidth::Min(18),
            ListColumn::Status => ColumnWidth::Fixed(8),
        }
    }

    fn text_min_width(self) -> usize {
        match self {
            ListColumn::Bucket => 7,
            ListColumn::Phase => 10,
            ListColumn::Gate => 14,
            ListColumn::Slug => 8,
            ListColumn::Status => 8,
        }
    }

    pub(crate) fn value(self, row: &impl ListColumnRow) -> String {
        match self {
            ListColumn::Bucket => row.bucket_label().to_string(),
            ListColumn::Phase => row.phase().to_string(),
            ListColumn::Gate => row.gate().to_string(),
            ListColumn::Slug => row.slug().to_string(),
            ListColumn::Status => parse_state_label(row.parse_state()).to_string(),
        }
    }
}

pub(crate) fn markdown_table<R: ListColumnRow>(columns: &[ListColumn], rows: &[&R]) -> String {
    let mut lines = vec![
        markdown_row(columns.iter().map(|column| column.header().to_string())),
        markdown_row(columns.iter().map(|_| "---".to_string())),
    ];
    lines.extend(
        rows.iter()
            .map(|row| markdown_row(columns.iter().map(|column| column.value(*row)))),
    );
    lines.join("\n")
}

pub(crate) fn text_table<R: ListColumnRow>(columns: &[ListColumn], rows: &[R]) -> String {
    let widths = text_widths(columns, rows);
    let mut lines = vec![text_row(
        columns.iter().map(|column| column.header().to_string()),
        &widths,
    )];
    lines.extend(
        rows.iter()
            .map(|row| text_row(columns.iter().map(|column| column.value(row)), &widths)),
    );
    lines.join("\n")
}

pub(crate) fn bucket_label_singular(bucket: Bucket) -> &'static str {
    match bucket {
        Bucket::Seeds => "seed",
        Bucket::Leaves => "leaf",
        Bucket::Fallen => "fallen",
        Bucket::Pressed => "pressed",
    }
}

pub(crate) fn bucket_label_plural(bucket: Bucket) -> &'static str {
    match bucket {
        Bucket::Seeds => "seeds",
        Bucket::Leaves => "leaves",
        Bucket::Fallen => "fallen",
        Bucket::Pressed => "pressed",
    }
}

pub(crate) fn parse_state_label(state: ParseState) -> &'static str {
    match state {
        ParseState::Ok => "ok",
        ParseState::Partial => "partial",
        ParseState::Error => "error",
    }
}

fn markdown_row(cells: impl IntoIterator<Item = String>) -> String {
    format!(
        "| {} |",
        cells
            .into_iter()
            .map(|cell| markdown_table_cell(&cell))
            .collect::<Vec<_>>()
            .join(" | ")
    )
}

fn markdown_table_cell(value: &str) -> String {
    value.replace(['\r', '\n'], " ").replace('|', "\\|")
}

fn text_widths<R: ListColumnRow>(columns: &[ListColumn], rows: &[R]) -> Vec<usize> {
    columns
        .iter()
        .map(|column| {
            let widest_value = rows
                .iter()
                .map(|row| display_width(&column.value(row)))
                .max()
                .unwrap_or(0);
            column
                .text_min_width()
                .max(display_width(column.header()))
                .max(widest_value)
        })
        .collect()
}

fn text_row(cells: impl IntoIterator<Item = String>, widths: &[usize]) -> String {
    cells
        .into_iter()
        .zip(widths.iter().copied())
        .enumerate()
        .map(|(index, (cell, width))| {
            if index + 1 == widths.len() {
                cell
            } else {
                format!("{cell:<width$}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_width(value: &str) -> usize {
    value.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Row {
        bucket: &'static str,
        phase: &'static str,
        gate: &'static str,
        slug: &'static str,
        parse_state: ParseState,
    }

    impl ListColumnRow for Row {
        fn bucket_label(&self) -> &str {
            self.bucket
        }

        fn phase(&self) -> &str {
            self.phase
        }

        fn gate(&self) -> &str {
            self.gate
        }

        fn slug(&self) -> &str {
            self.slug
        }

        fn parse_state(&self) -> ParseState {
            self.parse_state
        }
    }

    #[test]
    fn default_columns_are_the_inventory_display_contract() {
        let headers: Vec<_> = LIST_COLUMNS.iter().map(|column| column.header()).collect();

        assert_eq!(headers, ["BUCKET", "PHASE", "GATE", "SLUG", "STATUS"]);
    }

    #[test]
    fn markdown_table_uses_column_metadata() {
        let row = Row {
            bucket: "leaf",
            phase: "Learn",
            gate: "intent",
            slug: "alpha",
            parse_state: ParseState::Ok,
        };

        assert_eq!(
            markdown_table(&LIST_COLUMNS, &[&row]),
            "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | Learn | intent | alpha | ok |"
        );
    }

    #[test]
    fn markdown_table_escapes_cell_content_once() {
        let row = Row {
            bucket: "leaf",
            phase: "Learn",
            gate: "intent",
            slug: "alpha|beta",
            parse_state: ParseState::Ok,
        };

        assert_eq!(
            markdown_table(&LIST_COLUMNS, &[&row]),
            "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | Learn | intent | alpha\\|beta | ok |"
        );
    }

    #[test]
    fn text_table_uses_the_same_column_order() {
        let row = Row {
            bucket: "leaf",
            phase: "Architect",
            gate: "⑦ Task Graph",
            slug: "active",
            parse_state: ParseState::Ok,
        };

        assert_eq!(
            text_table(&LIST_COLUMNS, &[row]),
            "BUCKET  PHASE      GATE           SLUG     STATUS\nleaf    Architect  ⑦ Task Graph   active   ok"
        );
    }
}
