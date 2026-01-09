use super::super::note::Note;
use super::super::enums::Side;
use ratatui::style::Color;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation() {
        let note = Note::new(10, 5, "Hello World".to_string(), true, Color::Red);
        
        assert_eq!(note.x, 10);
        assert_eq!(note.y, 5);
        assert_eq!(note.content, "Hello World");
        assert_eq!(note.selected, true);
        assert_eq!(note.color, Color::Red);
    }

    #[test]
    fn test_get_dimensions_empty_content() {
        let note = Note::new(0, 0, "".to_string(), false, Color::White);
        let (width, height) = note.get_dimensions();
        
        // Empty content should have width 0 + 2 (border) = 2, height 1 + 2 (border) = 3
        assert_eq!(width, 2);
        assert_eq!(height, 3);
    }

    #[test]
    fn test_get_dimensions_single_line() {
        let note = Note::new(0, 0, "Hello".to_string(), false, Color::White);
        let (width, height) = note.get_dimensions();
        
        // "Hello" = 5 chars + 2 (border) = 7 width, 1 line + 2 (border) = 3 height
        assert_eq!(width, 7);
        assert_eq!(height, 3);
    }

    #[test]
    fn test_get_dimensions_multiline() {
        let note = Note::new(0, 0, "Hello\nWorld\nTest".to_string(), false, Color::White);
        let (width, height) = note.get_dimensions();
        
        // 3 lines (2 newlines + 1) + 2 (border) = 5 height
        // Longest line is "Hello" or "World" = 5 chars + 2 (border) = 7 width
        assert_eq!(width, 7);
        assert_eq!(height, 5);
    }

    #[test]
    fn test_get_dimensions_multiline_different_lengths() {
        let note = Note::new(0, 0, "Hi\nThis is a longer line\nShort".to_string(), false, Color::White);
        let (width, height) = note.get_dimensions();
        
        // 3 lines + 2 (border) = 5 height
        // Longest line is "This is a longer line" = 21 chars + 2 (border) = 23 width
        assert_eq!(width, 23);
        assert_eq!(height, 5);
    }

    #[test]
    fn test_get_dimensions_trailing_newline() {
        let note = Note::new(0, 0, "Hello\nWorld\n".to_string(), false, Color::White);
        let (width, height) = note.get_dimensions();
        
        // 2 newlines + 1 = 3 lines, + 2 (border) = 5 height
        // Longest line is "Hello" or "World" = 5 chars + 2 (border) = 7 width
        assert_eq!(width, 7);
        assert_eq!(height, 5);
    }

    #[test]
    fn test_get_connection_point_right() {
        let note = Note::new(10, 20, "Test\nContent".to_string(), false, Color::White);
        let (x, y) = note.get_connection_point(Side::Right);
        
        let (width, height) = note.get_dimensions();
        let mut actual_width = width;
        let mut actual_height = height;
        
        // Apply minimum size constraints
        if actual_width < 20 { actual_width = 20; }
        if actual_height < 4 { actual_height = 4; }
        actual_width += 1; // cursor space
        
        let expected_x = note.x + actual_width as usize - 1;
        let expected_y = note.y + (actual_height / 2) as usize;
        
        assert_eq!(x, expected_x);
        assert_eq!(y, expected_y);
    }

    #[test]
    fn test_get_connection_point_left() {
        let note = Note::new(10, 20, "Test\nContent".to_string(), false, Color::White);
        let (x, y) = note.get_connection_point(Side::Left);
        
        let (_, height) = note.get_dimensions();
        let mut actual_height = height;
        if actual_height < 4 { actual_height = 4; }
        
        let expected_x = note.x;
        let expected_y = note.y + (actual_height / 2) as usize;
        
        assert_eq!(x, expected_x);
        assert_eq!(y, expected_y);
    }

    #[test]
    fn test_get_connection_point_top() {
        let note = Note::new(10, 20, "Test\nContent".to_string(), false, Color::White);
        let (x, y) = note.get_connection_point(Side::Top);
        
        let (width, _) = note.get_dimensions();
        let mut actual_width = width;
        if actual_width < 20 { actual_width = 20; }
        actual_width += 1; // cursor space
        
        let expected_x = note.x + (actual_width / 2) as usize;
        let expected_y = note.y;
        
        assert_eq!(x, expected_x);
        assert_eq!(y, expected_y);
    }

    #[test]
    fn test_get_connection_point_bottom() {
        let note = Note::new(10, 20, "Test\nContent".to_string(), false, Color::White);
        let (x, y) = note.get_connection_point(Side::Bottom);
        
        let (width, height) = note.get_dimensions();
        let mut actual_width = width;
        let mut actual_height = height;
        
        if actual_width < 20 { actual_width = 20; }
        if actual_height < 4 { actual_height = 4; }
        actual_width += 1; // cursor space
        
        let expected_x = note.x + (actual_width / 2) as usize;
        let expected_y = note.y + actual_height as usize - 1;
        
        assert_eq!(x, expected_x);
        assert_eq!(y, expected_y);
    }

    #[test]
    fn test_get_connection_point_minimum_size() {
        // Test with a very small note to ensure minimum size constraints are applied
        let note = Note::new(0, 0, "Hi".to_string(), false, Color::White);
        
        // Right side connection
        let (x_right, _) = note.get_connection_point(Side::Right);
        // Width should be enforced to minimum 20 + 1 (cursor) = 21
        // Connection point should be at x + 21 - 1 = x + 20
        assert_eq!(x_right, note.x + 20);
    }

    #[test]
    fn test_note_with_unicode_content() {
        let note = Note::new(0, 0, "Hello 🌍\nWorld 🚀".to_string(), false, Color::White);
        let (width, height) = note.get_dimensions();
        
        // "Hello 🌍" = 7 chars (Hello + space + emoji), "World 🚀" = 7 chars
        // Width should be 7 + 2 (border) = 9
        // Height should be 2 lines + 2 (border) = 4
        assert_eq!(width, 9);
        assert_eq!(height, 4);
    }

    #[test]
    fn test_connection_points_all_sides() {
        let note = Note::new(100, 200, "Large\nNote\nContent\nHere".to_string(), false, Color::Blue);
        
        let right = note.get_connection_point(Side::Right);
        let left = note.get_connection_point(Side::Left);
        let top = note.get_connection_point(Side::Top);
        let bottom = note.get_connection_point(Side::Bottom);
        
        // All connection points should have reasonable coordinates
        assert!(right.0 > note.x);
        assert_eq!(left.0, note.x);
        assert!(top.1 == note.y);
        assert!(bottom.1 > note.y);
        
        // Right and left should have same y-coordinate (middle of note)
        assert_eq!(right.1, left.1);
        
        // Top and bottom should have same x-coordinate (middle of note)
        assert_eq!(top.0, bottom.0);
    }
}