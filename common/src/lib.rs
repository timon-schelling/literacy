use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Text {
    pub segments: Vec<Segment>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Segment {
    pub words: Vec<Word>,
    pub audio: Audio,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Audio {
    None,
    Wav(Wav),
    Ref(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Wav {
    Raw(Vec<u8>),
    Compressed(Vec<u8>),
    Base64(String),
    Base64Compressed(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Word {
    pub content: String,
    pub start: f64,
    pub end: f64,
}
