/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ██████╗░░░░░█████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ╚════██╗░░░██╔══██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ░░███╔═╝░░░██║░░██║░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ██╔══╝░░░░░██║░░██║░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗╚█████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝░╚════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk

*/

use super::record::RunRecord;

pub struct Recomputed {
    pub correct: usize,
    pub total: usize,
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
}

pub fn recompute(target: &str, typed: &str, elapsed_ms: u64) -> Recomputed {
    let tc: Vec<char> = target.chars().collect();
    let mut correct = 0usize;
    let mut total = 0usize;
    for (i, ch) in typed.chars().enumerate() {
        total += 1;
        if tc.get(i) == Some(&ch) {
            correct += 1;
        }
    }
    let minutes = (elapsed_ms as f64 / 60_000.0).max(1e-9);
    let wpm = (correct as f64 / 5.0) / minutes;
    let raw_wpm = (total as f64 / 5.0 ) / minutes;
    let accuracy = if total == 0 { 100.0 } else { correct as f64 / total as f64 * 100.0 };
    Recomputed { correct, total, wpm, raw_wpm, accuracy }
}

#[derive(Debug)]
pub enum AuditResult {
    Plausible,
    Rejected(Vec<String>),
}

pub fn audit(r: &RunRecord) -> AuditResult {
    fn q6(x: f64) -> f64 {
        (x * 1_000_000.0).round() / 1_000_000.0
    }

    if r.kind == "steno" {
        return audit_steno(r);
    }

    let mut issues = Vec::new();

    let re = recompute(&r.target, &r.typed, r.elapsed_ms);
    if (q6(re.wpm) - r.wpm).abs() > 0.5 {
        issues.push(format!("wpm: claimed {:.2}, recomputed {:.2}", r.wpm, re.wpm));
    }
    if (re.raw_wpm - r.raw_wpm).abs() > 0.5 {
        issues.push(format!("raw_wpm: claimed {:.2}, recomputed {:.2}", r.raw_wpm, re.raw_wpm));
    }
    if (re.accuracy - r.accuracy).abs() > 0.5 {
        issues.push(format!("accuracy: claimed {:.2}, recomputed {:.2}", r.accuracy, re.accuracy));
    }

    if r.elapsed_ms == 0 && r.wpm > 0.0 {
        issues.push("zero elapsed time with nonzero wpm".into());
    }
    if r.wpm > 300.0 {
        issues.push(format!("wpm {:.1} exceeeds human plausible limit (~300)", r.wpm));        
    }
    if r.typed.chars().count() as u64 > r.elapsed_ms / 20 {
        issues.push("typed more than 50 characters per second".into());
    }

    if r.mode.starts_with("time ") {
        let secs: u64 = r.mode.trim_start_matches("time ").trim().parse().unwrap_or(0);
        if secs > 0 {
            let expect = secs * 1000;
            let drift = (r.elapsed_ms as i64 - expect as i64).abs();
            if drift > 2000 {
                issues.push(format!("time mode: elapsed {}ms vs expected {}ms", r.elapsed_ms, expect));
            }
        }
    }

    if !r.wpm_samples.is_empty() {
        let n = r.wpm_samples.len();
        let mean = r.wpm_samples.iter().sum::<f64>() / n as f64;
        if (mean - r.wpm).abs() > r.wpm * 0.5 + 10.0 {
            issues.push(format!("wpm_samples mean {:.1} far from wpm {:.1}", mean, r.wpm));
        }
        let flat = r.wpm_samples.iter().all(|s| (s -r.wpm_samples[0]).abs() < 0.001);
        if flat && n > 3 {
            issues.push("wpm_samples are perfectly flat (likely synthetic)".into());
        }
    }

    if issues.is_empty() { AuditResult::Plausible } else { AuditResult::Rejected(issues) }
}

fn audit_steno(r: &RunRecord) -> AuditResult {
    fn q6(x: f64) -> f64 {
        (x * 1_000_000.0).round() / 1_000_000.0
    }

    let mut issues = Vec::new();

    if r.strokes.is_empty() {
        return AuditResult::Rejected(vec!["Steno record has no strokes".into()]);
    }

    let correct = r.strokes.iter().filter(|s| s.correct).count();
    let total = r.strokes.len();

    let recomputed_acc = correct as f64 / total as f64 * 100.0;
    if (q6(recomputed_acc) - r.accuracy).abs() > 0.5 {
        issues.push(format!(
            "accuracy: claimed {:.2}, recomputed {:.2}",
            r.accuracy, recomputed_acc
        ));
    }

    let minutes = (r.elapsed_ms as f64 / 60_000.0).max(1e-9);
    let recomputed_wpm = (correct as f64 / 5.0) / minutes;
    if (q6(recomputed_wpm) - r.wpm).abs() > 0.5 {
        issues.push(format!(
            "wpm: claimed {:.2}, recomputed {:.2}",
            r.wpm, recomputed_wpm
        ));
    }

    let mut cum_ms = 0u64;
    for (i, s) in r.strokes.iter().enumerate() {
        cum_ms += s.elapsed_ms;
        let expected_norm: String = s
            .expected
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_ascii_lowercase())
            .collect();
        let actual_norm: String = s
            .actual
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_ascii_lowercase())
            .collect();
        let really_correct = expected_norm == actual_norm;
        if really_correct != s.correct {
            issues.push(format!(
                "stroke {}: correct flag {} but normalized match {}",
                i, s.correct, really_correct
            ));
        }
    }

    if cum_ms > r.elapsed_ms + 2000 {
        issues.push(format!(
            "strokes sum to {}ms but elapsed is {}ms",
            cum_ms, r.elapsed_ms
        ));
    }

    if r.elapsed_ms == 0 && r.wpm > 0.0 {
        issues.push("zero elapsed time with nonzero wpm".into());
    }
    if r.wpm > 400.0 {
        issues.push(format!("steno wpm {:.1} exceeds plausible limit (~400)", r.wpm));
    }

    if issues.is_empty() {
        AuditResult::Plausible
    } else {
        AuditResult::Rejected(issues)
    }
}
