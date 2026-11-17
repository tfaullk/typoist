/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ██████╗░░░░░█████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ╚════██╗░░░██╔══██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ░░███╔═╝░░░██║░░██║░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ██╔══╝░░░░░██║░░██║░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗╚█████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝░╚════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk

*/

use serde::{Deserialize, Serialize};


pub const SCHEMA: u8 = 2;
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StenoStrokeRecord {
    pub expected: String,
    pub actual: String,
    pub correct: bool,
    pub elapsed_ms: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RunRecord {
    pub schema: u8,
    pub version: String,
    pub mode: String,
    pub word_set: String,
    pub target: String,
    pub typed: String,
    pub elapsed_ms: u64,
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub wpm_samples: Vec<f64>,
    pub started_at_unix: u64,
    pub nonce: String,
    pub prev_hash: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub sig: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub strokes: Vec<StenoStrokeRecord>,
}

pub fn canonical_body(r: &RunRecord) -> Vec<u8> {
    fn q6(x: f64) -> f64 {
        (x * 1_000_000.0).round() / 1_000_000.0
    }

    let samples_q: Vec<u64> = r.wpm_samples.iter().map(|&s| q6(s).to_bits()).collect();

    let strokes_view: Vec<StrokeView<'_>> = r
        .strokes
        .iter()
        .map(|s| StrokeView {
            expected: &s.expected,
            actual: &s.actual,
            correct: s.correct,
            elapsed_ms: s.elapsed_ms,
        })
        .collect();

    serde_json::to_vec(&BodyView {
        schema: r.schema,
        version: &r.version,
        mode: &r.mode,
        word_set: &r.word_set,
        target: &r.target,
        typed: &r.typed,
        elapsed_ms: r.elapsed_ms,
        wpm_bits: q6(r.wpm).to_bits(),
        raw_wpm_bits: q6(r.raw_wpm).to_bits(),
        accuracy_bits: q6(r.accuracy).to_bits(),
        consistency_bits: q6(r.consistency).to_bits(),
        wpm_sample_bits: samples_q,
        started_at_unix: r.started_at_unix,
        nonce: &r.nonce,
        prev_hash: &r.prev_hash,
        kind: &r.kind,
        strokes: strokes_view,
    })
    .expect("record serialization cannot fail")
}

#[derive(Serialize)]
struct BodyView<'a> {
    schema: u8,
    version: &'a str,
    mode: &'a str,
    word_set: &'a str,
    target: &'a str,
    typed: &'a str,
    elapsed_ms: u64,
    wpm_bits: u64,
    raw_wpm_bits: u64,
    accuracy_bits: u64,
    consistency_bits: u64,
    wpm_sample_bits: Vec<u64>,  
    started_at_unix: u64,
    nonce: &'a str,
    prev_hash: &'a str,
    kind: &'a str,
    strokes: Vec<StrokeView<'a>>,
}

#[derive(Serialize)]
struct StrokeView<'a> {
    expected: &'a str,
    actual: &'a str,
    correct: bool,
    elapsed_ms: u64,
}

pub fn body_hash(r: &RunRecord) -> String {
    blake3::hash(&canonical_body(r)).to_hex().to_string()
}
