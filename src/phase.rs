//! LEAF phase model and the polish-boundary 멈칫.
//!
//! Phase transitions used to be implicit: an agent just started writing the
//! next gate file, so the boundary where `leaf:polish` should run was invisible
//! to the tool. `leaf next` makes the transition a real machine event. Before it
//! advances, it checks whether the phase being left still carries the
//! machine-only "polish pending" token; if so it pauses (멈칫) and tells the
//! agent to polish, rather than forcing it. `leaf doctor` reports the same
//! condition for any already-passed phase, so a skipped polish is visible
//! instead of silent.

use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::ExitCode;

/// Machine-only marker: a gate file carrying this line (at the start of a line)
/// marks its phase as not-yet-polished. `leaf:polish` removes it. It is a
/// deliberately machine-shaped token so prose that *discusses* the mechanism
/// (this design doc, references) does not match — detection is line-anchored.
pub(crate) const POLISH_PENDING_TOKEN: &str = "<!-- leaf:polish-pending -->";

/// Block scaffolded into each phase's first gate file: the machine token plus a
/// human-readable reminder. Detection keys only on the token line.
pub(crate) const POLISH_PENDING_BLOCK: &str = "<!-- leaf:polish-pending -->\n\
     <!-- 이 phase를 마치면 leaf:polish로 누적 전체를 다듬고 위 토큰 줄을 지운 뒤 `leaf next` 하세요. -->\n\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    Learn,
    Example,
    Architect,
    Feedback,
}

impl Phase {
    pub(crate) const ORDER: [Phase; 4] = [
        Phase::Learn,
        Phase::Example,
        Phase::Architect,
        Phase::Feedback,
    ];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Phase::Learn => "Learn",
            Phase::Example => "Example",
            Phase::Architect => "Architect",
            Phase::Feedback => "Feedback",
        }
    }

    pub(crate) fn dir(self) -> &'static str {
        match self {
            Phase::Learn => "01-Learn",
            Phase::Example => "02-Example",
            Phase::Architect => "03-Architect",
            Phase::Feedback => "04-Feedback",
        }
    }

    /// First gate label, written into `current gate` when entering this phase.
    pub(crate) fn first_gate_label(self) -> &'static str {
        match self {
            Phase::Learn => "① Intent",
            Phase::Example => "③ Criteria",
            Phase::Architect => "⑤ Design",
            Phase::Feedback => "⑨ Review",
        }
    }

    pub(crate) fn index(self) -> usize {
        Phase::ORDER
            .iter()
            .position(|&p| p == self)
            .expect("phase is in ORDER")
    }

    pub(crate) fn next(self) -> Option<Phase> {
        Phase::ORDER.get(self.index() + 1).copied()
    }

    /// Match a `current phase` status value to a phase by its leading word, so
    /// annotated values like `Architect (⑤ Design)` still resolve.
    pub(crate) fn from_status_value(value: &str) -> Option<Phase> {
        let trimmed = value.trim();
        Phase::ORDER
            .into_iter()
            .find(|phase| trimmed.starts_with(phase.name()))
    }
}

/// True if any gate file in `phase_dir` still carries [`POLISH_PENDING_TOKEN`]
/// at the start of a line. Checkpoint copies (`YYMMDD-HHMM <file>`) are skipped
/// so a preserved snapshot of an old draft does not keep a polished phase dirty.
/// A missing directory reads as polished (false): an absent phase has nothing
/// to polish.
pub(crate) fn phase_unpolished(phase_dir: &Path) -> bool {
    let Ok(read_dir) = std::fs::read_dir(phase_dir) else {
        return false;
    };
    for entry in read_dir.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".md") || is_checkpoint_copy(&name) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(entry.path())
            && content
                .lines()
                .any(|line| line.trim_start().starts_with(POLISH_PENDING_TOKEN))
        {
            return true;
        }
    }
    false
}

/// Checkpoint copies are named `YYMMDD-HHMM <original>` (see `checkpoint.rs`).
fn is_checkpoint_copy(name: &str) -> bool {
    let Some((stamp, _rest)) = name.split_once(' ') else {
        return false;
    };
    let Some((date, time)) = stamp.split_once('-') else {
        return false;
    };
    date.len() == 6
        && date.bytes().all(|b| b.is_ascii_digit())
        && time.len() == 4
        && time.bytes().all(|b| b.is_ascii_digit())
}

/// Rewrite the `current phase` / `current gate` preamble lines of a status
/// document to `phase`. Only the preamble (before the first `##` heading) is
/// touched, matching the status parser. Other lines are preserved verbatim.
pub(crate) fn rewrite_status_phase(content: &str, phase: Phase) -> String {
    let mut out = Vec::new();
    let mut in_preamble = true;
    for line in content.lines() {
        if in_preamble && line.trim_start().starts_with("##") {
            in_preamble = false;
        }
        if in_preamble && is_status_field(line, "current phase") {
            out.push(format!("- current phase: {}", phase.name()));
        } else if in_preamble && is_status_field(line, "current gate") {
            out.push(format!("- current gate: {}", phase.first_gate_label()));
        } else {
            out.push(line.to_string());
        }
    }
    let mut result = out.join("\n");
    if content.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Match a `- key:` status field line, case-insensitively.
fn is_status_field(line: &str, key: &str) -> bool {
    let trimmed = line.trim_start();
    let trimmed = trimmed.strip_prefix('-').map_or(trimmed, str::trim_start);
    trimmed.to_ascii_lowercase().starts_with(&format!("{key}:"))
}

/// `leaf next <slug>`: advance one phase, pausing if the current phase is
/// unpolished. Reads the current phase from `00-status.md`, and either prints a
/// 멈칫 (and changes nothing) or scaffolds the next phase and rewrites the
/// status preamble.
pub(crate) fn run_next(root_path: &Path) -> Result<ExitCode> {
    let status_path = root_path.join("00-status.md");
    let content = std::fs::read_to_string(&status_path)
        .with_context(|| format!("failed to read {}", status_path.display()))?;

    let current_value =
        crate::inventory::parse_status_summary(&content, crate::inventory::StageDir::Sprouts)
            .current_phase;

    let Some(current) = current_value.as_deref().and_then(Phase::from_status_value) else {
        bail!("cannot determine current phase from 00-status.md `current phase` field");
    };

    let Some(next) = current.next() else {
        println!(
            "이미 마지막 phase(Feedback)입니다. close-out(⑩ 후 keep/press/fall)은 using-leaf를 따르세요."
        );
        return Ok(ExitCode::SUCCESS);
    };

    if phase_unpolished(&root_path.join(current.dir())) {
        print_pause(current);
        return Ok(ExitCode::SUCCESS);
    }

    crate::scaffold::scaffold_phase(root_path, next)?;
    let updated = rewrite_status_phase(&content, next);
    std::fs::write(&status_path, updated)
        .with_context(|| format!("failed to write {}", status_path.display()))?;

    println!("✓ {} polished — {} 진입.", current.name(), next.name());
    println!(
        "  생성: .leaf/.../{}/  ·  current phase → {}",
        next.dir(),
        next.name()
    );
    Ok(ExitCode::SUCCESS)
}

fn print_pause(current: Phase) {
    println!(
        "✋ 멈칫 — {0}을(를) polish하지 않고 다음 phase로 넘어가려 합니다.",
        current.name()
    );
    println!();
    println!(
        "  {0}({1})에 미polish 마커가 남아있습니다.",
        current.name(),
        current.dir()
    );
    println!(
        "  이 경계는 {}까지의 누적 문서를 '하나의 이어지는 보고서'로 읽고 다듬는 자리입니다.",
        current.name()
    );
    println!();
    println!(
        "    → leaf:polish 를 {} 누적에 돌려 마커를 지운 뒤 다시 `leaf next` 하세요.",
        current.name()
    );
    println!();
    println!("(다음 phase는 아직 생성하지 않았습니다. 전진은 보류됐습니다.)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn token_at_line_start_marks_phase_unpolished() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("01-intent.md"),
            format!("{POLISH_PENDING_TOKEN}\n# Intent\n"),
        )
        .expect("write");
        assert!(phase_unpolished(dir.path()));
    }

    #[test]
    fn token_quoted_in_prose_does_not_mark_unpolished() {
        // ⑥ critic's regression: a document that *discusses* the token (in
        // backticks or mid-sentence) must not be read as an unpolished marker.
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("05-design.md"),
            format!(
                "# Design\n\n마커는 `{POLISH_PENDING_TOKEN}` 이고 줄머리에서만 매치된다.\n\
                 - 설명: {POLISH_PENDING_TOKEN} 를 polish가 제거한다.\n"
            ),
        )
        .expect("write");
        assert!(
            !phase_unpolished(dir.path()),
            "prose mentions of the token must not count as a marker"
        );
    }

    #[test]
    fn absent_token_reads_polished() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("01-intent.md"), "# Intent\n").expect("write");
        assert!(!phase_unpolished(dir.path()));
    }

    #[test]
    fn checkpoint_copy_with_token_is_ignored() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("260622-1257 01-intent.md"),
            format!("{POLISH_PENDING_TOKEN}\n"),
        )
        .expect("write");
        assert!(
            !phase_unpolished(dir.path()),
            "a preserved snapshot must not keep a polished phase dirty"
        );
    }

    #[test]
    fn missing_phase_dir_reads_polished() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(!phase_unpolished(&dir.path().join("absent")));
    }

    #[test]
    fn from_status_value_matches_annotated_phase() {
        assert_eq!(
            Phase::from_status_value("Architect (⑤ Design)"),
            Some(Phase::Architect)
        );
        assert_eq!(
            Phase::from_status_value("  Learn (휴지 — triple 잠금)"),
            Some(Phase::Learn)
        );
        assert_eq!(Phase::from_status_value("Feedback"), Some(Phase::Feedback));
        assert_eq!(Phase::from_status_value("nonsense"), None);
    }

    #[test]
    fn phase_order_and_next_are_consistent() {
        assert_eq!(Phase::Learn.next(), Some(Phase::Example));
        assert_eq!(Phase::Architect.next(), Some(Phase::Feedback));
        assert_eq!(Phase::Feedback.next(), None);
        assert!(Phase::Learn.index() < Phase::Architect.index());
    }

    #[test]
    fn rewrite_status_replaces_only_preamble_phase_and_gate() {
        let status = "# Sprout Status\n\n\
             - why: x\n\
             - current phase: Learn\n\
             - current gate: ① Intent\n\
             - next action: go\n\n\
             ## Overview\n\
             - current phase: must-not-touch\n";
        let out = rewrite_status_phase(status, Phase::Example);
        assert!(out.contains("- current phase: Example"));
        assert!(out.contains("- current gate: ③ Criteria"));
        assert!(out.contains("- why: x"));
        assert!(out.contains("- next action: go"));
        // The preamble ends at the first `##`; later lines are preserved.
        assert!(out.contains("- current phase: must-not-touch"));
        assert!(out.ends_with('\n'));
    }
}
