use std::{fs, path::Path};

#[derive(PartialEq, Debug)]
pub enum IoErrorKind {
    DirFind,
    DirCreate,
    FileRead,
    FileWrite,
}

/// Writes data as pretty-printed JSON, overwriting if file exists.
pub fn write_json_data<T>(path: &Path, data: &T) -> Result<(), Box<dyn std::error::Error>>
where
    T: serde::Serialize,
{
    let json_string = serde_json::to_string_pretty(data)?;
    fs::write(path, json_string)?;
    Ok(())
}

/// Reads and deserializes JSON data from file.
pub fn read_json_data<T>(path: &Path) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let json_string = fs::read_to_string(path)?;
    let data: T = serde_json::from_str(&json_string)?;
    Ok(data)
}
