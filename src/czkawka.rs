use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct CzkawkaFileEntry {
    pub path: String,
    #[serde(default)]
    pub similarity: u32,
}
pub type CzkawkaSimilarFileGroup = Vec<CzkawkaFileEntry>;
pub type CzKawkaSimilarResult = Vec<CzkawkaSimilarFileGroup>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cannot open file: {0}")]
    Path(#[from] std::io::Error),

    #[error("Cannot JSON deserialize file: {0}")]
    Parse(#[from] serde_json::Error),
}

pub fn parse_similar_result(result_file_path: &str) -> Result<CzKawkaSimilarResult, Error> {
    let czkawka_result = fs::File::open(result_file_path)?;
    let similar_file_group: Vec<CzkawkaSimilarFileGroup> = serde_json::from_reader(czkawka_result)?;
    Ok(similar_file_group)
}
