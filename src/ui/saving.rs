use serde::Serialize;

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::WallBlock;

#[derive(Serialize)]
struct SaveData<'a> {
    sources: &'a Vec<&'a Source>,
    mics: &'a Vec<&'a Microphone>,
    wallblocks: &'a Vec<&'a WallBlock>,
}

pub fn save(
    sources: &Vec<&Source>,
    mics: &Vec<&Microphone>,
    wallblocks: &Vec<&WallBlock>,
) -> Result<Vec<u8>, serde_json::Error> {
    let save_data = SaveData {
        sources,
        mics,
        wallblocks,
    };

    serde_json::to_vec(&save_data)
}
