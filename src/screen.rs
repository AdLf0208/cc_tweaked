//! Where the computer's terminal is drawn on its front face.
//!
//! Pure: no host, no world. `lib.rs` reads the block's state string from the host and the
//! pose here; this module owns the numbers the drawing uses, and the parse of the state's
//! `facing` (the pinned API reads a *position's* property, `level_block_property`, but not
//! a `BlockState`'s, so the submit hook parses the state's own string).

/// Site of the drawn glyphs as a fraction of a block: 6 px of advance at 1/96 puts 16
/// columns across the face (`runtime::TERM_WIDTH`).
pub const FONT_SCALE: f32 = 1.0 / 96.0;
/// Line height in font pixels: 10 lines of 9 px fit the face (`runtime::TERM_LINES`).
pub const LINE_PIXELS: f32 = 9.0;

/// The lower-left corner of the face in the pose's coordinates, and how far to turn the
/// pose about y so the glyphs face outwards and read left to right.
///
/// The pose starts at the block's lower north-west corner, its glyphs facing +z; a face
/// that is not `north` moves the corner and turns the glyphs by a quarter per side. A
/// facing the computer does not have draws as `north`, the block's default.
#[must_use]
pub fn pose(facing: &str) -> ([f32; 3], f32) {
    match facing {
        "south" => ([0.0, 0.94, 1.008], 0.0),
        "east" => ([1.008, 0.94, 1.0], 90.0),
        "west" => ([-0.008, 0.94, 0.0], 270.0),
        _ => ([1.0, 0.94, -0.008], 180.0),
    }
}

/// The value of `facing` in a block state string, as `/setblock` writes one:
/// `computercraft:computer_normal[facing=east,state=off]` names `east`. `None` for a state
/// with no properties, or with no `facing` among them.
#[must_use]
pub fn facing_of(state: &str) -> Option<&str> {
    let (_, properties) = state.split_once('[')?;
    let properties = properties.strip_suffix(']').unwrap_or(properties);
    properties
        .split(',')
        .filter_map(|property| property.split_once('='))
        .find_map(|(name, value)| (name == "facing").then_some(value))
}

/// Where line `index`'s left end sits, in font pixels from the pose's origin (y down).
#[must_use]
pub fn line_origin(index: usize) -> [f32; 2] {
    [0.0, index as f32 * LINE_PIXELS]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_state_string_names_the_facing() {
        assert_eq!(
            facing_of("computercraft:computer_normal[facing=east,state=off]"),
            Some("east")
        );
        assert_eq!(
            facing_of("computercraft:computer_normal[facing=north,state=blinking]"),
            Some("north")
        );
    }

    #[test]
    fn a_property_after_the_facing_is_read_past() {
        assert_eq!(
            facing_of("computercraft:computer_normal[state=on,facing=west]"),
            Some("west")
        );
    }

    #[test]
    fn a_state_without_properties_has_no_facing() {
        assert_eq!(facing_of("computercraft:computer_normal"), None);
        assert_eq!(facing_of("minecraft:stone"), None);
        assert_eq!(facing_of("computercraft:computer_normal[state=off]"), None);
        assert_eq!(facing_of(""), None);
    }

    #[test]
    fn a_facing_value_is_not_confused_with_its_property_name() {
        // `facing=` inside a value is not a property boundary.
        assert_eq!(
            facing_of("computercraft:computer_normal[north_facing=none,facing=south]"),
            Some("south")
        );
    }

    #[test]
    fn each_facing_has_its_own_corner_and_turn() {
        assert_eq!(pose("south"), ([0.0, 0.94, 1.008], 0.0));
        assert_eq!(pose("east"), ([1.008, 0.94, 1.0], 90.0));
        assert_eq!(pose("west"), ([-0.008, 0.94, 0.0], 270.0));
        assert_eq!(pose("north"), ([1.0, 0.94, -0.008], 180.0));
    }

    #[test]
    fn a_facing_the_block_does_not_have_falls_back_to_north() {
        assert_eq!(pose("up"), pose("north"));
    }

    #[test]
    fn lines_step_down_the_face() {
        assert_eq!(line_origin(0), [0.0, 0.0]);
        assert_eq!(line_origin(1), [0.0, LINE_PIXELS]);
        assert_eq!(line_origin(9), [0.0, 9.0 * LINE_PIXELS]);
    }
}
