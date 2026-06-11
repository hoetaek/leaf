use crate::doctor::{DoctorReport, Location, Severity};
use anyhow::Result;
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn write_text(writer: &mut impl Write, report: &DoctorReport) -> Result<()> {
    let summary = report.summary();

    writeln!(writer, "leaf doctor")?;
    writeln!(writer)?;
    writeln!(
        writer,
        "summary  errors {}  warnings {}  ok {}",
        summary.errors, summary.warnings, summary.ok
    )?;
    writeln!(writer, "result   {}", result_line(report))?;

    write_group(writer, report, Severity::Error, "Errors")?;
    write_group(writer, report, Severity::Warn, "Warnings")?;
    write_group(writer, report, Severity::Ok, "OK checks")?;

    Ok(())
}

pub(crate) fn write_json(writer: &mut impl Write, report: &DoctorReport) -> Result<()> {
    let output = JsonReport::from_report(report);
    serde_json::to_writer_pretty(&mut *writer, &output)?;
    writeln!(writer)?;
    Ok(())
}

fn result_line(report: &DoctorReport) -> &'static str {
    let summary = report.summary();
    if summary.errors > 0 {
        "not ready: fix errors before trusting leaf list"
    } else if summary.warnings > 0 {
        "usable with warnings: leaf list may be degraded"
    } else {
        "ready: leaf list should display cleanly"
    }
}

fn write_group(
    writer: &mut impl Write,
    report: &DoctorReport,
    severity: Severity,
    heading: &str,
) -> Result<()> {
    let findings = report
        .findings
        .iter()
        .filter(|finding| finding.severity == severity)
        .collect::<Vec<_>>();
    if findings.is_empty() {
        return Ok(());
    }

    writeln!(writer)?;
    writeln!(writer, "{heading}")?;
    for finding in findings {
        writeln!(writer)?;
        writeln!(writer, "  {} {}", text_severity(severity), finding.code)?;
        write_location(writer, &finding.location)?;
        writeln!(writer, "    reason  {}", finding.message)?;
        if let Some(impact) = &finding.impact {
            writeln!(writer, "    impact  {impact}")?;
        }
    }

    Ok(())
}

fn write_location(writer: &mut impl Write, location: &Location) -> Result<()> {
    match location {
        Location::None => {}
        Location::Path(path) => {
            writeln!(writer, "    path    {}", display_path(path))?;
        }
        Location::Paths(paths) => {
            if let Some((first, rest)) = paths.split_first() {
                writeln!(writer, "    paths   {}", display_path(first))?;
                for path in rest {
                    writeln!(writer, "            {}", display_path(path))?;
                }
            }
        }
    }
    Ok(())
}

fn text_severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Ok => "OK",
        Severity::Warn => "WARN",
        Severity::Error => "ERROR",
    }
}

fn json_severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Ok => "ok",
        Severity::Warn => "warn",
        Severity::Error => "error",
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[derive(Serialize)]
struct JsonReport {
    leaf_root: String,
    summary: JsonSummary,
    findings: Vec<JsonFinding>,
}

#[derive(Serialize)]
struct JsonSummary {
    ok: usize,
    warnings: usize,
    errors: usize,
}

#[derive(Serialize)]
struct JsonFinding {
    severity: &'static str,
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    paths: Option<Vec<String>>,
}

impl JsonReport {
    fn from_report(report: &DoctorReport) -> Self {
        let summary = report.summary();
        JsonReport {
            leaf_root: display_path(&report.leaf_root),
            summary: JsonSummary {
                ok: summary.ok,
                warnings: summary.warnings,
                errors: summary.errors,
            },
            findings: report
                .findings
                .iter()
                .map(JsonFinding::from_finding)
                .collect(),
        }
    }
}

impl JsonFinding {
    fn from_finding(finding: &crate::doctor::DoctorFinding) -> Self {
        let (path, paths) = match &finding.location {
            Location::None => (None, None),
            Location::Path(path) => (Some(display_path(path)), None),
            Location::Paths(paths) => (
                None,
                Some(
                    paths
                        .iter()
                        .map(PathBuf::as_path)
                        .map(display_path)
                        .collect(),
                ),
            ),
        };

        JsonFinding {
            severity: json_severity(finding.severity),
            code: finding.code,
            message: finding.message.clone(),
            path,
            paths,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::DoctorFinding;

    #[test]
    fn text_groups_errors_warnings_and_ok_findings() {
        let report = DoctorReport::new(
            ".leaf",
            vec![
                DoctorFinding::ok("leaf_root_present", ".leaf initialized").with_path(".leaf"),
                DoctorFinding::warn("duplicate_slug", "slug appears in more than one stage")
                    .with_paths([".leaf/01-sprouts/demo", ".leaf/02-leaves/demo"]),
                DoctorFinding::error(
                    "stage_dir_mismatch",
                    "stage sprout conflicts with directory leaves; expected leaf",
                )
                .with_path(".leaf/02-leaves/demo/00-status.md"),
            ],
        );

        let mut output = Vec::new();
        write_text(&mut output, &report).expect("text output");
        let output = String::from_utf8(output).expect("utf8 output");

        assert!(output.contains("summary  errors 1  warnings 1  ok 1"));
        assert!(output.contains("result   not ready: fix errors before trusting leaf list"));
        assert!(output.find("Errors").unwrap() < output.find("Warnings").unwrap());
        assert!(output.find("Warnings").unwrap() < output.find("OK checks").unwrap());
        assert!(
            output.contains("    paths   .leaf/01-sprouts/demo\n            .leaf/02-leaves/demo")
        );
    }

    #[test]
    fn json_uses_flat_findings_with_mutually_exclusive_path_fields() {
        let report = DoctorReport::new(
            ".leaf",
            vec![
                DoctorFinding::warn("duplicate_slug", "slug appears in more than one stage")
                    .with_paths([".leaf/01-sprouts/demo", ".leaf/02-leaves/demo"]),
            ],
        );

        let mut output = Vec::new();
        write_json(&mut output, &report).expect("json output");
        let json: serde_json::Value = serde_json::from_slice(&output).expect("valid json");

        assert_eq!(json["leaf_root"], ".leaf");
        assert_eq!(json["summary"]["warnings"], 1);
        assert_eq!(json["findings"][0]["severity"], "warn");
        assert_eq!(json["findings"][0]["code"], "duplicate_slug");
        assert!(json["findings"][0].get("path").is_none());
        assert_eq!(
            json["findings"][0]["paths"],
            serde_json::json!([".leaf/01-sprouts/demo", ".leaf/02-leaves/demo"])
        );
    }
}
