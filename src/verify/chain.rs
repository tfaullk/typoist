/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ██████╗░░░░░█████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ╚════██╗░░░██╔══██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ░░███╔═╝░░░██║░░██║░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ██╔══╝░░░░░██║░░██║░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗╚█████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝░╚════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk

*/

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::error::Result;
use super::identity::Identity;
use super::record::{body_hash, canonical_body, RunRecord, GENESIS_HASH};

pub struct PbChain {
    path: PathBuf,
}

impl PbChain {
    pub fn open() -> Self {
        let path = crate::config::paths::config_dir().join("pbs.jsonl");
        Self { path }
    }

    pub fn path(&self) -> &PathBuf { &self.path }

    pub fn last_hash(&self) -> String {
        let Ok(file) = fs::File::open(&self.path) else { return GENESIS_HASH.into() };
        let mut last = GENESIS_HASH.to_string();
        for line in BufReader::new(file).lines().flatten() {
            if let Ok(r) = serde_json::from_str::<RunRecord>(&line) {
                last = body_hash(&r);
            }
        }
        last
    }
    
    pub fn append(&self, mut record: RunRecord, id: &Identity) -> Result<RunRecord> {
        record.prev_hash = self.last_hash();
        let body = canonical_body(&record);
        record.sig = id.sign(&body);
        let mut f = OpenOptions::new().create(true).append(true).open(&self.path)?;
        let line = serde_json::to_string(&record)?;
        writeln!(f, "{}", line)?;
        Ok(record)
    }
}
