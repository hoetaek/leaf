use crate::date;
use crate::doctor::srp_sidecar;
use crate::git::RepoPaths;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Component, Path};
use std::time::SystemTime;
use unicode_width::UnicodeWidthStr;

/// Author and maintain SRP sidecar contracts (`*.leaf.local.toml`). Validation
/// stays in `leaf doctor`; this command only creates and refreshes the files.
#[derive(Debug, clap::Subcommand)]
pub(crate) enum SidecarCommand {
    /// Scaffold <artifact>.leaf.local.toml next to an existing artifact.
    New {
        /// Repo-relative path to the artifact the sidecar documents.
        artifact: String,
        /// One-sentence single responsibility (defaults to a TODO placeholder).
        #[arg(long)]
        responsibility: Option<String>,
    },
    /// Re-record last_verified to today, clearing the stale warning.
    Verify {
        /// Repo-relative path to the documented artifact.
        artifact: String,
    },
    /// List every sidecar with its freshness.
    List {
        /// Write machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

pub(crate) fn run(command: SidecarCommand, paths: &RepoPaths, now: SystemTime) -> Result<()> {
    match command {
        SidecarCommand::New {
            artifact,
            responsibility,
        } => new(paths, &artifact, responsibility.as_deref(), now),
        SidecarCommand::Verify { artifact } => verify(paths, &artifact, now),
        SidecarCommand::List { json } => list(paths, json),
    }
}

fn new(
    paths: &RepoPaths,
    artifact: &str,
    responsibility: Option<&str>,
    now: SystemTime,
) -> Result<()> {
    // Normalize to the same repo-relative form `doctor` derives from the
    // filename, so the written `artifact` field never mismatches its pairing.
    let artifact = normalize_artifact(artifact)?;
    if !paths.root.join(&artifact).exists() {
        bail!("artifact does not exist: {artifact}");
    }
    let sidecar_rel = format!("{artifact}{}", srp_sidecar::SUFFIX);
    let sidecar_path = paths.root.join(&sidecar_rel);
    if sidecar_path.exists() {
        bail!("sidecar already exists: {sidecar_rel}");
    }

    // Ensure the git-exclude rules (same set as `leaf init`) so the sidecar
    // stays out of git status; no `.leaf/` directory is created.
    crate::storage::ensure_exclude_lines(&paths.exclude)?;

    let today = date::today_utc(now)?;
    let responsibility = responsibility
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let is_todo = responsibility.is_none();
    let responsibility_text = responsibility
        .map(str::to_string)
        .unwrap_or_else(|| format!("TODO: {artifact}의 단일 책임을 한 문장으로"));

    let body = scaffold_body(&artifact, &responsibility_text, &today);
    crate::scaffold::write_new_file(&sidecar_path, &body)
        .with_context(|| format!("failed to create sidecar {sidecar_rel}"))?;

    println!("created {sidecar_rel}");
    if is_todo {
        eprintln!("⚠ responsibility가 TODO입니다 — 채운 뒤 leaf sidecar verify로 재확인하세요.");
    }
    Ok(())
}

fn verify(paths: &RepoPaths, artifact: &str, now: SystemTime) -> Result<()> {
    let sidecar_rel = format!("{artifact}{}", srp_sidecar::SUFFIX);
    let sidecar_path = paths.root.join(&sidecar_rel);
    if !sidecar_path.exists() {
        bail!("no sidecar at {sidecar_rel}");
    }

    let content = fs::read_to_string(&sidecar_path)
        .with_context(|| format!("failed to read sidecar {sidecar_rel}"))?;
    // verify is not a validator; refuse a malformed file instead of claiming success.
    if toml::from_str::<toml::Value>(&content).is_err() {
        bail!("sidecar is not valid; run leaf doctor");
    }
    let today = date::today_utc(now)?;
    let updated = replace_last_verified(&content, &today)?;
    // Writing the file last bumps its mtime past the artifact, clearing stale.
    fs::write(&sidecar_path, updated)
        .with_context(|| format!("failed to write sidecar {sidecar_rel}"))?;

    println!("✓ 재확인: last_verified → {today} (stale 해소)");
    eprintln!("  계약 내용이 v1에 맞는지는 leaf doctor로 확인하세요 — verify는 검증기가 아닙니다.");
    Ok(())
}

#[derive(Serialize)]
struct SidecarRow {
    artifact: String,
    sidecar: String,
    last_verified: Option<String>,
    state: &'static str,
}

fn list(paths: &RepoPaths, json: bool) -> Result<()> {
    let scan = srp_sidecar::collect_sidecar_paths(&paths.root);
    let mut rows: Vec<SidecarRow> = scan
        .sidecars
        .iter()
        .map(|sidecar_path| classify(&paths.root, sidecar_path))
        .collect();
    rows.sort_by(|a, b| a.artifact.cmp(&b.artifact));

    if json {
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        serde_json::to_writer_pretty(&mut stdout, &rows).context("failed to write sidecar JSON")?;
        writeln!(stdout).context("failed to write sidecar JSON")?;
        return Ok(());
    }

    if rows.is_empty() {
        println!("no sidecars found");
        return Ok(());
    }

    // Pad by display width, not byte or char count: multibyte artifact paths
    // (e.g. Korean) render two columns per glyph, so `{:<width$}` alone — which
    // counts chars — would misalign the table.
    let width = rows
        .iter()
        .map(|row| row.artifact.width())
        .max()
        .unwrap_or(0)
        .max("ARTIFACT".width());
    println!(
        "{}  {:<13}  STATE",
        pad_display("ARTIFACT", width),
        "LAST_VERIFIED"
    );
    for row in &rows {
        let last_verified = row.last_verified.as_deref().unwrap_or("-");
        println!(
            "{}  {last_verified:<13}  {}",
            pad_display(&row.artifact, width),
            row.state
        );
    }

    let count = |state: &str| rows.iter().filter(|row| row.state == state).count();
    let mut summary = format!("\n{} sidecars · {} stale", rows.len(), count("stale"));
    if count("missing") > 0 {
        summary.push_str(&format!(" · {} missing", count("missing")));
    }
    if count("broken") > 0 {
        summary.push_str(&format!(" · {} broken", count("broken")));
    }
    println!("{summary}");
    Ok(())
}

/// Left-pad `text` with spaces to occupy `width` terminal columns, measuring by
/// display width so multibyte glyphs (which render two columns) line up.
fn pad_display(text: &str, width: usize) -> String {
    let pad = width.saturating_sub(text.width());
    format!("{text}{}", " ".repeat(pad))
}

fn classify(root: &Path, sidecar_path: &Path) -> SidecarRow {
    let sidecar_rel = repo_relative(root, sidecar_path);
    let artifact_rel = sidecar_rel
        .strip_suffix(srp_sidecar::SUFFIX)
        .unwrap_or(&sidecar_rel)
        .to_string();

    let parsed = fs::read_to_string(sidecar_path)
        .ok()
        .and_then(|content| toml::from_str::<toml::Value>(&content).ok());
    let Some(document) = parsed else {
        return SidecarRow {
            artifact: artifact_rel,
            sidecar: sidecar_rel,
            last_verified: None,
            state: "broken",
        };
    };

    let last_verified = document
        .get("last_verified")
        .and_then(toml::Value::as_str)
        .map(str::to_string);
    let artifact_path = root.join(&artifact_rel);
    let state = if !artifact_path.exists() {
        "missing"
    } else if srp_sidecar::is_stale(sidecar_path, &artifact_path) {
        "stale"
    } else {
        "fresh"
    };

    SidecarRow {
        artifact: artifact_rel,
        sidecar: sidecar_rel,
        last_verified,
        state,
    }
}

/// v1 SRP contract, fields ordered for a human reader (schema first). Serde
/// emits struct fields in declaration order, unlike a `toml::Table` (BTreeMap),
/// which would sort them alphabetically.
#[derive(Serialize)]
struct ScaffoldContract<'a> {
    schema: &'a str,
    artifact: &'a str,
    status: &'a str,
    last_verified: &'a str,
    responsibility: &'a str,
}

fn scaffold_body(artifact: &str, responsibility: &str, today: &str) -> String {
    // Serialize through the `toml` crate so every value is correctly escaped
    // (control chars, quotes, backslashes) — a hand-rolled formatter is not.
    let contract = ScaffoldContract {
        schema: srp_sidecar::SCHEMA,
        artifact,
        status: "advisory",
        last_verified: today,
        responsibility,
    };
    toml::to_string(&contract).expect("sidecar scaffold serializes to TOML")
}

/// Rewrite the single `last_verified = "..."` line to `today`, preserving the
/// indentation and anything trailing (e.g. a comment). Bails when no such line
/// exists — `verify` is not a validator, so it routes malformed files to
/// `leaf doctor` instead of guessing.
fn replace_last_verified(content: &str, today: &str) -> Result<String> {
    let mut out = String::with_capacity(content.len());
    let mut replaced = false;
    for line in content.split_inclusive('\n') {
        if !replaced && let Some(rewritten) = rewrite_last_verified_line(line, today) {
            out.push_str(&rewritten);
            replaced = true;
        } else {
            out.push_str(line);
        }
    }
    if !replaced {
        bail!("sidecar is not valid; run leaf doctor");
    }
    Ok(out)
}

fn rewrite_last_verified_line(line: &str, today: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let rest = trimmed.strip_prefix("last_verified")?.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    // First `"` closes the value; v1 `last_verified` is a date with no inner quotes.
    let close = rest.find('"')?;
    let after = &rest[close + 1..];
    Some(format!("{indent}last_verified = \"{today}\"{after}"))
}

fn repo_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Canonicalize an artifact argument to a repo-relative `a/b/c` string: drop
/// `./` components, reject anything that escapes the repo (`..`, absolute).
/// This matches the form `doctor` derives from the sidecar filename.
fn normalize_artifact(artifact: &str) -> Result<String> {
    let mut parts = Vec::new();
    for component in Path::new(artifact).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            _ => bail!("artifact path must be inside the repo: {artifact}"),
        }
    }
    if parts.is_empty() {
        bail!("artifact path must be inside the repo: {artifact}");
    }
    Ok(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_body_is_valid_v1_with_required_fields() {
        let body = scaffold_body("src/foo.rs", "Owns foo.", "2026-06-29");
        assert!(body.contains("schema = \"leaf.srp-sidecar.v1\""));
        assert!(body.contains("artifact = \"src/foo.rs\""));
        assert!(body.contains("status = \"advisory\""));
        assert!(body.contains("last_verified = \"2026-06-29\""));
        assert!(body.contains("responsibility = \"Owns foo.\""));
        let _: toml::Value = toml::from_str(&body).expect("scaffold is valid TOML");
        // Fields keep declaration order (schema first), not alphabetical order.
        let positions: Vec<usize> = [
            "schema",
            "artifact",
            "status",
            "last_verified",
            "responsibility",
        ]
        .into_iter()
        .map(|key| body.find(key).expect("field present"))
        .collect();
        assert!(
            positions.windows(2).all(|pair| pair[0] < pair[1]),
            "scaffold fields must stay in declaration order: {body}"
        );
    }

    #[test]
    fn pad_display_accounts_for_wide_glyphs() {
        // A 2-column Korean glyph plus padding to width 4 yields no extra space;
        // an ASCII char to the same width gets three trailing spaces.
        assert_eq!(pad_display("가", 4).width(), 4);
        assert_eq!(pad_display("a", 4), "a   ");
    }

    #[test]
    fn replace_last_verified_swaps_value_and_keeps_trailing_comment() {
        let original =
            "schema = \"x\"\nlast_verified = \"2000-01-01\"  # checked\nresponsibility = \"y\"\n";
        let updated = replace_last_verified(original, "2026-06-29").expect("replaced");
        assert!(updated.contains("last_verified = \"2026-06-29\""));
        assert!(updated.contains("# checked"));
        assert!(!updated.contains("2000-01-01"));
        assert!(updated.contains("schema = \"x\""));
    }

    #[test]
    fn replace_last_verified_bails_when_line_absent() {
        let original = "schema = \"x\"\nresponsibility = \"y\"\n";
        assert!(replace_last_verified(original, "2026-06-29").is_err());
    }

    #[test]
    fn normalize_artifact_strips_curdir_and_rejects_escapes() {
        assert_eq!(
            normalize_artifact("./src/a.rs").expect("dotslash"),
            "src/a.rs"
        );
        assert_eq!(normalize_artifact("src/a.rs").expect("plain"), "src/a.rs");
        assert!(normalize_artifact("../x").is_err());
        assert!(normalize_artifact("/abs").is_err());
    }
}
