const NON_ZERO_SEED: u64 = 0x9e37_79b9_7f4a_7c15;
const PREFERRED_RENDER_WIDTH: usize = 112;
const MIN_TREE_DRAW_WIDTH: usize = 32;
const MARKERS: &[u8] = b"1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const FOLIAGE: &[char] = &['&', '%', '#', '@'];
const DEMO_LEAF_COUNTS: &[usize] = &[0, 3, 10, 20, 50, 100];

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

#[derive(Clone, Copy)]
struct BranchSpec {
    x: f64,
    y: f64,
    angle: f64,
    length: f64,
    depth: usize,
}

#[derive(Clone, Copy)]
struct FoliageCluster {
    center_x: isize,
    center_y: isize,
    radius_x: isize,
    radius_y: isize,
    budget: usize,
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

pub(crate) fn write_demo_text<W: std::io::Write>(
    mut writer: W,
    options: TreeRenderOptions,
) -> anyhow::Result<()> {
    writer.write_all(render_demo_to_string(options).as_bytes())?;
    Ok(())
}

fn render_to_string(model: &TreeModel, options: TreeRenderOptions) -> String {
    let width = render_width(options.width);
    if width < MIN_TREE_DRAW_WIDTH {
        return render_compact_summary(model, options.color, width);
    }

    let mut output = String::new();
    push_tree_header(&mut output, model, options.color, width);
    output.push('\n');

    for line in render_canvas(model, width) {
        output.push_str(&line_to_string(&line, options.color));
        output.push('\n');
    }
    output.push('\n');

    push_sprouts(&mut output, &model.sprouts, options.color);
    push_leaf_sections(&mut output, model, options.color, width);
    push_fallen(&mut output, &model.fallen, options.color, width);

    output
}

fn render_demo_to_string(options: TreeRenderOptions) -> String {
    let width = render_width(options.width);
    let mut output = String::new();

    push_wrapped_styled_line(
        &mut output,
        "leaf tree demo | top -> bottom: more folders in .leaf/02-leaves",
        Style::Foliage,
        options.color,
        width,
    );

    for (index, &count) in DEMO_LEAF_COUNTS.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        output.push('\n');
        push_wrapped_styled_line(
            &mut output,
            &format!(
                "===== {} / {} demo =====",
                leaf_count_label(count),
                demo_stage_label(count)
            ),
            Style::Foliage,
            options.color,
            width,
        );
        output.push_str(&render_to_string(
            &demo_model(count),
            TreeRenderOptions {
                color: options.color,
                width,
            },
        ));
    }

    output
}

fn leaf_count_label(count: usize) -> String {
    let label = if count == 1 { "leaf" } else { "leaves" };
    format!("{count} {label}")
}

fn demo_stage_label(count: usize) -> &'static str {
    match count {
        0 => "seedling",
        1..=3 => "young",
        4..=10 => "branching",
        11..=20 => "grown",
        21..=50 => "mature",
        _ => "saturated",
    }
}

fn demo_model(leaf_count: usize) -> TreeModel {
    let leaves = (1..=leaf_count)
        .map(|index| TreeLeaf {
            slug: format!("demo-leaf-{index:02}"),
            pressed: false,
        })
        .collect();
    TreeModel {
        leaves,
        sprouts: Vec::new(),
        fallen: Vec::new(),
    }
}

fn render_width(width: usize) -> usize {
    width.clamp(1, PREFERRED_RENDER_WIDTH)
}

fn render_compact_summary(model: &TreeModel, color: bool, width: usize) -> String {
    let mut output = String::new();
    push_wrapped_styled_line(&mut output, "leaf tree", Style::Foliage, color, width);
    push_wrapped_styled_line(
        &mut output,
        "tree view too narrow",
        Style::Foliage,
        color,
        width,
    );
    push_wrapped_styled_line(
        &mut output,
        &format!("leaves {}", model.leaf_count()),
        Style::Ordinary,
        color,
        width,
    );
    push_wrapped_styled_line(
        &mut output,
        &format!("pressed {}", model.pressed_count()),
        Style::Pressed,
        color,
        width,
    );
    push_wrapped_styled_line(
        &mut output,
        &format!("sprouts {}", model.sprouts.len()),
        Style::Sprout,
        color,
        width,
    );
    push_wrapped_styled_line(
        &mut output,
        &format!("fallen {}", model.fallen.len()),
        Style::Fallen,
        color,
        width,
    );
    output
}

fn push_tree_header(output: &mut String, model: &TreeModel, color: bool, width: usize) {
    let one_line = format!(
        "leaf tree | leaves {} | pressed {} | sprouts {} | fallen {}",
        model.leaf_count(),
        model.pressed_count(),
        model.sprouts.len(),
        model.fallen.len()
    );
    if visible_len(&one_line) <= width {
        push_styled_line(output, &one_line, Style::Foliage, color);
        return;
    }

    push_wrapped_styled_line(output, "leaf tree", Style::Foliage, color, width);
    for line in [
        format!("leaves {}", model.leaf_count()),
        format!("pressed {}", model.pressed_count()),
        format!("sprouts {}", model.sprouts.len()),
        format!("fallen {}", model.fallen.len()),
    ] {
        push_wrapped_styled_line(output, &line, Style::Foliage, color, width);
    }
}

fn render_canvas(model: &TreeModel, width: usize) -> Vec<Vec<Cell>> {
    let leaf_count = model.leaf_count();
    let maturity = 1.0 - (-(leaf_count as f64) / 18.0).exp();
    let width_scale = (width as f64 / PREFERRED_RENDER_WIDTH as f64).clamp(0.45, 1.0);
    let max_layers = if width < 48 { 4 } else { 5 };
    let branch_layers = if leaf_count == 0 {
        1
    } else {
        (1 + ((leaf_count + 1) as f64).log2().floor() as usize).clamp(2, max_layers)
    };
    let trunk_height = 5 + (8.0 * maturity * width_scale).round() as usize;
    let crown_height = 6 + branch_layers * 3;
    let height = crown_height + trunk_height;
    let center = width / 2;
    let mut canvas = vec![vec![Cell::blank(); width]; height];
    let mut rng = StableRng::new(model.stable_seed());
    let mut tips = Vec::new();
    let base_y = height.saturating_sub(2) as f64;
    let initial_length = (8.0 + 6.0 * maturity + branch_layers as f64) * width_scale;

    draw_tree_branch(
        &mut canvas,
        &mut tips,
        &mut rng,
        BranchSpec {
            x: center as f64,
            y: base_y,
            angle: std::f64::consts::FRAC_PI_2,
            length: initial_length,
            depth: branch_layers,
        },
        leaf_count,
    );
    draw_trunk(
        &mut canvas,
        center,
        crown_height,
        height,
        leaf_count,
        maturity,
    );

    let foliage_positions = draw_tip_foliage(&mut canvas, model, &tips, model.stable_seed());
    draw_markers(&mut canvas, model, &foliage_positions);
    draw_sprouts_around_tree(&mut canvas, center, model.sprouts.len());

    canvas
}

fn draw_tree_branch(
    canvas: &mut [Vec<Cell>],
    tips: &mut Vec<(usize, usize)>,
    rng: &mut StableRng,
    branch: BranchSpec,
    leaf_count: usize,
) {
    let end_x = branch.x + branch.length * branch.angle.cos();
    let end_y = branch.y - branch.length * branch.angle.sin() * 0.55;
    let width = branch_width(branch.depth, leaf_count);
    draw_line(
        canvas,
        branch.x.round() as isize,
        branch.y.round() as isize,
        end_x.round() as isize,
        end_y.round() as isize,
        width,
    );

    if branch.depth == 0 {
        let tip_x = end_x.round() as isize;
        let tip_y = end_y.round() as isize;
        if in_canvas(canvas, tip_x, tip_y) {
            tips.push((tip_x as usize, tip_y as usize));
        }
        return;
    }

    let kids = if leaf_count > 20 && branch.depth <= 2 && rng.next_u64().is_multiple_of(3) {
        3
    } else {
        2
    };

    for child in 0..kids {
        let side = match (kids, child) {
            (3, 1) => 0.0,
            (_, child) if child % 2 == 0 => -1.0,
            _ => 1.0,
        };
        let spread = if side == 0.0 {
            random_range(rng, -8.0, 8.0)
        } else {
            side * random_range(rng, 28.0, 54.0)
        };
        let child_angle =
            (branch.angle + spread.to_radians()).clamp(20_f64.to_radians(), 160_f64.to_radians());
        let child_length = branch.length * random_range(rng, 0.70, 0.82);
        draw_tree_branch(
            canvas,
            tips,
            rng,
            BranchSpec {
                x: end_x,
                y: end_y,
                angle: child_angle,
                length: child_length,
                depth: branch.depth - 1,
            },
            leaf_count,
        );
    }
}

fn branch_width(depth: usize, leaf_count: usize) -> usize {
    let mature_bonus = usize::from(leaf_count > 20);
    (1 + depth.saturating_sub(2) + mature_bonus).min(5)
}

fn draw_line(canvas: &mut [Vec<Cell>], x0: isize, y0: isize, x1: isize, y1: isize, width: usize) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = dx.abs().max(dy.abs()).max(1);
    let ch = branch_char(dx, dy);
    let half = width as isize / 2;

    for step in 0..=steps {
        let t = step as f64 / steps as f64;
        let x = (x0 as f64 + dx as f64 * t).round() as isize;
        let y = (y0 as f64 + dy as f64 * t).round() as isize;
        for offset in -half..(width as isize - half) {
            put_cell_isize(canvas, x + offset, y, ch, Style::Trunk);
        }
        if width >= 4 && step % 2 == 0 {
            put_cell_isize(canvas, x, y + 1, ch, Style::Trunk);
        }
    }
}

fn branch_char(dx: isize, dy: isize) -> char {
    if dx == 0 || dy.abs() > dx.abs() * 2 {
        '|'
    } else if dx * dy < 0 {
        '/'
    } else if dx * dy > 0 {
        '\\'
    } else {
        '~'
    }
}

fn draw_trunk(
    canvas: &mut [Vec<Cell>],
    center: usize,
    crown_height: usize,
    height: usize,
    leaf_count: usize,
    maturity: f64,
) {
    let trunk_width =
        2 + usize::from(leaf_count >= 10 && maturity > 0.42) + usize::from(maturity > 0.82);
    let top = crown_height.saturating_sub(2);
    let bottom = height.saturating_sub(2);
    for y in top..=bottom {
        let half = trunk_width / 2;
        for offset in 0..trunk_width {
            let x = center + offset - half;
            put_cell(canvas, x, y, '|', Style::Trunk);
        }
    }

    let base = match trunk_width {
        4 => "/||||\\",
        3 => "/|||\\",
        _ => "/||\\",
    };
    let start = center.saturating_sub(base.chars().count() / 2);
    for (offset, ch) in base.chars().enumerate() {
        put_cell(canvas, start + offset, bottom, ch, Style::Trunk);
    }
}

fn draw_sprouts_around_tree(canvas: &mut [Vec<Cell>], center: usize, sprout_count: usize) {
    if sprout_count == 0 || canvas.len() < 5 {
        return;
    }

    let top_y = canvas.len() as isize - 5;
    let stem_y = canvas.len() as isize - 4;
    let center = center as isize;
    let offsets = [-14, 14, -22, 22, -30, 30, -38, 38];

    for offset in offsets.into_iter().take(sprout_count.min(8)) {
        let x = center + offset;
        put_cell_isize(canvas, x - 1, top_y, '\\', Style::Sprout);
        put_cell_isize(canvas, x, top_y, '|', Style::Sprout);
        put_cell_isize(canvas, x + 1, top_y, '/', Style::Sprout);
        put_cell_isize(canvas, x, stem_y, '|', Style::Sprout);
    }
}

fn draw_tip_foliage(
    canvas: &mut [Vec<Cell>],
    model: &TreeModel,
    tips: &[(usize, usize)],
    seed: u64,
) -> Vec<(usize, usize)> {
    let leaf_count = model.leaf_count();
    if leaf_count == 0 || tips.is_empty() {
        return Vec::new();
    }

    let maturity = 1.0 - (-(leaf_count as f64) / 18.0).exp();
    let saturation = (leaf_count as f64 / 100.0).clamp(0.0, 1.0);
    let width = canvas.first().map_or(0, Vec::len);
    let budget_cap = (width as f64 * 24.0).round().clamp(260.0, 2600.0);
    let total_budget =
        (18.0 * leaf_count as f64 + 40.0 * leaf_count as f64 * saturation + 170.0 * maturity)
            .round()
            .min(budget_cap) as usize;
    let radius_x = 3 + (7.0 * maturity + 5.0 * saturation).round() as isize;
    let radius_y = 1 + (3.0 * maturity + 2.0 * saturation).round() as isize;
    let mut rng = StableRng::new(seed);
    let mut sorted_tips = tips.to_vec();
    sorted_tips.sort_by_key(|(x, y)| (*x, *y));
    let mut positions = Vec::new();
    let mut centers = Vec::new();

    for index in 0..leaf_count {
        let tip_index = index * sorted_tips.len() / leaf_count;
        let (tip_x, tip_y) = sorted_tips[tip_index];
        let center_x = tip_x as isize + random_isize(&mut rng, -1, 1);
        let center_y = tip_y as isize + random_isize(&mut rng, -1, 1);
        centers.push((center_x, center_y));
        let base = total_budget / leaf_count;
        let extra = usize::from(index < total_budget % leaf_count);
        let pressed_bonus = usize::from(model.leaves[index].pressed) * 3;
        let max_blob = (24.0 + 36.0 * saturation).round() as usize;
        let blob = (base + extra + pressed_bonus).clamp(6, max_blob);
        add_foliage_cluster(
            canvas,
            &mut positions,
            &mut rng,
            FoliageCluster {
                center_x,
                center_y,
                radius_x,
                radius_y,
                budget: blob,
            },
        );
    }

    if leaf_count >= 6 {
        centers.sort_by_key(|(x, y)| (*x, *y));
        let bridge_passes = if leaf_count >= 100 {
            10
        } else if leaf_count >= 50 {
            7
        } else if leaf_count >= 20 {
            2
        } else {
            1
        };
        for _ in 0..bridge_passes {
            for pair in centers.windows(2) {
                let (left_x, left_y) = pair[0];
                let (right_x, right_y) = pair[1];
                let dist = (right_x - left_x).abs();
                if dist > 28 {
                    continue;
                }
                let steps = (dist / 2).clamp(2, 14);
                for step in 1..steps {
                    let t = step as f64 / steps as f64;
                    let x = (left_x as f64 + (right_x - left_x) as f64 * t).round() as isize
                        + random_isize(&mut rng, -1, 1);
                    let y = (left_y as f64 + (right_y - left_y) as f64 * t).round() as isize
                        + random_isize(&mut rng, -1, 1);
                    put_foliage(canvas, &mut positions, &mut rng, x, y);
                }
            }
        }
    }

    positions
}

fn add_foliage_cluster(
    canvas: &mut [Vec<Cell>],
    positions: &mut Vec<(usize, usize)>,
    rng: &mut StableRng,
    cluster: FoliageCluster,
) {
    let mut placed = 0;
    let mut attempts = 0;
    while placed < cluster.budget && attempts < cluster.budget * 8 {
        attempts += 1;
        let ox = random_isize(rng, -cluster.radius_x, cluster.radius_x);
        let oy = random_isize(rng, -cluster.radius_y, cluster.radius_y / 2);
        let normalized_x = ox as f64 / cluster.radius_x.max(1) as f64;
        let normalized_y = oy as f64 / cluster.radius_y.max(1) as f64;
        if normalized_x * normalized_x + normalized_y * normalized_y > 1.15 {
            continue;
        }
        if put_foliage(
            canvas,
            positions,
            rng,
            cluster.center_x + ox,
            cluster.center_y + oy,
        ) {
            placed += 1;
        }
    }
}

fn put_foliage(
    canvas: &mut [Vec<Cell>],
    positions: &mut Vec<(usize, usize)>,
    rng: &mut StableRng,
    x: isize,
    y: isize,
) -> bool {
    if !in_canvas(canvas, x, y) {
        return false;
    }
    let ch = FOLIAGE[(rng.next_u64() as usize) % FOLIAGE.len()];
    put_cell_isize(canvas, x, y, ch, Style::Foliage);
    positions.push((x as usize, y as usize));
    true
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

fn put_cell_isize(canvas: &mut [Vec<Cell>], x: isize, y: isize, ch: char, style: Style) {
    if in_canvas(canvas, x, y) {
        put_cell(canvas, x as usize, y as usize, ch, style);
    }
}

fn in_canvas(canvas: &[Vec<Cell>], x: isize, y: isize) -> bool {
    y >= 0
        && (y as usize) < canvas.len()
        && x >= 0
        && canvas
            .get(y as usize)
            .is_some_and(|row| (x as usize) < row.len())
}

fn random_range(rng: &mut StableRng, min: f64, max: f64) -> f64 {
    let unit = rng.next_u64() as f64 / u64::MAX as f64;
    min + (max - min) * unit
}

fn random_isize(rng: &mut StableRng, min: isize, max: isize) -> isize {
    let span = (max - min + 1).max(1) as u64;
    min + (rng.next_u64() % span) as isize
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
        .filter(|(_, leaf)| leaf.pressed)
        .map(|(index, leaf)| format!("{} {}", marker_char(*index), leaf.slug))
        .collect::<Vec<_>>();
    let ordinary_values = marker_leaves
        .iter()
        .filter(|(_, leaf)| !leaf.pressed)
        .map(|(index, leaf)| format!("{} {}", marker_char(*index), leaf.slug))
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
        push_styled_line(output, "gold fruit:", Style::Pressed, color);
        push_wrapped_values(output, "  ", &pressed_values, Style::Pressed, color, width);
        if hidden_pressed > 0 {
            push_wrapped_styled_line(
                output,
                &format!(
                    "  {}",
                    hidden_marker_message(hidden_pressed, "pressed leaf", "pressed leaves")
                ),
                Style::Pressed,
                color,
                width,
            );
        }
    }

    if ordinary_count > 0 {
        if pressed_count > 0 {
            output.push('\n');
        }
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
            push_styled_line(output, "green leaves:", Style::Ordinary, color);
            push_wrapped_values(
                output,
                "  ",
                &ordinary_values,
                Style::Ordinary,
                color,
                width,
            );
            if hidden_ordinary > 0 {
                push_wrapped_styled_line(
                    output,
                    &format!(
                        "  {}",
                        hidden_marker_message(hidden_ordinary, "ordinary leaf", "ordinary leaves")
                    ),
                    Style::Ordinary,
                    color,
                    width,
                );
            }
        }
    }

    let hidden = hidden_pressed + hidden_ordinary;
    if hidden > 0 {
        push_wrapped_values(
            output,
            "  ",
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

fn push_sprouts(output: &mut String, sprouts: &[String], color: bool) {
    if sprouts.is_empty() {
        return;
    }

    output.push('\n');
    push_styled_line(output, "active sprouts:", Style::Sprout, color);
    let shown_labels = sprouts
        .iter()
        .take(if sprouts.len() <= 4 { 4 } else { 8 })
        .cloned()
        .collect::<Vec<_>>();
    for sprout in &shown_labels {
        push_styled_line(output, &format!(r"  \|/ {sprout}"), Style::Sprout, color);
    }
    let hidden = sprouts.len().saturating_sub(shown_labels.len());
    if hidden > 0 {
        push_styled_line(
            output,
            &format!("  + {hidden} more active sprouts"),
            Style::Sprout,
            color,
        );
    }
    output.push('\n');
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
        push_wrapped_styled_line(output, label.trim_end(), style, color, width);
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
            if line != prefix {
                push_wrapped_styled_line(output, line.trim_end(), style, color, width);
            } else if !label.trim().is_empty() {
                push_wrapped_styled_line(output, label.trim_end(), style, color, width);
            }
            if visible_len(value) + 2 > width {
                for wrapped in wrap_text(value, width.saturating_sub(2).max(1)) {
                    push_styled_line(output, &format!("  {wrapped}"), style, color);
                }
                line = prefix.clone();
            } else {
                line = format!("  {value}");
            }
        } else {
            line.push_str(separator);
            line.push_str(value);
        }
    }
    if line != prefix && !line.trim().is_empty() {
        push_wrapped_styled_line(output, line.trim_end(), style, color, width);
    }
}

fn push_wrapped_styled_line(
    output: &mut String,
    text: &str,
    style: Style,
    color: bool,
    width: usize,
) {
    if visible_len(text) <= width.max(1) {
        push_styled_line(output, text, style, color);
        return;
    }

    for line in wrap_text(text, width) {
        push_styled_line(output, &line, style, color);
    }
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

fn wrap_text(text: &str, width: usize) -> Vec<String> {
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
        vec![String::new()]
    } else {
        lines
    }
}

fn append_wrapped_word(lines: &mut Vec<String>, current: &mut String, word: &str, width: usize) {
    let word_len = visible_len(word);
    if current.is_empty() && word_len <= width {
        current.push_str(word);
        return;
    }
    if !current.is_empty() && visible_len(current) + 1 + word_len <= width {
        current.push(' ');
        current.push_str(word);
        return;
    }
    if !current.is_empty() {
        lines.push(std::mem::take(current));
    }
    if word_len <= width {
        current.push_str(word);
        return;
    }

    for chunk in split_text_by_width(word, width) {
        if visible_len(&chunk) == width {
            lines.push(chunk);
        } else {
            current.push_str(&chunk);
        }
    }
}

fn split_text_by_width(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut chunks = Vec::new();
    let mut chunk = String::new();
    for ch in text.chars() {
        if visible_len(&chunk) + 1 > width {
            chunks.push(std::mem::take(&mut chunk));
        }
        chunk.push(ch);
    }
    if !chunk.is_empty() {
        chunks.push(chunk);
    }
    chunks
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

    fn max_visible_line_width(input: &str) -> usize {
        strip_ansi(input)
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
    }

    fn widest_run(row: &[Cell], needle: char) -> usize {
        let mut widest = 0;
        let mut current = 0;
        for cell in row {
            if cell.ch == needle {
                current += 1;
                widest = widest.max(current);
            } else {
                current = 0;
            }
        }
        widest
    }

    #[test]
    fn foliage_for_young_tree_clusters_around_branch_tips() {
        let model = model_with_leaves(3, Some(2));
        let tips = vec![(20, 5), (40, 4), (60, 6)];
        let mut canvas = vec![vec![Cell::blank(); 80]; 12];

        let positions = draw_tip_foliage(&mut canvas, &model, &tips, 0xfeed_beef);

        assert!(!positions.is_empty());
        for (x, y) in positions {
            let near_tip = tips
                .iter()
                .any(|(tip_x, tip_y)| x.abs_diff(*tip_x) <= 6 && y.abs_diff(*tip_y) <= 3);
            assert!(
                near_tip,
                "foliage at ({x},{y}) must be near a branch tip, not an independent oval crown"
            );
        }
    }

    #[test]
    fn medium_leaf_count_keeps_trunk_visually_moderate() {
        let canvas = render_canvas(&model_with_leaves(11, Some(1)), 112);
        let trunk_row = &canvas[canvas.len().saturating_sub(3)];
        let widest_trunk = widest_run(trunk_row, '|');

        assert!(
            widest_trunk <= 3,
            "11 leaves should read as a medium tree, not a stump-like trunk; widest run={widest_trunk}"
        );
    }

    #[test]
    fn tree_render_respects_requested_narrow_width() {
        let rendered = render_to_string(
            &model_with_leaves(20, None),
            TreeRenderOptions {
                color: false,
                width: 40,
            },
        );

        assert!(
            max_visible_line_width(&rendered) <= 40,
            "tree output must fit the requested width:\n{rendered}"
        );
        assert!(rendered.contains("green leaves:"));
    }

    #[test]
    fn tree_render_extremely_narrow_width_falls_back_without_overflow() {
        let rendered = render_to_string(
            &model_with_leaves(20, None),
            TreeRenderOptions {
                color: false,
                width: 20,
            },
        );

        assert!(
            max_visible_line_width(&rendered) <= 20,
            "fallback output must fit even tiny widths:\n{rendered}"
        );
        assert!(rendered.contains("tree view too narrow"));
    }

    #[test]
    fn tree_render_splits_long_leaf_names_to_requested_width() {
        let mut model = model_with_leaves(1, None);
        model.leaves[0].slug = "very-long-leaf-name-that-cannot-fit-in-one-narrow-line".to_string();
        let rendered = render_to_string(
            &model,
            TreeRenderOptions {
                color: false,
                width: 40,
            },
        );

        assert!(
            max_visible_line_width(&rendered) <= 40,
            "long names must wrap instead of overflowing:\n{rendered}"
        );
    }

    #[test]
    fn demo_render_respects_requested_narrow_width() {
        let rendered = render_demo_to_string(TreeRenderOptions {
            color: false,
            width: 40,
        });

        assert!(
            max_visible_line_width(&rendered) <= 40,
            "demo output must fit the requested width:\n{rendered}"
        );
        assert!(rendered.contains("0 leaves / seedling"));
        assert!(rendered.contains("50 leaves / mature"));
        assert!(rendered.contains("100 leaves / saturated"));
        assert!(!rendered.contains("5 leaves / small"));
    }

    #[test]
    fn demo_zero_stage_uses_tree_shape_not_sprout_placeholder() {
        let text = render_demo_to_string(TreeRenderOptions {
            color: false,
            width: 112,
        });
        let start = text
            .find("===== 0 leaves / seedling demo =====")
            .expect("zero stage");
        let end = text
            .find("===== 3 leaves / young demo =====")
            .expect("next stage");
        let zero_stage = &text[start..end];

        assert!(
            !zero_stage.contains(r"\|/"),
            "demo stages should use the leaf tree shape, not a seedling placeholder:\n{zero_stage}"
        );
        assert!(
            zero_stage.contains("leaf tree | leaves 0")
                && (zero_stage.contains("/|\\")
                    || zero_stage.contains("||")
                    || zero_stage.contains("/||\\")),
            "zero-leaf demo stage should still look like the same tree family:\n{zero_stage}"
        );
    }

    #[test]
    fn demo_growth_contract_uses_recursive_fractal_branching() {
        let rendered = render_to_string(
            &demo_model(20),
            TreeRenderOptions {
                color: false,
                width: 112,
            },
        );
        let branch_chars = rendered
            .chars()
            .filter(|ch| matches!(ch, '/' | '\\' | '|' | '~'))
            .count();

        assert!(
            rendered.contains("leaf tree | leaves 20")
                && rendered.contains("green leaves:")
                && branch_chars >= 80,
            "demo growth must reuse the full recursive tree renderer, not a manual icon:\n{rendered}"
        );
    }

    #[test]
    fn demo_hundred_stage_has_fuller_crown_than_fifty_stage() {
        let fifty = render_canvas(&demo_model(50), PREFERRED_RENDER_WIDTH);
        let hundred = render_canvas(&demo_model(100), PREFERRED_RENDER_WIDTH);
        let fifty_cells = crown_cell_count(fifty);
        let hundred_cells = crown_cell_count(hundred);

        assert!(
            hundred_cells > fifty_cells + 80,
            "100-leaf saturated demo should visibly fill in beyond the 50-leaf mature stage: 50={fifty_cells}, 100={hundred_cells}"
        );
    }

    #[test]
    fn demo_growth_stages_get_progressively_lusher() {
        let counts = [3, 10, 20, 50, 100];
        let crown_cells = counts
            .into_iter()
            .map(|count| {
                (
                    count,
                    crown_cell_count(render_canvas(&demo_model(count), PREFERRED_RENDER_WIDTH)),
                )
            })
            .collect::<Vec<_>>();

        for window in crown_cells.windows(2) {
            let (previous_count, previous_cells) = window[0];
            let (next_count, next_cells) = window[1];
            assert!(
                next_cells > previous_cells,
                "later demo stage should have a fuller crown: {previous_count} leaves={previous_cells}, {next_count} leaves={next_cells}"
            );
        }

        let lookup = |count| {
            crown_cells
                .iter()
                .find_map(|(item_count, cells)| (*item_count == count).then_some(*cells))
                .expect("demo stage count")
        };
        assert!(
            lookup(10) >= lookup(3) + 120,
            "10-leaf branching stage should be visibly lusher than 3-leaf young stage: {crown_cells:?}"
        );
        assert!(
            lookup(20) >= lookup(10) + 140,
            "20-leaf grown stage should be visibly lusher than 10-leaf branching stage: {crown_cells:?}"
        );
        assert!(
            lookup(50) >= lookup(20) + 300,
            "50-leaf mature stage should approach the lush Python reference: {crown_cells:?}"
        );
        assert!(
            lookup(100) >= lookup(50) + 120,
            "100-leaf saturated stage should still fill in beyond 50 leaves before capping: {crown_cells:?}"
        );
    }

    fn crown_cell_count(canvas: Vec<Vec<Cell>>) -> usize {
        canvas
            .iter()
            .flatten()
            .filter(|cell| {
                matches!(
                    cell.style,
                    Style::Foliage | Style::Ordinary | Style::Pressed
                )
            })
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
    fn active_sprouts_also_render_around_tree_base() {
        let model = TreeModel {
            leaves: vec![TreeLeaf {
                slug: "pressed-leaf".to_string(),
                pressed: true,
            }],
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
        let tree_canvas = rendered
            .split("active sprouts:")
            .next()
            .expect("tree canvas before sprouts section");

        assert!(
            tree_canvas.contains(r"\|/"),
            "tree canvas should show active sprouts around the tree base:\n{rendered}"
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
