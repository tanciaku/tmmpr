use ratatui::style::Color;

/// Converts a `Color` to its string representation.
///
/// Returns an empty string for unsupported colors.
pub fn get_color_name_in_string(color: Color) -> String {
    String::from(match color {
        Color::Red => "Red",
        Color::Green => "Green",
        Color::Yellow => "Yellow",
        Color::Blue => "Blue",
        Color::Magenta => "Magenta",
        Color::Cyan => "Cyan",
        Color::White => "White",
        Color::Black => "Black",
        _ => "",
    })
}

/// Parses a color name string into a `Color`.
///
/// Defaults to `White` for unrecognized color names.
pub fn get_color_from_string(color_name_str: &str) -> Color {
    match color_name_str {
        "Red" => Color::Red,
        "Green" => Color::Green,
        "Yellow" => Color::Yellow,
        "Blue" => Color::Blue,
        "Magenta" => Color::Magenta,
        "Cyan" => Color::Cyan,
        "White" => Color::White,
        "Black" => Color::Black,
        _ => Color::White,
    }
}
