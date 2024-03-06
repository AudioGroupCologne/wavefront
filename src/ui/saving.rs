use serde::Serialize;

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::Wall;

#[derive(Serialize)]
struct SaveData<'a> {
    sources: &'a Vec<&'a Source>,
    mics: &'a Vec<&'a Microphone>,
    walls: &'a Vec<&'a Wall>,
}

pub fn save(
    sources: &Vec<&Source>,
    mics: &Vec<&Microphone>,
    walls: &Vec<&Wall>,
) -> Result<Vec<u8>, serde_json::Error> {
    let save_data = SaveData {
        sources,
        mics,
        walls,
    };

    serde_json::to_vec(&save_data)
}
