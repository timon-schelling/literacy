use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Text {
    pub segments: Vec<Segment>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Segment {
    pub words: Vec<Word>,
    pub audio: Audio,
    pub duration: f64,
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
#[serde(untagged)]
pub enum Word {
    Raw(String),
    Timestamped {
        content: String,
        start: f64,
        end: f64,
    },
}

impl Into<String> for Word {
    fn into(self) -> String {
        match self {
            Word::Raw(content) => content,
            Word::Timestamped { content, .. } => content,
        }
    }
}

impl Into<String> for &Word {
    fn into(self) -> String {
        match self {
            Word::Raw(content) => content.clone(),
            Word::Timestamped { content, .. } => content.clone(),
        }
    }
}
