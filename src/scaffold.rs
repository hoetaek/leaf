use anyhow::{Context, Result, bail};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const DIRECTORIES: &[&str] = &[
    "01-Learn",
    "01-Learn/02-references",
    "02-Example",
    "03-Architect",
    "04-Feedback",
];

const FILES: &[(&str, &str)] = &[
    (
        "00-status.md",
        "# Sprout Status\n\n\
         - stage: sprout\n\
         - current phase: Learn\n\
         - current gate: ① Intent\n\
         - first missing gate: ① Intent\n\
         - next action: draft the one-sentence intent in 01-Learn/01-intent.md\n\n\
         ## Overview\n\n\
         - request: TODO capture the user's request in the user's words\n\
         - purpose: TODO state why this LEAF should exist\n\
         - expected output: TODO name the artifact, decision, or result this LEAF is aiming at\n\
         - current scope: TODO state what is included, excluded, split, or still undecided\n\
         - consistency rule: update this overview whenever intent, scope, output, or gate files change what this LEAF is doing\n\n\
         ## Document Map\n\n\
         - ① Intent: `01-Learn/01-intent.md`\n\
         - ② Unknowns & Context: `01-Learn/02-unknowns.md`\n\
         - ③ Criteria: `02-Example/03-criteria.md`\n\
         - ④ Wireframe: `02-Example/04-wireframe.md`\n\
         - ⑤ Design / ⑦ Tasks: `03-Architect/05-design.md`, `03-Architect/07-tasks.md`\n\
         - ⑨ Review / ⑩ Retrospect: `04-Feedback/`\n",
    ),
    (
        "01-Learn/01-intent.md",
        "# Intent\n\nCapture the raw idea and the current one-sentence intent here.\n",
    ),
    (
        "01-Learn/02-unknowns.md",
        "# Unknowns And Context\n\n## Domain concepts\n\n## Standards and conventions\n\n## External facts\n\n## Internal facts\n",
    ),
    (
        "01-Learn/02-references/README.md",
        "# References\n\nLearn always builds context here — this is not a lazy, fill-when-stuck folder. Pull both external references (comparable cases, prior art, authoritative sources) and internal ones (your own documents, data, prior decisions) into context files, one folder or file per source, kept in a form you can see. Then summarize only what the work truly needs out into ../02-unknowns.md, with its source.\n",
    ),
    (
        "02-Example/03-criteria.md",
        "# Criteria\n\nState purpose, constraints, and observable acceptance checks here.\n",
    ),
    (
        "02-Example/04-wireframe.md",
        "# Wireframe\n\nUse a concrete text-first example before generalizing the work.\n",
    ),
    (
        "03-Architect/05-design.md",
        "# Design\n\nRecord the implementation-facing design after the concrete example holds.\n",
    ),
    (
        "03-Architect/07-tasks.md",
        "# Tasks\n\nBreak the work into reviewable implementation slices.\n",
    ),
];

pub(crate) fn create_sprout(repo_root: &Path, slug: &str) -> Result<PathBuf> {
    let sprout = repo_root
        .join(".leaf")
        .join(crate::inventory::Stage::Sprout.dir_name())
        .join(slug);
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

    for directory in DIRECTORIES {
        let path = sprout.join(directory);
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create directory {}", path.display()))?;
    }

    for (relative_path, body) in FILES {
        write_new_file(&sprout.join(relative_path), body)?;
    }

    Ok(sprout)
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
        let body = FILES
            .iter()
            .find(|(name, _)| *name == "00-status.md")
            .map(|(_, body)| *body)
            .expect("sprout scaffold includes 00-status.md");

        let summary = parse_status_summary(body, StageDir::Sprouts);

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
        let body = FILES
            .iter()
            .find(|(name, _)| *name == "00-status.md")
            .map(|(_, body)| *body)
            .expect("sprout scaffold includes 00-status.md");

        assert!(
            body.contains("## Overview"),
            "00-status.md should summarize what this LEAF is doing"
        );
        assert!(
            body.contains("- request:"),
            "overview should preserve the user's request at status level"
        );
        assert!(
            body.contains("- purpose:"),
            "overview should show why the LEAF exists"
        );
        assert!(
            body.contains("- expected output:"),
            "overview should show what artifact/result this LEAF is aiming at"
        );
        assert!(
            body.contains("- consistency rule:"),
            "overview should remind agents to keep status and gate docs aligned"
        );
    }
}
