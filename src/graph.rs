use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;

const RELATION_PREDICATES: &[&str] = &[
    "cites",
    "refines",
    "supersedes",
    "depends_on",
    "derived_from",
    "related_to",
];

#[derive(Debug, Serialize)]
pub(crate) struct KnowledgeGraph {
    leaf_root: String,
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<GraphWarning>,
}

#[derive(Debug, Serialize)]
struct GraphNode {
    id: String,
    #[serde(rename = "type")]
    concept_type: String,
    slug: String,
    title: Option<String>,
    description: Option<String>,
    resource: Option<String>,
    tags: Vec<String>,
    path: String,
}

#[derive(Debug, Serialize)]
struct GraphEdge {
    source: String,
    predicate: String,
    target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    path: String,
}

#[derive(Debug, Serialize)]
struct GraphWarning {
    path: String,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LinkEdge {
    pub(crate) predicate: String,
    pub(crate) target: String,
    pub(crate) note: Option<String>,
}

pub(crate) fn load(repo_root: &Path) -> Result<KnowledgeGraph> {
    let leaf_root = repo_root.join(".leaf");
    if !leaf_root.is_dir() {
        bail!(".leaf/ is not initialized in this git repository\nhint: run `leaf init`");
    }

    let leaves_dir = leaf_root.join("02-leaves");
    let entries = match fs::read_dir(&leaves_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(KnowledgeGraph {
                leaf_root: ".leaf".to_string(),
                nodes: Vec::new(),
                edges: Vec::new(),
                warnings: Vec::new(),
            });
        }
        Err(err) => return Err(err).context(format!("failed to read {}", leaves_dir.display())),
    };

    let mut leaf_dirs = Vec::new();
    for entry in entries {
        let entry = entry.context("failed to read leaf entry")?;
        if entry
            .file_type()
            .context("failed to inspect leaf entry")?
            .is_dir()
        {
            leaf_dirs.push(entry.path());
        }
    }
    leaf_dirs.sort();

    let mut graph = KnowledgeGraph {
        leaf_root: ".leaf".to_string(),
        nodes: Vec::new(),
        edges: Vec::new(),
        warnings: Vec::new(),
    };

    for leaf_dir in leaf_dirs {
        let Some(slug) = leaf_dir
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        load_pressed_leaf(repo_root, &leaf_dir, &slug, &mut graph);
    }

    Ok(graph)
}

pub(crate) fn write_json(writer: &mut impl Write, graph: &KnowledgeGraph) -> Result<()> {
    serde_json::to_writer_pretty(&mut *writer, graph)?;
    writeln!(writer)?;
    Ok(())
}

pub(crate) fn write_text(writer: &mut impl Write, graph: &KnowledgeGraph) -> Result<()> {
    writeln!(writer, "leaf graph")?;
    writeln!(writer)?;
    writeln!(writer, "nodes    {}", graph.nodes.len())?;
    writeln!(writer, "edges    {}", graph.edges.len())?;
    if !graph.warnings.is_empty() {
        writeln!(writer, "warnings {}", graph.warnings.len())?;
    }
    for node in &graph.nodes {
        writeln!(
            writer,
            "- {}  {}",
            node.id,
            node.title.as_deref().unwrap_or(&node.slug)
        )?;
    }
    Ok(())
}

fn load_pressed_leaf(repo_root: &Path, leaf_dir: &Path, slug: &str, graph: &mut KnowledgeGraph) {
    let pressed_path = leaf_dir.join("pressed.md");
    if !pressed_path.is_file() {
        return;
    }

    let rel_pressed = repo_relative(repo_root, &pressed_path);
    let content = match fs::read_to_string(&pressed_path) {
        Ok(content) => content,
        Err(err) => {
            graph.warnings.push(GraphWarning {
                path: rel_pressed,
                message: format!("failed to read pressed.md: {err}"),
            });
            return;
        }
    };

    let frontmatter = parse_yaml_frontmatter(&content);
    let node_id = frontmatter
        .and_then(|frontmatter| frontmatter_value(frontmatter, "citation_handle"))
        .unwrap_or_else(|| format!("leaf:{slug}"));
    let concept_type = frontmatter
        .and_then(|frontmatter| frontmatter_value(frontmatter, "type"))
        .unwrap_or_else(|| "Leaf Pressed Digest".to_string());

    let node = GraphNode {
        id: node_id.clone(),
        concept_type,
        slug: slug.to_string(),
        title: frontmatter.and_then(|frontmatter| frontmatter_value(frontmatter, "title")),
        description: frontmatter
            .and_then(|frontmatter| frontmatter_value(frontmatter, "description")),
        resource: frontmatter.and_then(|frontmatter| frontmatter_value(frontmatter, "resource")),
        tags: frontmatter
            .and_then(|frontmatter| frontmatter_value(frontmatter, "tags"))
            .map(|value| parse_yaml_inline_list(&value))
            .unwrap_or_default(),
        path: rel_pressed,
    };
    graph.nodes.push(node);

    let linked_path = leaf_dir.join("linked.md");
    if !linked_path.is_file() {
        return;
    }
    let rel_linked = repo_relative(repo_root, &linked_path);
    let linked = match fs::read_to_string(&linked_path) {
        Ok(content) => content,
        Err(err) => {
            graph.warnings.push(GraphWarning {
                path: rel_linked,
                message: format!("failed to read linked.md: {err}"),
            });
            return;
        }
    };

    for (line_number, line) in linked.lines().enumerate() {
        match parse_link_line(line) {
            Ok(Some(edge)) => graph.edges.push(GraphEdge {
                source: node_id.clone(),
                predicate: edge.predicate,
                target: edge.target,
                note: edge.note,
                path: format!("{rel_linked}:{}", line_number + 1),
            }),
            Ok(None) => {}
            Err(message) => graph.warnings.push(GraphWarning {
                path: format!("{rel_linked}:{}", line_number + 1),
                message,
            }),
        }
    }
}

pub(crate) fn parse_link_line(line: &str) -> std::result::Result<Option<LinkEdge>, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || !trimmed.starts_with('-') {
        return Ok(None);
    }

    let body = trimmed.trim_start_matches('-').trim();
    if !body.contains("->") {
        return Ok(None);
    }

    let Some((raw_predicate, raw_target_and_note)) = body.split_once("->") else {
        return Err("link must use `predicate -> target`".to_string());
    };
    let predicate = trim_code(raw_predicate.trim()).to_string();
    if predicate.is_empty() {
        return Err("link predicate is empty".to_string());
    }
    if !RELATION_PREDICATES.contains(&predicate.as_str()) {
        return Err(format!(
            "unknown link predicate {predicate:?}; expected one of {}",
            RELATION_PREDICATES.join(", ")
        ));
    }

    let (raw_target, raw_note) = split_target_note(raw_target_and_note.trim());
    let target = trim_code(raw_target.trim()).to_string();
    if target.is_empty() {
        return Err("link target is empty".to_string());
    }
    let note = raw_note
        .map(str::trim)
        .filter(|note| !note.is_empty())
        .map(str::to_string);

    Ok(Some(LinkEdge {
        predicate,
        target,
        note,
    }))
}

fn split_target_note(value: &str) -> (&str, Option<&str>) {
    if let Some((target, note)) = value.split_once(" - ") {
        (target, Some(note))
    } else {
        (value, None)
    }
}

fn trim_code(value: &str) -> &str {
    if value.len() >= 2 && value.starts_with('`') && value.ends_with('`') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn parse_yaml_frontmatter(content: &str) -> Option<&str> {
    let mut lines = content.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }

    let start = content.find('\n').map_or(content.len(), |index| index + 1);
    let mut offset = start;
    for line in lines {
        if line.trim() == "---" {
            return content.get(start..offset);
        }
        offset += line.len();
        if content.as_bytes().get(offset) == Some(&b'\n') {
            offset += 1;
        }
    }

    None
}

fn frontmatter_value(frontmatter: &str, key: &str) -> Option<String> {
    for line in frontmatter.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once(':') else {
            continue;
        };
        if raw_key.trim() != key {
            continue;
        }
        return Some(unquote_yaml_scalar(raw_value.trim()).to_string());
    }

    None
}

fn unquote_yaml_scalar(value: &str) -> &str {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn parse_yaml_inline_list(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(trimmed);
    inner
        .split(',')
        .map(|value| unquote_yaml_scalar(value.trim()).to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn repo_relative(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn parse_link_line_accepts_minimal_relation() {
        let edge = parse_link_line("- `refines` -> `leaf:pressed-format` - narrows it")
            .expect("parsed")
            .expect("edge");

        assert_eq!(edge.predicate, "refines");
        assert_eq!(edge.target, "leaf:pressed-format");
        assert_eq!(edge.note.as_deref(), Some("narrows it"));
    }

    #[test]
    fn parse_link_line_rejects_unknown_predicate() {
        let message = parse_link_line("- `causes` -> `leaf:other`").expect_err("unknown predicate");

        assert!(message.contains("unknown link predicate"));
    }

    #[test]
    fn load_exports_pressed_nodes_and_link_edges() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/02-leaves/reference")
            .create_dir_all()
            .expect("leaf dir");
        root.child(".leaf/02-leaves/reference/pressed.md")
            .write_str(
                "---\n\
                 type: Leaf Pressed Digest\n\
                 title: Reference Leaf\n\
                 description: One-sentence summary.\n\
                 resource: .leaf/02-leaves/reference\n\
                 tags: [leaf, reference]\n\
                 timestamp: 2026-06-22T10:00:00+09:00\n\
                 citation_handle: leaf:reference\n\
                 stage: leaf\n\
                 ---\n\
                 \n\
                 # Reference Leaf\n",
            )
            .expect("pressed");
        root.child(".leaf/02-leaves/reference/linked.md")
            .write_str("# Links\n\n- `cites` -> `okf:spec` - Source format\n")
            .expect("links");

        let graph = load(root.path()).expect("graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id, "leaf:reference");
        assert_eq!(graph.nodes[0].tags, vec!["leaf", "reference"]);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].source, "leaf:reference");
        assert_eq!(graph.edges[0].predicate, "cites");
        assert_eq!(graph.edges[0].target, "okf:spec");
    }
}
