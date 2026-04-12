use super::super::enums::Side;
use ratatui::style::Color;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::states::map::new_note;

    #[test]
    fn test_note_creation() {
        let note = new_note(10, 5, "Hello World".to_string(), Color::Red);

        assert_eq!(note.x, 10);
        assert_eq!(note.y, 5);
        assert_eq!(note.data.content, "Hello World");
        assert_eq!(note.data.color, Color::Red);
    }

    #[test]
    fn test_get_dimensions_empty_content() {
        let note = new_note(0, 0, "".to_string(), Color::White);

        let (width, height) = note.get_dimensions();

        assert_eq!(width, 21);
        assert_eq!(height, 4);
    }

    #[test]
    fn test_get_dimensions_single_line() {
        let note = new_note(0, 0, "Hello".to_string(), Color::White);
        let (width, height) = note.get_dimensions();

        assert_eq!(width, 21);
        assert_eq!(height, 4);
    }

    #[test]
    fn test_get_dimensions_multiline() {
        let note = new_note(0, 0, "Hello\nWorld\nTest".to_string(), Color::White);
        let (width, height) = note.get_dimensions();

        // 3 lines (2 newlines + 1) + 2 (border) = 5 height
        assert_eq!(width, 21);
        assert_eq!(height, 5);
    }

    #[test]
    fn test_get_dimensions_multiline_different_lengths() {
        let note = new_note(
            0,
            0,
            "Hi\nThis is a looooonger line\nShort".to_string(),
            Color::White,
        );
        let (width, height) = note.get_dimensions();

        // Longest line is "This is a looooonger line" = 25 chars + 2 (border) + 1 (cursor) = 28 width
        assert_eq!(width, 28);
        assert_eq!(height, 5);
    }

    #[test]
    fn test_get_dimensions_trailing_newline() {
        let note = new_note(0, 0, "Hello\nWorld\n".to_string(), Color::White);
        let (width, height) = note.get_dimensions();

        // 2 newlines + 1 = 3 lines, + 2 (border) = 5 height
        assert_eq!(width, 21);
        assert_eq!(height, 5);
    }

    #[test]
    fn test_get_connection_point_right() {
        let note = new_note(10, 20, "Test\nContent".to_string(), Color::White);
        let (x, y) = note.get_connection_point(Side::Right);

        let (width, height) = note.get_dimensions();

        let expected_x = note.x + width as usize - 1;
        let expected_y = note.y + (height / 2) as usize;

        assert_eq!(x, expected_x);
        assert_eq!(y, expected_y);
    }

    #[test]
    fn test_get_connection_point_left() {
        let note = new_note(10, 20, "Test\nContent".to_string(), Color::White);
        let (x, y) = note.get_connection_point(Side::Left);

        let (_, height) = note.get_dimensions();
        let expected_x = note.x;
        let expected_y = note.y + (height / 2) as usize;

        assert_eq!(x, expected_x);
        assert_eq!(y, expected_y);
    }

    #[test]
    fn test_get_connection_point_top() {
        let note = new_note(10, 20, "Test\nContent".to_string(), Color::White);
        let (x, y) = note.get_connection_point(Side::Top);

        let (width, _) = note.get_dimensions();

        let expected_x = note.x + (width / 2) as usize;
        let expected_y = note.y;

        assert_eq!(x, expected_x);
        assert_eq!(y, expected_y);
    }

    #[test]
    fn test_get_connection_point_bottom() {
        let note = new_note(10, 20, "Test\nContent".to_string(), Color::White);
        let (x, y) = note.get_connection_point(Side::Bottom);

        let (width, height) = note.get_dimensions();

        let expected_x = note.x + (width / 2) as usize;
        let expected_y = note.y + height as usize - 1;

        assert_eq!(x, expected_x);
        assert_eq!(y, expected_y);
    }

    #[test]
    fn test_get_connection_point_minimum_size() {
        // Test with a very small note to ensure minimum size constraints are applied
        let note = new_note(0, 0, "Hi".to_string(), Color::White);

        // Right side connection
        let (x_right, _) = note.get_connection_point(Side::Right);
        // Width should be enforced to minimum 20 + 1 (cursor) = 21
        // Connection point should be at x + 21 - 1 = x + 20
        assert_eq!(x_right, note.x + 20);
    }

    #[test]
    fn test_note_with_unicode_content() {
        let note = new_note(
            0,
            0,
            "Sample note text!! 📝✨🎯💡🔥🎨".to_string(),
            Color::White,
        );
        let (width, height) = note.get_dimensions();

        // "Sample note text!! 📝✨🎯💡🔥🎨" = 31 chars + 2 (border) + 1 (cursor) = 34 width
        assert_eq!(width, 34);
        assert_eq!(height, 4);
    }

    #[test]
    fn test_connection_points_all_sides() {
        let note = new_note(
            100,
            200,
            "Large\nNote\nContent\nHere".to_string(),
            Color::Blue,
        );

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
