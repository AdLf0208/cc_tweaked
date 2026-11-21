//! The computer block family: the three blocks the jar ships that are computers.
//!
//! Pure: no host, no world. The jar has sixteen blockstates for sixteen blocks; three of them are
//! computers (`computer_normal`, `computer_advanced`, `computer_command`) and share one block
//! entity, one terminal and one set of commands. Everything else the jar ships (monitors, disk
//! drives, turtles, cables, modems, the printer, the speaker, the lectern, the relay) is a
//! different family and not part of this one.
//!
//! Two spellings are needed, and the difference is a platform fact: the block entity type is
//! declared with **paths** (`computer_normal`, namespaced by the package's own id), while the
//! world guards and the command readout see **ids** (`computercraft:computer_normal`). Keeping
//! both in one place is what stops the two from drifting apart.

/// The three computers, by block id.
pub const COMPUTERS: [&str; 3] = [
    "computercraft:computer_normal",
    "computercraft:computer_advanced",
    "computercraft:computer_command",
];

/// The same three, by path: what the registrar's `blocks(&[…])` takes.
pub const PATHS: [&str; 3] = ["computer_normal", "computer_advanced", "computer_command"];

/// Whether `block` (a block id) is one of this port's computers.
#[must_use]
pub fn is_computer(block: &str) -> bool {
    COMPUTERS.contains(&block)
}

/// The registrar path of `block`, when it is one of this port's computers.
#[must_use]
pub fn path(block: &str) -> Option<&'static str> {
    COMPUTERS
        .iter()
        .position(|member| *member == block)
        .map(|index| PATHS[index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_computer_the_jar_ships_is_a_member() {
        for block in COMPUTERS {
            assert!(is_computer(block), "{block}");
        }
        assert_eq!(
            COMPUTERS.len(),
            3,
            "the jar's sixteen blockstates hold three computers"
        );
    }

    #[test]
    fn another_block_is_not_a_member() {
        // The jar's own neighbours, a vanilla block and the empty string: none of them may be
        // treated as a computer (the block entity would be read from a block that has none).
        for block in [
            "computercraft:monitor_normal",
            "computercraft:computer",
            "computercraft:computer_normal_2",
            "minecraft:stone",
            "other:computer_normal",
            "",
        ] {
            assert!(!is_computer(block), "{block}");
        }
    }

    #[test]
    fn a_path_names_the_computer_it_belongs_to() {
        assert_eq!(
            path("computercraft:computer_advanced"),
            Some("computer_advanced")
        );
        assert_eq!(
            path("computercraft:computer_command"),
            Some("computer_command")
        );
        assert_eq!(path("minecraft:stone"), None);
    }

    #[test]
    fn the_two_spellings_line_up() {
        // The registrar takes paths and the world guards take ids: an id whose path is not
        // declared would be a computer the block entity type does not cover.
        for (index, member) in PATHS.iter().enumerate() {
            assert_eq!(COMPUTERS[index], format!("computercraft:{member}"));
            assert_eq!(path(COMPUTERS[index]), Some(*member));
        }
    }
}
