use ratatui::style::Color;
use serde::{Serializer, Deserializer, Deserialize};
use crate::utils::{get_color_name_in_string, get_color_from_string};

pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let color_string = get_color_name_in_string(*color);
    serializer.serialize_str(&color_string)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    
    Ok(get_color_from_string(&s))
}