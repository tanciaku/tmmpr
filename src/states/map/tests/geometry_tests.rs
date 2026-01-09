#[cfg(test)]
mod tests {
    use crate::states::map::geometry::SignedRect;

    #[test]
    fn test_signed_rect_no_intersection() {
        let rect1 = SignedRect { x: 0, y: 0, width: 10, height: 10 };
        let rect2 = SignedRect { x: 20, y: 20, width: 10, height: 10 };
        
        assert!(rect1.intersection(&rect2).is_none());
        assert!(rect2.intersection(&rect1).is_none());
    }

    #[test]
    fn test_signed_rect_touching_rectangles() {
        // Rectangles that touch at edges but don't overlap
        let rect1 = SignedRect { x: 0, y: 0, width: 10, height: 10 };
        let rect2 = SignedRect { x: 10, y: 0, width: 10, height: 10 };
        
        assert!(rect1.intersection(&rect2).is_none());
        assert!(rect2.intersection(&rect1).is_none());
    }

    #[test]
    fn test_signed_rect_partial_intersection() {
        let rect1 = SignedRect { x: 0, y: 0, width: 10, height: 10 };
        let rect2 = SignedRect { x: 5, y: 5, width: 10, height: 10 };
        
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection.x, 5);
        assert_eq!(intersection.y, 5);
        assert_eq!(intersection.width, 5);
        assert_eq!(intersection.height, 5);
        
        // Test symmetry
        let intersection2 = rect2.intersection(&rect1).unwrap();
        assert_eq!(intersection.x, intersection2.x);
        assert_eq!(intersection.y, intersection2.y);
        assert_eq!(intersection.width, intersection2.width);
        assert_eq!(intersection.height, intersection2.height);
    }

    #[test]
    fn test_signed_rect_complete_containment() {
        let outer = SignedRect { x: 0, y: 0, width: 20, height: 20 };
        let inner = SignedRect { x: 5, y: 5, width: 10, height: 10 };
        
        let intersection = outer.intersection(&inner).unwrap();
        assert_eq!(intersection.x, 5);
        assert_eq!(intersection.y, 5);
        assert_eq!(intersection.width, 10);
        assert_eq!(intersection.height, 10);
        
        // Test reverse containment
        let intersection2 = inner.intersection(&outer).unwrap();
        assert_eq!(intersection.x, intersection2.x);
        assert_eq!(intersection.y, intersection2.y);
        assert_eq!(intersection.width, intersection2.width);
        assert_eq!(intersection.height, intersection2.height);
    }

    #[test]
    fn test_signed_rect_identical_rectangles() {
        let rect1 = SignedRect { x: 10, y: 15, width: 20, height: 25 };
        let rect2 = SignedRect { x: 10, y: 15, width: 20, height: 25 };
        
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection.x, 10);
        assert_eq!(intersection.y, 15);
        assert_eq!(intersection.width, 20);
        assert_eq!(intersection.height, 25);
    }

    #[test]
    fn test_signed_rect_negative_coordinates() {
        // Test with negative coordinates (important for screen-space calculations)
        let rect1 = SignedRect { x: -10, y: -10, width: 20, height: 20 };
        let rect2 = SignedRect { x: -5, y: -5, width: 10, height: 10 };
        
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection.x, -5);
        assert_eq!(intersection.y, -5);
        assert_eq!(intersection.width, 10);
        assert_eq!(intersection.height, 10);
    }

    #[test]
    fn test_signed_rect_viewport_clipping() {
        // Simulate a viewport and a note that's partially off-screen
        let viewport = SignedRect { x: 0, y: 0, width: 100, height: 100 };
        let note_partially_offscreen = SignedRect { x: -20, y: 50, width: 40, height: 30 };
        
        let visible_area = note_partially_offscreen.intersection(&viewport).unwrap();
        assert_eq!(visible_area.x, 0);
        assert_eq!(visible_area.y, 50);
        assert_eq!(visible_area.width, 20);
        assert_eq!(visible_area.height, 30);
    }

    #[test]
    fn test_signed_rect_completely_offscreen() {
        // Note completely to the left of the viewport
        let viewport = SignedRect { x: 0, y: 0, width: 100, height: 100 };
        let note_offscreen = SignedRect { x: -50, y: 50, width: 30, height: 30 };
        
        assert!(note_offscreen.intersection(&viewport).is_none());
    }

    #[test]
    fn test_signed_rect_zero_dimensions() {
        let rect1 = SignedRect { x: 0, y: 0, width: 0, height: 10 };
        let rect2 = SignedRect { x: 0, y: 0, width: 10, height: 10 };
        
        assert!(rect1.intersection(&rect2).is_none());
        
        let rect3 = SignedRect { x: 0, y: 0, width: 10, height: 0 };
        assert!(rect3.intersection(&rect2).is_none());
    }

    #[test]
    fn test_signed_rect_single_pixel_intersection() {
        let rect1 = SignedRect { x: 0, y: 0, width: 10, height: 10 };
        let rect2 = SignedRect { x: 9, y: 9, width: 10, height: 10 };
        
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection.x, 9);
        assert_eq!(intersection.y, 9);
        assert_eq!(intersection.width, 1);
        assert_eq!(intersection.height, 1);
    }
}