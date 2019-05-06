use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_writer};
use std::io::{BufRead, Write};
use unique::Id;

use crate::formula::Formula;
use crate::graph::flatten;
use crate::options::OPTIONS;
use crate::score::Score;
use crate::system::os_error;

#[derive(Serialize)]
struct OutRecord {
    id: usize,
    nodes: Vec<u8>,
    edges: Vec<(usize, usize)>,
}

#[derive(Deserialize)]
struct InRecord {
    id: usize,
    score: f32,
}

pub fn enqueue_heuristic(f: &Id<Formula>) {
    if let Some(mut file) = OPTIONS.heuristic_file_in.as_ref() {
        let (nodes, edges) = flatten(f.into());
        let id = unsafe { Id::into_raw(f.clone()) } as usize;
        let record = OutRecord { id, nodes, edges };
        to_writer(file, &record).unwrap_or_else(|e| {
            log::error!("failed to write data to heuristic pipe: {}", e);
            os_error();
        });
        writeln!(file).unwrap_or_else(|e| {
            log::error!("failed to write newline to heuristic pipe: {}", e);
            os_error();
        });
    }
}

pub fn deque_heuristic() -> (Id<Formula>, Score) {
    if let Some(mtx) = OPTIONS.heuristic_file_out.as_ref() {
        let mut file = mtx.lock().unwrap();
        let mut line = String::new();
        file.read_line(&mut line).unwrap_or_else(|e| {
            log::error!("failed to read data from heuristic pipe: {}", e);
            os_error();
        });
        let record: InRecord = from_str(&line).unwrap_or_else(|e| {
            log::error!("failed to read data from heuristic pipe: {}", e);
            os_error();
        });

        let f = unsafe { Id::from_id(record.id) };
        let score = record.score.into();
        (f, score)
    }
    else {
        unreachable!()
    }
}
