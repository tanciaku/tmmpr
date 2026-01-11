use ratatui::style::Color;

/// Helper function for displaying the color currently set for the selected note/connection
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
