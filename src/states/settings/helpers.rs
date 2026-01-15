use crate::states::map::Side;

pub fn cycle_side(side: Side) -> Side {
    match side {
        Side::Right => Side::Bottom,
        Side::Bottom => Side::Left,
        Side::Left => Side::Top,
        Side::Top => Side::Right,
    }
}

pub fn side_to_string(side: Side) -> String {
    match side {
        Side::Right => String::from("Right"),
        Side::Bottom => String::from("Bottom"),
        Side::Left => String::from("Left"),
        Side::Top => String::from("Top"),
    }
}