use bufstream::BufStream;
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::collections::HashMap;
use std::io::ErrorKind::WouldBlock;
use std::io::{BufRead, Write};
use std::net::TcpStream;
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

lazy_static! {
    static ref IN_FLIGHT: RwLock<HashMap<usize, Id<Formula>>> =
        RwLock::new(HashMap::new());
    static ref SOCKET: Mutex<BufStream<TcpStream>> = {
        let socket = TcpStream::connect(&OPTIONS.heuristic_address)
            .unwrap_or_else(|e| {
                log::error!("could not connect to heuristic: {}", e);
                os_error()
            });
        socket.set_nonblocking(true).unwrap_or_else(|e| {
            log::error!("setting non-blocking socket options failed: {}", e);
            os_error()
        });
        log::info!("connected to heuristic at {}", OPTIONS.heuristic_address);
        Mutex::new(BufStream::new(socket))
    };
}

pub fn send_to_heuristic(f: &Id<Formula>) -> bool {
    let id = Id::id(f);
    {
        let mut in_flight = IN_FLIGHT.write();
        in_flight.insert(id, f.clone());
    }

    let (nodes, edges) = flatten(f.into());
    let record = OutRecord { id, nodes, edges };
    let mut serialized = to_string(&record).unwrap();
    serialized.push('\n');
    let write_result = SOCKET.lock().write_all(serialized.as_bytes());
    match write_result {
        Ok(_) => true,
        Err(ref e) if e.kind() == WouldBlock => false,
        Err(e) => {
            log::error!("failed to write to heuristic socket: {}", e);
            os_error()
        }
    }
}

pub fn receive_from_heuristic() -> Option<(Id<Formula>, Score)> {
    let mut line = String::new();
    let read_result = SOCKET.lock().read_line(&mut line);
    match read_result {
        Ok(_) => {}
        Err(ref e) if e.kind() == WouldBlock => return None,
        Err(e) => {
            log::error!("failed to read from heuristic socket: {}", e);
            os_error()
        }
    };
    if line == "\n" {
        return None;
    }
    let record: InRecord = from_str(&line).unwrap_or_else(|e| {
        log::error!("heuristic provided bad data: {}", e);
        os_error();
    });

    let f = match IN_FLIGHT.read().get(&record.id) {
        Some(f) => f.clone(),
        None => {
            return None;
        }
    };
    IN_FLIGHT.write().remove(&record.id);

    let score = record.score.into();
    Some((f, score))
}

pub fn initialize() {
    if !OPTIONS.heuristic_off {
        lazy_static::initialize(&IN_FLIGHT);
        lazy_static::initialize(&SOCKET);
    }
}
