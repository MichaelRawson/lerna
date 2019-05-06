use serde::Serialize;
use serde_json::to_writer;
use std::io::Write;
use unique::Id;

use crate::formula::Formula;
use crate::graph::flatten;
use crate::options::OPTIONS;
use crate::status::Status;
use crate::system::os_error;

#[derive(Serialize)]
struct Record {
    nodes: Vec<u8>,
    edges: Vec<(usize, usize)>,
    y: u8,
}

pub fn record(f: &Id<Formula>, status: Status) {
    if let Some(mut file) = OPTIONS.record_file.as_ref() {
        let (nodes, edges) = flatten(f.into());
        let y = status as u8;
        let record = Record { nodes, edges, y };
        to_writer(file, &record).unwrap_or_else(|e| {
            log::error!("failed to write data to record file: {}", e);
            os_error();
        });
        writeln!(file).unwrap_or_else(|e| {
            log::error!("failed to write newline to record file: {}", e);
            os_error();
        });
    }
}
