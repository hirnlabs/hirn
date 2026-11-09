//! Document linting through `hyalo-mdlint`.
//!
//! Tier 1 owns the Markdown dialect, so it also owns the checks that keep the
//! dialect honest. Four passes, matching what `hyalo lint` runs:
//!
//! 1. **Schema**: frontmatter against the project's `schema.toml` definitions
//!    (`hyalo_mdlint::schema::lint_file`).
//! 2. **Body**: the mdbook-lint ruleset plus the HYALO native body rules
//!    (`HYALO001` bare `[]` checkboxes, `HYALO002` completed status with
//!    unticked tasks) via `HyaloLintEngine::lint_body`.
//! 3. **Frontmatter HYALO rules**: `HYALO003` date format, `HYALO004` datetime
//!    format, `HYALO007` title is a scalar, via their dedicated engine passes.
//!
//! Findings are reported, never enforced: writing a document that violates its
//! schema is allowed, and the violations come back with the write.

use std::path::Path;

use hyalo_core::schema::{PropertyConstraint, SchemaConfig};
use hyalo_mdlint::config::LintConfig;
use hyalo_mdlint::engine::HyaloLintEngine;
use hyalo_mdlint::schema::Severity;
use hyalo_mdlint::{DiagSeverity, Diagnostic};
use indexmap::IndexMap;

use crate::error::{Error, Result};
use crate::types::DocPath;

/// A lint finding on one document.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LintFinding {
    /// Rule id, e.g. `HYALO001`, `MD013`, or a `schema/*` kind.
    pub rule: String,
    /// Whether the finding is an error or a warning.
    pub severity: LintSeverity,
    /// 1-based line in the file; `1` for findings about the document as a whole.
    pub line: usize,
    /// Human-readable message.
    pub message: String,
}

/// Severity, normalised across the schema and body rule engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LintSeverity {
    Error,
    Warn,
}

/// Lint a Markdown file on disk.
///
/// `root` is the project root: the schema pass needs the real file, because it
/// re-reads the frontmatter with its own parser. A missing file yields no
/// findings rather than an error.
pub fn lint_file(
    root: &Path,
    doc: &DocPath,
    schema: Option<&SchemaConfig>,
) -> Result<Vec<LintFinding>> {
    let path = doc.to_fs_path(root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let source = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
    lint_source(doc, &path, &source, schema)
}

/// Lint one Markdown source.
pub fn lint_source(
    doc: &DocPath,
    path: &Path,
    source: &str,
    schema: Option<&SchemaConfig>,
) -> Result<Vec<LintFinding>> {
    let mut findings = Vec::new();

    if let Some(schema) = schema {
        let result = hyalo_mdlint::schema::lint_file(path, doc.as_str(), schema, false)
            .map_err(|e| Error::Parse(format!("schema lint failed: {e}")))?;
        findings.extend(result.violations.into_iter().map(|violation| {
            LintFinding {
                rule: violation
                    .kind
                    .unwrap_or("schema")
                    .trim_start_matches("schema/")
                    .to_string(),
                severity: match violation.severity {
                    Severity::Error => LintSeverity::Error,
                    Severity::Warn => LintSeverity::Warn,
                },
                line: 1,
                message: violation.message,
            }
        }));
    }

    let engine = HyaloLintEngine::create()
        .map_err(|e| Error::Parse(format!("lint engine unavailable: {e}")))?;
    let config = LintConfig::default();
    let filter: Vec<String> = Vec::new();

    let frame = hyalo_core::frontmatter::DocumentFrame::parse(source)
        .map_err(|e| Error::Parse(e.to_string()))?;
    let body = frame.body(source.as_bytes()).unwrap_or_default();
    let body = String::from_utf8_lossy(body);

    let properties: IndexMap<String, serde_json::Value> = frame
        .yaml(source.as_bytes())
        .and_then(|yaml| std::str::from_utf8(yaml).ok())
        .and_then(|yaml| hyalo_core::frontmatter::parse_yaml_map(yaml).ok())
        .unwrap_or_default();

    let status = properties.get("status").and_then(serde_json::Value::as_str);

    let diagnostics = engine
        .lint_body(
            &body,
            doc.as_str(),
            status,
            schema.is_some_and(|schema| {
                schema
                    .types
                    .values()
                    .any(|t| t.properties.contains_key("status"))
            }),
            &config,
            &filter,
        )
        .map_err(|e| Error::Parse(format!("body lint failed: {e}")))?;
    push_diagnostics(&mut findings, diagnostics);

    // HYALO003 / HYALO007 inspect the frontmatter map directly.
    push_diagnostics(
        &mut findings,
        engine.lint_frontmatter_hyalo003(doc.as_str(), &properties, &config, &filter, false),
    );
    push_diagnostics(
        &mut findings,
        engine.lint_frontmatter_hyalo007(doc.as_str(), &properties, &config, &filter, false),
    );

    // HYALO004 only validates schema-declared datetime properties.
    let datetime_pairs = datetime_pairs(schema, &properties);
    push_diagnostics(
        &mut findings,
        engine.lint_frontmatter_hyalo004(doc.as_str(), &datetime_pairs, &config, &filter, false),
    );

    Ok(findings)
}

fn push_diagnostics(findings: &mut Vec<LintFinding>, diagnostics: Vec<Diagnostic>) {
    findings.extend(diagnostics.into_iter().map(|diagnostic| LintFinding {
        rule: diagnostic.rule_id,
        severity: match diagnostic.severity {
            DiagSeverity::Error => LintSeverity::Error,
            DiagSeverity::Warn => LintSeverity::Warn,
        },
        line: diagnostic.line,
        message: diagnostic.message,
    }));
}

/// The `(name, value, is_tz)` triples HYALO004 expects: only schema-declared
/// datetime properties with a string value.
fn datetime_pairs<'a>(
    schema: Option<&SchemaConfig>,
    properties: &'a IndexMap<String, serde_json::Value>,
) -> Vec<(&'a str, &'a str, bool)> {
    let Some(schema) = schema else {
        return Vec::new();
    };
    let mut keys: Vec<(&str, bool)> = Vec::new();
    for type_schema in schema.types.values() {
        for (key, constraint) in &type_schema.properties {
            match constraint {
                PropertyConstraint::DateTime => keys.push((key.as_str(), false)),
                PropertyConstraint::DateTimeTz => keys.push((key.as_str(), true)),
                _ => {}
            }
        }
    }
    keys.sort_unstable();
    keys.dedup();
    keys.into_iter()
        .filter_map(|(key, is_tz)| {
            properties
                .get_key_value(key)
                .and_then(|(stored_key, value)| {
                    value
                        .as_str()
                        .map(|value| (stored_key.as_str(), value, is_tz))
                })
        })
        .collect()
}
