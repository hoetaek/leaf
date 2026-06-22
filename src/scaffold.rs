use crate::phase::{POLISH_PENDING_BLOCK, Phase};
use anyhow::{Context, Result, bail};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// `00-status.md` body, written once at sprout creation. Phase gate files are
/// scaffolded lazily, one phase at a time, by [`scaffold_phase`].
const STATUS_TEMPLATE: &str = "# Sprout Status\n\n\
     - why: TODO state the problem this LEAF exists to solve — keep it sharp\n\
     - what: TODO name the deliverable this LEAF is aiming at (or `none — <reason>`)\n\
     - wireframe: TODO name the cheap-preview form of that deliverable (or `none — <reason>`)\n\
     - stage: sprout\n\
     - current phase: Learn\n\
     - current gate: ① Intent\n\
     - first missing gate: ① Intent\n\
     - next action: draft the one-sentence intent in 01-Learn/01-intent.md\n\n\
     ## Overview\n\n\
     - request: TODO capture the user's request in the user's words\n\
     - current scope: TODO state what is included, excluded, split, or still undecided\n\
     - consistency rule: update why / what / wireframe and this overview whenever intent, scope, output, or gate files change what this LEAF is doing\n\n\
     ## Document Map\n\n\
     - ① Intent: `01-Learn/01-intent.md`\n\
     - ② Unknowns & Context: `01-Learn/02-unknowns.md`\n\
     - ③ Criteria: `02-Example/03-criteria.md`\n\
     - ④ Wireframe: `02-Example/04-wireframe.md`\n\
     - ⑤ Design / ⑦ Tasks: `03-Architect/05-design.md`, `03-Architect/07-tasks.md`\n\
     - ⑨ Review / ⑩ Retrospect: `04-Feedback/`\n";

const UNKNOWNS_BODY: &str = "# Unknowns And Context\n\n\
     ② is gathered by four parallel scouts; keep what each finds under the\n\
     matching heading, then summarize the load-bearing answers here with their\n\
     source.\n\n\
     ## A. Terrain — what exists (refs, concepts, internal assets, tools)\n\n\
     ## B. Method — how it's done (best practice, cases, anti-patterns)\n\n\
     ## C. Judgment — where it forks (trade-offs, debates, hidden premises)\n\n\
     ## D. Context — why it's this way (history, recent change, analogies, stakeholders)\n";

const REFERENCES_BODY: &str = "# References\n\n\
     Learn always builds context here — this is not a lazy, fill-when-stuck\n\
     folder. ② runs as four parallel scouts, each writing what it finds to this\n\
     folder, one file per topic named for what it covers:\n\n\
     - A. Terrain — what exists: external refs (comparable cases, prior art,\n\
       authoritative sources), domain concepts, internal assets (your own\n\
       documents, data, prior decisions), tools.\n\
     - B. Method — how it's done: best practices, real cases/benchmarks,\n\
       failure cases and anti-patterns.\n\
     - C. Judgment — where it forks: trade-offs and selection criteria, live\n\
       debates, hidden premises. Never skip this one — it is what turns\n\
       collected material into a decision.\n\
     - D. Context — why it's this way: history, recent changes, analogies from\n\
       adjacent fields, stakeholders.\n\n\
     Each scout returns grounds (what it found and where), not a verdict. Then\n\
     summarize only what the work truly needs out into ../02-unknowns.md, with\n\
     its source, as a reading map rather than an answer.\n";

const FEEDBACK_BODY: &str = "# Feedback\n\n\
     이 phase의 exit는 close-out이다. ⑩ Retrospect 후 keep/press/fall 결정 전에, 누적 전체\n\
     (Learn→Feedback)를 하나의 보고서로 `leaf:polish`하여 최종 산출물이 draft 상태로\n\
     마감되지 않게 한다. ⑨ Review는 `09-review.md`, ⑩ Retrospect는 `10-retrospect.md`에 쓴다.\n";

/// Directories created when a phase is scaffolded, relative to the sprout root.
fn phase_dirs(phase: Phase) -> &'static [&'static str] {
    match phase {
        Phase::Learn => &["01-Learn", "01-Learn/02-references"],
        Phase::Example => &["02-Example"],
        Phase::Architect => &["03-Architect"],
        Phase::Feedback => &["04-Feedback"],
    }
}

/// Gate files created when a phase is scaffolded. The **first** entry is the
/// phase's marker file: [`scaffold_phase`] prepends [`POLISH_PENDING_BLOCK`] to
/// it so the phase starts dirty until `leaf:polish` removes the token.
fn phase_files(phase: Phase) -> &'static [(&'static str, &'static str)] {
    match phase {
        Phase::Learn => &[
            (
                "01-Learn/01-intent.md",
                "# Intent\n\nCapture the raw idea and the current one-sentence intent here.\n",
            ),
            ("01-Learn/02-unknowns.md", UNKNOWNS_BODY),
            ("01-Learn/02-references/README.md", REFERENCES_BODY),
        ],
        Phase::Example => &[
            (
                "02-Example/03-criteria.md",
                "# Criteria\n\nState purpose, constraints, and observable acceptance checks here.\n",
            ),
            (
                "02-Example/04-wireframe.md",
                "# Wireframe\n\nUse a concrete text-first example before generalizing the work.\n",
            ),
        ],
        Phase::Architect => &[
            (
                "03-Architect/05-design.md",
                "# Design\n\nRecord the implementation-facing design after the concrete example holds.\n",
            ),
            (
                "03-Architect/07-tasks.md",
                "# Tasks\n\nBreak the work into reviewable implementation slices.\n",
            ),
        ],
        Phase::Feedback => &[("04-Feedback/README.md", FEEDBACK_BODY)],
    }
}

pub(crate) fn create_sprout(repo_root: &Path, slug: &str) -> Result<PathBuf> {
    let leaf_root = repo_root.join(".leaf");
    let sprout = leaf_root
        .join(crate::inventory::Stage::Sprout.dir_name())
        .join(slug);
    for stage in [
        crate::inventory::Stage::Leaf,
        crate::inventory::Stage::Fallen,
    ] {
        let existing = leaf_root.join(stage.dir_name()).join(slug);
        if existing.is_dir() {
            bail!(
                "leaf slug already exists in lifecycle stage: {}",
                existing.display()
            );
        }
    }

    match fs::create_dir(&sprout) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            bail!("leaf sprout already exists: {slug}");
        }
        Err(err) => {
            return Err(err)
                .with_context(|| format!("failed to create sprout {}", sprout.display()));
        }
    }

    write_new_file(&sprout.join("00-status.md"), STATUS_TEMPLATE)?;
    // Lazy scaffold: only the Learn phase exists at creation. `leaf next`
    // grows each later phase, pausing at the boundary if the leaving phase is
    // still unpolished.
    scaffold_phase(&sprout, Phase::Learn)?;

    Ok(sprout)
}

/// Create one phase's directories and gate files under `sprout`. Idempotent:
/// existing files are left untouched, so re-running `leaf next` is safe. The
/// first gate file of the phase is seeded with the polish-pending marker.
pub(crate) fn scaffold_phase(sprout: &Path, phase: Phase) -> Result<()> {
    for directory in phase_dirs(phase) {
        let path = sprout.join(directory);
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create directory {}", path.display()))?;
    }

    for (index, (relative_path, body)) in phase_files(phase).iter().enumerate() {
        let path = sprout.join(relative_path);
        if path.exists() {
            continue;
        }
        let content = if index == 0 {
            format!("{POLISH_PENDING_BLOCK}{body}")
        } else {
            (*body).to_string()
        };
        write_new_file(&path, &content)?;
    }

    Ok(())
}

fn write_new_file(path: &Path, body: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("failed to create file {}", path.display()))?;
    file.write_all(body.as_bytes())
        .with_context(|| format!("failed to write file {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{ParseState, StageDir, parse_status_summary};

    /// A freshly scaffolded sprout must satisfy every status field `leaf doctor`
    /// requires for the sprout stage, so `leaf new` followed by `leaf doctor`
    /// reports no `status_missing_fields` warning.
    #[test]
    fn sprout_status_template_has_all_doctor_required_fields() {
        let summary = parse_status_summary(STATUS_TEMPLATE, StageDir::Sprouts);

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert!(
            summary.missing_fields.is_empty(),
            "fresh sprout status is missing doctor-required fields: {:?}",
            summary.missing_fields
        );
        assert_eq!(summary.stage.as_deref(), Some("sprout"));
        assert!(summary.legacy_state.is_none());
        assert_eq!(summary.current_phase.as_deref(), Some("Learn"));
        assert_eq!(summary.current_gate.as_deref(), Some("① Intent"));
    }

    #[test]
    fn sprout_status_template_has_live_overview_section() {
        assert!(
            STATUS_TEMPLATE.contains("## Overview"),
            "00-status.md should summarize what this LEAF is doing"
        );
        assert!(
            STATUS_TEMPLATE.contains("- request:"),
            "overview should preserve the user's request at status level"
        );
        assert!(
            STATUS_TEMPLATE.contains("- why:")
                && STATUS_TEMPLATE.contains("- what:")
                && STATUS_TEMPLATE.contains("- wireframe:"),
            "status should carry the locked why / what / wireframe triple"
        );
        assert!(
            STATUS_TEMPLATE.contains("- consistency rule:"),
            "overview should remind agents to keep status and gate docs aligned"
        );
    }

    #[test]
    fn sprout_status_template_surfaces_triple_within_preview_budget() {
        // Mirror preview::useful_lines with STATUS_PREVIEW_LINES = 8: trim,
        // drop blank lines, take the first 8. The triple must land inside it.
        let preview: String = STATUS_TEMPLATE
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .take(8)
            .collect::<Vec<_>>()
            .join("\n");

        for label in ["- why:", "- what:", "- wireframe:"] {
            assert!(
                preview.contains(label),
                "`{label}` must be within the 8-line TUI preview budget; got:\n{preview}"
            );
        }
    }

    #[test]
    fn create_sprout_scaffolds_only_learn_phase() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path();
        fs::create_dir(repo_root.join(".leaf")).expect(".leaf");
        fs::create_dir(repo_root.join(".leaf/01-sprouts")).expect("sprouts dir");

        let sprout = create_sprout(repo_root, "demo").expect("create sprout");

        assert!(sprout.join("00-status.md").is_file());
        assert!(sprout.join("01-Learn/01-intent.md").is_file());
        assert!(
            sprout.join("01-Learn/02-references/README.md").is_file(),
            "Learn references folder is scaffolded"
        );
        // Lazy: later phases do not exist yet.
        assert!(!sprout.join("02-Example").exists(), "Example is lazy");
        assert!(!sprout.join("03-Architect").exists(), "Architect is lazy");
        assert!(!sprout.join("04-Feedback").exists(), "Feedback is lazy");
    }

    #[test]
    fn first_gate_file_carries_polish_pending_token() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path();
        fs::create_dir(repo_root.join(".leaf")).expect(".leaf");
        fs::create_dir(repo_root.join(".leaf/01-sprouts")).expect("sprouts dir");

        let sprout = create_sprout(repo_root, "demo").expect("create sprout");

        let intent = fs::read_to_string(sprout.join("01-Learn/01-intent.md")).expect("read intent");
        assert!(
            intent.lines().next().expect("first line").trim_start() == POLISH_PENDING_TOKEN_LINE,
            "Learn's first gate file starts with the polish-pending token"
        );
        // Non-first gate files are not seeded with the token.
        let unknowns =
            fs::read_to_string(sprout.join("01-Learn/02-unknowns.md")).expect("read unknowns");
        assert!(!unknowns.contains(POLISH_PENDING_TOKEN_LINE));
    }

    #[test]
    fn scaffold_phase_is_idempotent_and_seeds_example_token() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path();
        fs::create_dir(repo_root.join(".leaf")).expect(".leaf");
        fs::create_dir(repo_root.join(".leaf/01-sprouts")).expect("sprouts dir");
        let sprout = create_sprout(repo_root, "demo").expect("create sprout");

        scaffold_phase(&sprout, Phase::Example).expect("scaffold example");
        let criteria =
            fs::read_to_string(sprout.join("02-Example/03-criteria.md")).expect("read criteria");
        assert!(criteria.contains(POLISH_PENDING_TOKEN_LINE));

        // Editing then re-scaffolding must not clobber existing content.
        fs::write(sprout.join("02-Example/03-criteria.md"), "edited").expect("edit");
        scaffold_phase(&sprout, Phase::Example).expect("re-scaffold example");
        let after =
            fs::read_to_string(sprout.join("02-Example/03-criteria.md")).expect("read criteria");
        assert_eq!(after, "edited", "existing gate file is left untouched");
    }

    const POLISH_PENDING_TOKEN_LINE: &str = crate::phase::POLISH_PENDING_TOKEN;
}
