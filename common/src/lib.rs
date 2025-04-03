use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Text {
    pub segments: Vec<Segment>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Segment {
    pub words: Vec<Word>,
    pub audio: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Word {
    pub content: String,
    pub start: f64,
    pub end: f64,
}
