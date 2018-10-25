use std::sync::Arc;

use collections::Set;
use formula::Formula;

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Justification {
    pub derived: Goal,
    pub from: Set<Arc<Formula>>,
    pub method: &'static str
}

pub trait JustificationSink {
    fn justify(&mut self, justification: Justification);
}

pub struct NullJustificationSink;

impl JustificationSink for NullJustificationSink {
    fn justify(&mut self, _justification: Justification) {}
}

pub struct RecordingJustificationSink {
    recorded: Vec<Justification>
}

impl RecordingJustificationSink {
    pub fn new() -> Self {
        RecordingJustificationSink {
            recorded: vec![]
        }
    }

    pub fn justifications(self) -> Vec<Justification> {
        self.recorded.into_iter()
    }
}

impl JustificationSink for RecordingJustificationSink {
    fn justify(&mut self, justification: Justification) {
        self.recorded.push(justification)
    }
}

macro_rules! justify {
    ($sink:ident, [$($from:tt)*] => $to:expr, $inference:tt) => {
        $sink.justify(::justification::Justification {
            derived: $to,
            from: set![$($from)*],
            method: stringify!($inference)
        })
    };
}
