/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ██████╗░░░░░█████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ╚════██╗░░░██╔══██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ░░███╔═╝░░░██║░░██║░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ██╔══╝░░░░░██║░░██║░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗╚█████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝░╚════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk

*/

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use ed25519_dalek::VerifyingKey;

use crate::error::Result;
use super::audit::{audit, AuditResult, recompute};
use super::record::RunRecord;

pub fn export_records(chain: &super::chain::PbChain, mode: Option<&str>) -> Result<Vec<String>> {
    let file = fs::File::open(chain.path())?;
    let mut out = Vec::new();
    for line in BufReader::new(file).lines().flatten() {
        let r: RunRecord = serde_json::from_str(&line)?;
        if mode.map_or(true, |m| r.mode == m) {
            out.push(line);
        }
    }
    Ok(out)
}

#[derive(Debug, Default)]
pub struct VerifyReport {
    pub ok: bool,
    pub total: usize,
    pub plausible: usize,
    pub rejected: usize,
    pub lines: Vec<String>,
    pub personal_bests: BTreeMap<String, f64>,
}

impl VerifyReport {
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("records: {} plausible: {} rejected: {}\n", self.total, self.plausible, self.rejected));
        for l in &self.lines { s.push_str(l); s.push('\n'); }
            
        if !self.personal_bests.is_empty() {
            s.push_str("PB By Mode:\n");
            for (mode, wpm) in &self.personal_bests {
                s.push_str(&format!("  {:<14} {:.1}\n", mode, wpm));
            }
        }

        s.push_str(if self.ok { "RESULT: PASS" } else { "RESULT: FAIL" });
        s
    }
}

pub fn verify_file(path: &Path, verifying: &VerifyingKey) -> Result<VerifyReport> {
    let file = fs::File::open(path)?;
    let mut report = VerifyReport { ok: true, ..Default::default() };
    let mut expected_prev = super::record::GENESIS_HASH.to_string();

    for (i, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        report.total += 1;
        let r: RunRecord = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                report.lines.push(format!("line {}: parse error: {}", i + 1, e));
                report.ok = false;
                return Ok(report);
            }
        };

        if r.schema != super::record::SCHEMA {
            report.lines.push(format!("line {}: unsupported schema {}", i + 1, r.schema));
            report.ok = false;
            return Ok(report);
        }

        if r.prev_hash != expected_prev {
            report.lines.push(format!("line {}: chain broken", i + 1));
            report.ok = false;
            return Ok(report);
        }
        let body = super::record::canonical_body(&r);
        if !super::identity::Identity::verify(verifying, &body, &r.sig) {
            report.lines.push(format!("line {}: bad signature", i + 1));
            report.ok = false;
            return Ok(report);
        }
        expected_prev = super::record::body_hash(&r);

        let is_steno = r.kind == "steno";

        match audit(&r) {
            AuditResult::Plausible => {
                report.plausible += 1;

                if is_steno {
                    let correct = r.strokes.iter().filter(|s| s.correct).count();
                    let total_strokes = r.strokes.len();
                    report.lines.push(format!(
                        "  [{}] {} {:.1} wpm {:.1}% acc ({}/{} strokes) plausible",
                        i + 1, r.mode, r.wpm, r.accuracy, correct, total_strokes,
                    ));
                } else {
                    let re = recompute(&r.target, &r.typed, r.elapsed_ms);
                    report.lines.push(format!(
                        "  [{}] {} {:.1} wpm (recomputed {:.1}) {:.1}% acc ({}/{} chars) plausible",
                        i + 1, r.mode, r.wpm, re.wpm, re.accuracy, re.correct, re.total,
                    ));
                }

                let entry = report
                    .personal_bests
                    .entry(r.mode.clone())
                    .or_insert(0.0);
                if r.wpm > *entry {
                    *entry = r.wpm;
                }
            }
            AuditResult::Rejected(errs) => {
                report.rejected += 1;
                report.ok = false;
                report.lines.push(format!("   [{}] {} REJECTED: {}", i + 1, r.mode, errs.join("; ")));
            }
        }
    }

    Ok(report)
}
