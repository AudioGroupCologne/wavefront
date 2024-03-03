use std::path::Path;

use serde::Deserialize;

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::WallBlock;

#[derive(Deserialize)]
pub struct SaveData {
    pub sources: Vec<Source>,
    pub mics: Vec<Microphone>,
    pub wallblocks: Vec<WallBlock>,
}

pub fn load(path: &Path) -> SaveData {
    let file = std::fs::File::open(path).unwrap();
    serde_json::from_reader(file).unwrap()
}
