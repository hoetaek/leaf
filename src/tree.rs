const NON_ZERO_SEED: u64 = 0x9e37_79b9_7f4a_7c15;
const MARKERS: &[u8] = b"1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const FOLIAGE: &[char] = &['&', '%', '#', '@'];

#[derive(Debug)]
pub(crate) struct TreeModel {
    leaves: Vec<TreeLeaf>,
    sprouts: Vec<String>,
    fallen: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct TreeLeaf {
    slug: String,
    pressed: bool,
}

pub(crate) struct TreeRenderOptions {
    pub(crate) color: bool,
    pub(crate) width: usize,
}

pub(crate) struct StableRng {
    state: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Style {
    Plain,
    Trunk,
    Foliage,
    Pressed,
    Ordinary,
    Sprout,
    Fallen,
}

#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    style: Style,
}

impl TreeModel {
    pub(crate) fn from_inventory(inventory: &crate::inventory::Inventory) -> Self {
        let mut leaves = Vec::new();
        let mut sprouts = Vec::new();
        let mut fallen = Vec::new();

        for stage in &inventory.stages {
            match stage.stage_dir {
                crate::inventory::StageDir::Leaves => {
                    leaves.extend(stage.items.iter().map(|item| TreeLeaf {
                        slug: item.slug.clone(),
                        pressed: item.path.join("pressed.md").is_file(),
                    }));
                }
                crate::inventory::StageDir::Sprouts => {
                    sprouts.extend(stage.items.iter().map(|item| item.slug.clone()));
                }
                crate::inventory::StageDir::Fallen => {
                    fallen.extend(stage.items.iter().map(|item| item.slug.clone()));
                }
                crate::inventory::StageDir::Pressed => {}
            }
        }

        leaves.sort_by(|left, right| left.slug.cmp(&right.slug));
        sprouts.sort();
        fallen.sort();

        TreeModel {
            leaves,
            sprouts,
            fallen,
        }
    }

    pub(crate) fn leaf_count(&self) -> usize {
        self.leaves.len()
    }

    pub(crate) fn pressed_count(&self) -> usize {
        self.leaves.iter().filter(|leaf| leaf.pressed).count()
    }

    pub(crate) fn stable_seed(&self) -> u64 {
        let mut parts = vec![b"leaf-tree-v1".to_vec()];
        for leaf in &self.leaves {
            parts.push(b"\xffleaf".to_vec());
            parts.push(leaf.slug.as_bytes().to_vec());
            parts.push(b"\x00pressed".to_vec());
            parts.push(vec![if leaf.pressed { b'1' } else { b'0' }]);
        }
        for sprout in &self.sprouts {
            parts.push(b"\xffsprout".to_vec());
            parts.push(sprout.as_bytes().to_vec());
        }
        for fallen in &self.fallen {
            parts.push(b"\xfffallen".to_vec());
            parts.push(fallen.as_bytes().to_vec());
        }

        let seed = fnv1a64(parts);
        if seed == 0 { NON_ZERO_SEED } else { seed }
    }
}

impl StableRng {
    pub(crate) fn new(seed: u64) -> Self {
        let state = if seed == 0 { NON_ZERO_SEED } else { seed };
        StableRng { state }
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }
}

fn fnv1a64(parts: impl IntoIterator<Item = impl AsRef<[u8]>>) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for part in parts {
        for &byte in part.as_ref() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

pub(crate) fn write_text<W: std::io::Write>(
    mut writer: W,
    model: &TreeModel,
    options: TreeRenderOptions,
) -> anyhow::Result<()> {
    writer.write_all(render_to_string(model, options).as_bytes())?;
    Ok(())
}

fn render_to_string(model: &TreeModel, options: TreeRenderOptions) -> String {
    let width = render_width(options.width);
    let mut output = String::new();
    push_styled_line(
        &mut output,
        &format!(
            "leaf tree | leaves {} | pressed {} | sprouts {} | fallen {}",
            model.leaf_count(),
            model.pressed_count(),
            model.sprouts.len(),
            model.fallen.len()
        ),
        Style::Foliage,
        options.color,
    );
    output.push('\n');

    for line in render_canvas(model, width) {
        output.push_str(&line_to_string(&line, options.color));
        output.push('\n');
    }
    output.push('\n');

    push_sprouts(&mut output, &model.sprouts, options.color, width);
    push_leaf_sections(&mut output, model, options.color, width);
    push_fallen(&mut output, &model.fallen, options.color, width);

    output
}

fn render_width(width: usize) -> usize {
    width.clamp(80, 112)
}

fn render_canvas(model: &TreeModel, width: usize) -> Vec<Vec<Cell>> {
    let leaf_count = model.leaf_count();
    let maturity = 1.0 - (-(leaf_count as f64) / 18.0).exp();
    let branch_layers = (1 + ((leaf_count + 1) as f64).log2().floor() as usize).clamp(1, 5);
    let trunk_height = 5 + (10.0 * maturity).round() as usize;
    let crown_width = (12 + (66.0 * maturity).round() as usize).min(width.saturating_sub(4));
    let foliage_budget = (8.0 * leaf_count as f64 + 160.0 * maturity)
        .round()
        .min(650.0) as usize;
    let crown_height = 6 + branch_layers * 3;
    let height = crown_height + trunk_height;
    let center = width / 2;
    let mut canvas = vec![vec![Cell::blank(); width]; height];

    let foliage_positions = draw_foliage(
        &mut canvas,
        width,
        crown_height,
        crown_width,
        foliage_budget,
        model.stable_seed(),
    );
    draw_branches_and_trunk(
        &mut canvas,
        center,
        crown_height,
        trunk_height,
        branch_layers,
    );
    draw_markers(&mut canvas, model, &foliage_positions);

    canvas
}

fn draw_foliage(
    canvas: &mut [Vec<Cell>],
    width: usize,
    crown_height: usize,
    crown_width: usize,
    foliage_budget: usize,
    seed: u64,
) -> Vec<(usize, usize)> {
    if foliage_budget == 0 {
        return Vec::new();
    }

    let center_x = width as isize / 2;
    let radius_x = crown_width as f64 / 2.0;
    let radius_y = crown_height as f64 / 2.0;
    let center_y = (crown_height as f64 - 1.0) / 2.0;
    let left = center_x - crown_width as isize / 2;
    let right = center_x + crown_width as isize / 2;
    let mut rng = StableRng::new(seed);
    let mut candidates = Vec::new();

    for y in 0..crown_height {
        let normalized_y = (y as f64 - center_y) / radius_y;
        for x in left..=right {
            if x <= 0 || x >= width as isize - 1 {
                continue;
            }
            let normalized_x = (x as f64 - center_x as f64) / radius_x;
            if normalized_x * normalized_x + normalized_y * normalized_y <= 1.0 {
                let score = rng.next_u64();
                let ch = FOLIAGE[(rng.next_u64() as usize) % FOLIAGE.len()];
                candidates.push((score, x as usize, y, ch));
            }
        }
    }

    candidates.sort_by_key(|(score, _, _, _)| *score);
    let mut selected = Vec::new();
    for (_, x, y, ch) in candidates.into_iter().take(foliage_budget) {
        put_cell(canvas, x, y, ch, Style::Foliage);
        selected.push((x, y));
    }
    selected
}

fn draw_branches_and_trunk(
    canvas: &mut [Vec<Cell>],
    center: usize,
    crown_height: usize,
    trunk_height: usize,
    branch_layers: usize,
) {
    for layer in 0..branch_layers {
        let y = crown_height.saturating_sub(2 + layer * 3);
        let span = 5 + layer * 5;
        for offset in 1..=span {
            if offset % 2 == 0 {
                put_cell(canvas, center.saturating_sub(offset), y, '/', Style::Trunk);
                put_cell(canvas, center + offset, y, '\\', Style::Trunk);
            }
        }
    }

    for y in crown_height.saturating_sub(2)..(crown_height + trunk_height) {
        put_cell(canvas, center.saturating_sub(1), y, '|', Style::Trunk);
        put_cell(canvas, center, y, '|', Style::Trunk);
    }
}

fn draw_markers(canvas: &mut [Vec<Cell>], model: &TreeModel, foliage_positions: &[(usize, usize)]) {
    if foliage_positions.is_empty() {
        return;
    }

    let marker_count = model
        .leaves
        .len()
        .min(MARKERS.len())
        .min(foliage_positions.len());
    let mut rng = StableRng::new(model.stable_seed() ^ 0xa076_1d64_78bd_642f);
    let mut positions = foliage_positions
        .iter()
        .map(|&(x, y)| (rng.next_u64(), x, y))
        .collect::<Vec<_>>();
    positions.sort_by_key(|(score, _, _)| *score);

    for (index, leaf) in model.leaves.iter().take(marker_count).enumerate() {
        let (_, x, y) = positions[index];
        let style = if leaf.pressed {
            Style::Pressed
        } else {
            Style::Ordinary
        };
        put_cell(canvas, x, y, marker_char(index), style);
    }
}

fn put_cell(canvas: &mut [Vec<Cell>], x: usize, y: usize, ch: char, style: Style) {
    if let Some(row) = canvas.get_mut(y)
        && let Some(cell) = row.get_mut(x)
    {
        *cell = Cell { ch, style };
    }
}

fn push_leaf_sections(output: &mut String, model: &TreeModel, color: bool, width: usize) {
    let marker_capacity = MARKERS.len();
    let marker_leaves = model
        .leaves
        .iter()
        .enumerate()
        .take(marker_capacity)
        .collect::<Vec<_>>();
    let pressed_count = model.pressed_count();
    let ordinary_count = model.leaf_count().saturating_sub(pressed_count);
    let pressed_values = marker_leaves
        .iter()
        .filter_map(|(index, leaf)| {
            leaf.pressed
                .then(|| format!("{} {}", marker_char(*index), leaf.slug))
        })
        .collect::<Vec<_>>();
    let ordinary_values = marker_leaves
        .iter()
        .filter_map(|(index, leaf)| {
            (!leaf.pressed).then(|| format!("{} {}", marker_char(*index), leaf.slug))
        })
        .collect::<Vec<_>>();
    let hidden_pressed = pressed_count.saturating_sub(pressed_values.len());
    let hidden_ordinary = ordinary_count.saturating_sub(ordinary_values.len());

    if pressed_count == 0 {
        push_styled_line(
            output,
            "no gold fruit: no pressed leaf yet",
            Style::Pressed,
            color,
        );
    } else if pressed_values.is_empty() {
        push_styled_line(
            output,
            &format!(
                "gold fruit: {}",
                hidden_marker_message(hidden_pressed, "pressed leaf", "pressed leaves")
            ),
            Style::Pressed,
            color,
        );
    } else {
        push_wrapped_values(
            output,
            "gold fruit:",
            &pressed_values,
            Style::Pressed,
            color,
            width,
        );
        if hidden_pressed > 0 {
            push_styled_line(
                output,
                &hidden_marker_message(hidden_pressed, "pressed leaf", "pressed leaves"),
                Style::Pressed,
                color,
            );
        }
    }

    if ordinary_count > 0 {
        if ordinary_values.is_empty() {
            push_styled_line(
                output,
                &format!(
                    "green leaves: {}",
                    hidden_marker_message(hidden_ordinary, "ordinary leaf", "ordinary leaves")
                ),
                Style::Ordinary,
                color,
            );
        } else {
            push_wrapped_values(
                output,
                "green leaves:",
                &ordinary_values,
                Style::Ordinary,
                color,
                width,
            );
            if hidden_ordinary > 0 {
                push_styled_line(
                    output,
                    &hidden_marker_message(hidden_ordinary, "ordinary leaf", "ordinary leaves"),
                    Style::Ordinary,
                    color,
                );
            }
        }
    }

    let hidden = hidden_pressed + hidden_ordinary;
    if hidden > 0 {
        push_wrapped_values(
            output,
            "",
            &[hidden_leaf_message(hidden)],
            Style::Plain,
            color,
            width,
        );
    }
}

fn hidden_marker_message(count: usize, singular: &str, plural: &str) -> String {
    let leaf_label = if count == 1 { singular } else { plural };
    let marker_label = if count == 1 { "marker" } else { "markers" };
    format!("+ {count} {leaf_label} not shown as {marker_label}")
}

fn hidden_leaf_message(count: usize) -> String {
    let leaf_label = if count == 1 { "leaf" } else { "leaves" };
    let marker_label = if count == 1 { "marker" } else { "markers" };
    format!("+ {count} more {leaf_label} not shown as {marker_label}")
}

fn push_sprouts(output: &mut String, sprouts: &[String], color: bool, width: usize) {
    if sprouts.is_empty() {
        return;
    }

    output.push('\n');
    push_styled_line(output, "active sprouts:", Style::Sprout, color);
    let shown = sprouts.len().min(8);
    let glyphs = std::iter::repeat_n(r"\|/", shown)
        .collect::<Vec<_>>()
        .join("  ");
    let stems = std::iter::repeat_n(" | ", shown)
        .collect::<Vec<_>>()
        .join("  ");
    push_styled_line(output, &format!("  {glyphs}"), Style::Sprout, color);
    push_styled_line(output, &format!("  {stems}"), Style::Sprout, color);

    let shown_labels = sprouts
        .iter()
        .take(if sprouts.len() <= 4 { 4 } else { 8 })
        .cloned()
        .collect::<Vec<_>>();
    push_wrapped_values(output, "  ", &shown_labels, Style::Sprout, color, width);
    let hidden = sprouts.len().saturating_sub(shown_labels.len());
    if hidden > 0 {
        push_styled_line(
            output,
            &format!("  + {hidden} more active sprouts"),
            Style::Sprout,
            color,
        );
    }
}

fn push_fallen(output: &mut String, fallen: &[String], color: bool, width: usize) {
    if fallen.is_empty() {
        return;
    }

    output.push('\n');
    let shown = fallen.iter().take(8).cloned().collect::<Vec<_>>();
    push_wrapped_values(output, "fallen:", &shown, Style::Fallen, color, width);
    let hidden = fallen.len().saturating_sub(shown.len());
    if hidden > 0 {
        push_styled_line(
            output,
            &format!("+ {hidden} more fallen items"),
            Style::Fallen,
            color,
        );
    }
}

fn push_wrapped_values(
    output: &mut String,
    label: &str,
    values: &[String],
    style: Style,
    color: bool,
    width: usize,
) {
    if values.is_empty() {
        push_styled_line(output, label.trim_end(), style, color);
        return;
    }

    let prefix = if label.trim().is_empty() {
        label.to_string()
    } else {
        format!("{label} ")
    };
    let mut line = prefix.clone();
    for value in values {
        let separator = if line == prefix { "" } else { ", " };
        if visible_len(&line) + separator.len() + visible_len(value) > width {
            push_styled_line(output, line.trim_end(), style, color);
            line = format!("  {value}");
        } else {
            line.push_str(separator);
            line.push_str(value);
        }
    }
    push_styled_line(output, line.trim_end(), style, color);
}

fn push_styled_line(output: &mut String, text: &str, style: Style, color: bool) {
    if color && style != Style::Plain {
        output.push_str(style.ansi_code());
        output.push_str(text);
        output.push_str("\x1b[0m");
    } else {
        output.push_str(text);
    }
    output.push('\n');
}

fn line_to_string(line: &[Cell], color: bool) -> String {
    let end = line
        .iter()
        .rposition(|cell| cell.ch != ' ')
        .map_or(0, |index| index + 1);
    let mut output = String::new();
    let mut active_style = Style::Plain;

    for cell in &line[..end] {
        if color && cell.style != active_style {
            if active_style != Style::Plain {
                output.push_str("\x1b[0m");
            }
            if cell.style != Style::Plain {
                output.push_str(cell.style.ansi_code());
            }
            active_style = cell.style;
        }
        output.push(cell.ch);
    }

    if color && active_style != Style::Plain {
        output.push_str("\x1b[0m");
    }
    output
}

fn visible_len(value: &str) -> usize {
    value.chars().count()
}

fn marker_char(index: usize) -> char {
    MARKERS[index] as char
}

impl Cell {
    fn blank() -> Self {
        Cell {
            ch: ' ',
            style: Style::Plain,
        }
    }
}

impl Style {
    fn ansi_code(self) -> &'static str {
        match self {
            Style::Plain => "",
            Style::Trunk => "\x1b[33m",
            Style::Foliage => "\x1b[32m",
            Style::Pressed => "\x1b[93;1m",
            Style::Ordinary => "\x1b[92;1m",
            Style::Sprout => "\x1b[92;1m",
            Style::Fallen => "\x1b[90m",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    fn status(stage: &str) -> String {
        format!(
            "- stage: {stage}\n\
             - current phase: Architect\n\
             - current gate: ⑦ Tasks\n\
             - first missing gate: ⑧ Artifact\n\
             - next action: render tree\n"
        )
    }

    fn write_leaf(root: &assert_fs::TempDir, slug: &str, pressed: bool) {
        root.child(format!(".leaf/02-leaves/{slug}/00-status.md"))
            .write_str(&status("leaf"))
            .expect("leaf status");
        if pressed {
            root.child(format!(".leaf/02-leaves/{slug}/pressed.md"))
                .write_str("# Pressed\n")
                .expect("pressed digest");
        }
    }

    fn write_sprout(root: &assert_fs::TempDir, slug: &str) {
        root.child(format!(".leaf/01-sprouts/{slug}/00-status.md"))
            .write_str(&status("sprout"))
            .expect("sprout status");
    }

    fn write_fallen(root: &assert_fs::TempDir, slug: &str) {
        root.child(format!(".leaf/03-fallen/{slug}/00-status.md"))
            .write_str("- stage: fallen\n- fallen reason: archived\n")
            .expect("fallen status");
    }

    fn load_model(root: &assert_fs::TempDir) -> TreeModel {
        let inventory = crate::inventory::load(root.path()).expect("inventory");
        TreeModel::from_inventory(&inventory)
    }

    #[test]
    fn tree_model_projects_sorted_stage_slugs_and_pressed_digest_state() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf").create_dir_all().expect("leaf root");
        write_leaf(&root, "beta", false);
        write_leaf(&root, "alpha", true);
        write_sprout(&root, "draft-b");
        write_sprout(&root, "draft-a");
        write_fallen(&root, "old-b");
        write_fallen(&root, "old-a");

        let model = load_model(&root);

        assert_eq!(
            model
                .leaves
                .iter()
                .map(|leaf| (leaf.slug.as_str(), leaf.pressed))
                .collect::<Vec<_>>(),
            vec![("alpha", true), ("beta", false)]
        );
        assert_eq!(model.sprouts, vec!["draft-a", "draft-b"]);
        assert_eq!(model.fallen, vec!["old-a", "old-b"]);
        assert_eq!(model.leaf_count(), 2);
        assert_eq!(model.pressed_count(), 1);
    }

    #[test]
    fn stable_seed_repeats_for_same_model_and_changes_when_pressed_state_changes() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf").create_dir_all().expect("leaf root");
        write_leaf(&root, "alpha", true);
        write_leaf(&root, "beta", false);

        let first_seed = load_model(&root).stable_seed();
        let second_seed = load_model(&root).stable_seed();
        assert_eq!(first_seed, second_seed);

        root.child(".leaf/02-leaves/beta/pressed.md")
            .write_str("# Pressed\n")
            .expect("pressed digest");
        let changed_seed = load_model(&root).stable_seed();
        assert_ne!(first_seed, changed_seed);
    }

    #[test]
    fn stable_rng_repeats_sequence_for_same_seed() {
        let mut left = StableRng::new(0x0123_4567_89ab_cdef);
        let mut right = StableRng::new(0x0123_4567_89ab_cdef);

        let left_values = (0..8).map(|_| left.next_u64()).collect::<Vec<_>>();
        let right_values = (0..8).map(|_| right.next_u64()).collect::<Vec<_>>();

        assert_eq!(left_values, right_values);
        assert!(left_values.iter().any(|&value| value != 0));
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;

    fn model_with_leaves(count: usize, pressed_every: Option<usize>) -> TreeModel {
        let leaves = (1..=count)
            .map(|index| TreeLeaf {
                slug: format!("leaf-{index:02}"),
                pressed: pressed_every.is_some_and(|every| index % every == 0),
            })
            .collect();
        TreeModel {
            leaves,
            sprouts: Vec::new(),
            fallen: Vec::new(),
        }
    }

    fn strip_ansi(input: &str) -> String {
        let mut output = String::new();
        let mut chars = input.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\x1b' && chars.peek() == Some(&'[') {
                chars.next();
                for next in chars.by_ref() {
                    if next == 'm' {
                        break;
                    }
                }
            } else {
                output.push(ch);
            }
        }
        output
    }

    fn foliage_cells(input: &str) -> usize {
        strip_ansi(input)
            .chars()
            .filter(|ch| matches!(ch, '&' | '%' | '#' | '@'))
            .count()
    }

    #[test]
    fn render_color_and_plain_keep_same_semantics() {
        let model = TreeModel {
            leaves: vec![
                TreeLeaf {
                    slug: "pressed-leaf".to_string(),
                    pressed: true,
                },
                TreeLeaf {
                    slug: "ordinary-leaf".to_string(),
                    pressed: false,
                },
            ],
            sprouts: vec!["draft-sprout".to_string()],
            fallen: vec!["archived-leaf".to_string()],
        };

        let color = render_to_string(
            &model,
            TreeRenderOptions {
                color: true,
                width: 112,
            },
        );
        let plain = render_to_string(
            &model,
            TreeRenderOptions {
                color: false,
                width: 112,
            },
        );

        assert!(
            color.contains("\x1b["),
            "default tree output must contain ANSI: {color:?}"
        );
        assert!(
            !plain.contains("\x1b["),
            "plain output must not contain ANSI: {plain:?}"
        );
        for needle in [
            "leaf tree",
            "leaves 2",
            "pressed 1",
            "sprouts 1",
            "fallen 1",
            "active sprouts:",
            "gold fruit:",
            "green leaves:",
            "fallen:",
            "pressed-leaf",
            "ordinary-leaf",
            "draft-sprout",
            "archived-leaf",
        ] {
            assert!(
                plain.contains(needle),
                "missing {needle:?} in plain output:\n{plain}"
            );
        }
    }

    #[test]
    fn active_sprouts_render_before_leaf_legends() {
        let model = TreeModel {
            leaves: vec![
                TreeLeaf {
                    slug: "pressed-leaf".to_string(),
                    pressed: true,
                },
                TreeLeaf {
                    slug: "ordinary-leaf".to_string(),
                    pressed: false,
                },
            ],
            sprouts: vec!["draft-sprout".to_string()],
            fallen: Vec::new(),
        };

        let rendered = render_to_string(
            &model,
            TreeRenderOptions {
                color: false,
                width: 112,
            },
        );

        let sprouts_at = rendered.find("active sprouts:").expect("sprouts section");
        let gold_at = rendered.find("gold fruit:").expect("gold fruit section");
        let green_at = rendered
            .find("green leaves:")
            .expect("green leaves section");

        assert!(
            sprouts_at < gold_at,
            "active sprouts must render before gold fruit:\n{rendered}"
        );
        assert!(
            sprouts_at < green_at,
            "active sprouts must render before green leaves:\n{rendered}"
        );
    }

    #[test]
    fn foliage_grows_with_leaf_count_but_output_height_stays_bounded() {
        let counts = [0, 1, 3, 10, 20, 50];
        let mut previous_foliage = 0;

        for count in counts {
            let rendered = render_to_string(
                &model_with_leaves(count, Some(3)),
                TreeRenderOptions {
                    color: false,
                    width: 112,
                },
            );
            let foliage = foliage_cells(&rendered);
            assert!(
                foliage >= previous_foliage,
                "foliage should not shrink from previous checkpoint; count={count}\n{rendered}"
            );
            assert!(
                rendered.lines().count() <= 90,
                "tree output must stay bounded for {count} leaves; got {} lines",
                rendered.lines().count()
            );
            previous_foliage = foliage;
        }
    }

    #[test]
    fn no_pressed_many_leaves_renders_no_gold_fruit_message() {
        let rendered = render_to_string(
            &model_with_leaves(50, None),
            TreeRenderOptions {
                color: false,
                width: 112,
            },
        );

        assert!(rendered.contains("leaves 50"));
        assert!(rendered.contains("pressed 0"));
        assert!(rendered.contains("green leaves:"));
        assert!(rendered.contains("no gold fruit: no pressed leaf yet"));
    }

    #[test]
    fn all_pressed_leaves_omit_empty_green_leaves_section() {
        let rendered = render_to_string(
            &model_with_leaves(5, Some(1)),
            TreeRenderOptions {
                color: false,
                width: 112,
            },
        );

        assert!(rendered.contains("gold fruit:"));
        assert!(!rendered.contains("green leaves:"));
    }

    #[test]
    fn hidden_pressed_leaf_does_not_render_empty_gold_fruit_legend() {
        let mut model = model_with_leaves(MARKERS.len() + 1, None);
        model.leaves[MARKERS.len()].pressed = true;

        let rendered = render_to_string(
            &model,
            TreeRenderOptions {
                color: false,
                width: 112,
            },
        );

        assert!(rendered.contains("pressed 1"));
        assert!(
            !rendered.lines().any(|line| line == "gold fruit:"),
            "hidden pressed leaf must not produce an empty gold fruit legend:\n{rendered}"
        );
        assert!(rendered.contains("gold fruit: + 1 pressed leaf not shown as marker"));
        assert!(rendered.contains("green leaves:"));
    }

    #[test]
    fn hidden_ordinary_leaf_does_not_render_empty_green_leaves_legend() {
        let mut model = model_with_leaves(MARKERS.len() + 1, Some(1));
        model.leaves[MARKERS.len()].pressed = false;

        let rendered = render_to_string(
            &model,
            TreeRenderOptions {
                color: false,
                width: 112,
            },
        );

        assert!(rendered.contains(&format!("pressed {}", MARKERS.len())));
        assert!(rendered.contains("gold fruit:"));
        assert!(
            !rendered.lines().any(|line| line == "green leaves:"),
            "hidden ordinary leaf must not produce an empty green leaves legend:\n{rendered}"
        );
        assert!(rendered.contains("green leaves: + 1 ordinary leaf not shown as marker"));
    }
}
