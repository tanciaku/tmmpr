use std::{fs, path::Path};

/// Writes the relevant data to a file
pub fn write_json_data<T>(path: &Path, data: &T) -> Result<(), Box<dyn std::error::Error>>
where
    T: serde::Serialize,
{
    let json_string = serde_json::to_string_pretty(data)?;
    fs::write(path, json_string)?;
    Ok(())
}

/// Reads the relevant data from a file
pub fn read_json_data<T>(path: &Path) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let json_string = fs::read_to_string(path)?;
    let data: T = serde_json::from_str(&json_string)?;
    Ok(data)
}
