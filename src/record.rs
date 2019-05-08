use lazy_static::lazy_static;
use serde::Serialize;
use serde_json::to_writer;
use std::fs::{File, OpenOptions};
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

lazy_static! {
    static ref RECORD_FILE: File = {
        if let Some(path) = OPTIONS.record_file.as_ref() {
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
                .unwrap_or_else(|e| {
                    log::error!("failed to open record file: {}", e);
                    os_error()
                })
        } else {
            unreachable!()
        }
    };
}

pub fn record(f: &Id<Formula>, status: Status) {
    if OPTIONS.record_file.is_some() {
        let (nodes, edges) = flatten(f.into());
        let y = status as u8;
        let record = Record { nodes, edges, y };
        to_writer(&*RECORD_FILE, &record).unwrap_or_else(|e| {
            log::error!("failed to write data to record file: {}", e);
            os_error();
        });
        writeln!(&*RECORD_FILE).unwrap_or_else(|e| {
            log::error!("failed to write newline to record file: {}", e);
            os_error();
        });
    }
}

pub fn initialize() {
    if OPTIONS.record_file.is_some() {
        lazy_static::initialize(&RECORD_FILE);
    }
}
