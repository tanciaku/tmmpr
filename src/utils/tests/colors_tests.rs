use ratatui::style::Color;

use crate::utils::colors::{get_color_name_in_string, get_color_from_string};

#[test]
fn test_get_color_name_in_string() {
    assert_eq!(get_color_name_in_string(Color::Red), "Red");
    assert_eq!(get_color_name_in_string(Color::Green), "Green");
    assert_eq!(get_color_name_in_string(Color::White), "White");
    
    // Unsupported colors return empty string
    assert_eq!(get_color_name_in_string(Color::Rgb(128, 128, 128)), "");
}

#[test]
fn test_get_color_from_string() {
    assert_eq!(get_color_from_string("Red"), Color::Red);
    assert_eq!(get_color_from_string("Blue"), Color::Blue);
    assert_eq!(get_color_from_string("Cyan"), Color::Cyan);
    
    // Invalid input defaults to White
    assert_eq!(get_color_from_string("InvalidColor"), Color::White);
    assert_eq!(get_color_from_string(""), Color::White);
}
