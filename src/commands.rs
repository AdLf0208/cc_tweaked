//! The `/computercraft` command surface: what a player asked of a computer, and the one
//! line the screen readout answers.
//!
//! Pure: no host, no world. `lib.rs` reads the typed invocation the server's dispatcher
//! parsed and hands the pieces here; this module decides what they mean and formats the
//! readout the e2e parses.

/// What a player asked a computer to do, once the subcommand and its arguments are known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// `load <pos> <program>`: replace the stored program; it runs on the next tick.
    Load {
        /// The program source, as typed (a chat line cannot hold a newline: separate
        /// statements with `;`).
        program: String,
    },
    /// `run <pos>`: run the stored program again.
    Run,
    /// `screen <pos>`: report the terminal as one line of JSON.
    Screen,
    /// `power <pos> on|off`: switch the computer on or off.
    Power {
        /// Whether the computer is switched on.
        on: bool,
    },
}

/// What one invocation of our command name asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asked {
    /// A recognised usage.
    Do(Action),
    /// Our command name, but the arguments do not form a usage: answer this hint.
    Refused(&'static str),
    /// A subcommand of ours that this slice does not implement.
    Unknown,
}

/// The usage line `/computercraft load` answers when its program is empty.
pub const LOAD_USAGE: &str = "Usage: /computercraft load <x> <y> <z> <program>";
/// The usage line `/computercraft power` answers for a switch that is neither on nor off.
pub const POWER_USAGE: &str = "Usage: /computercraft power <x> <y> <z> on|off";

/// The operator level every usage is registered at. The four commands act on a
/// computer by coordinates with no ownership or reach rule of their own (a
/// computer has no owner record and a console source has no position), so they
/// are gamemaster tools, as `/data` is: a player cannot power-cycle or read
/// another player's computer through them.
pub const OPERATOR_LEVEL: u8 = 2;

/// The four usages and the level each is registered at.
#[must_use]
pub fn usages() -> [(&'static str, u8); 4] {
    [
        (
            "computercraft load <pos:block_pos> <program:text>",
            OPERATOR_LEVEL,
        ),
        ("computercraft run <pos:block_pos>", OPERATOR_LEVEL),
        ("computercraft screen <pos:block_pos>", OPERATOR_LEVEL),
        (
            "computercraft power <pos:block_pos> <switch:word>",
            OPERATOR_LEVEL,
        ),
    ]
}

/// The subcommand and the arguments the dispatcher parsed, read as an [`Asked`].
///
/// `program` is the `load` usage's greedy `text` argument and `switch` the `power` usage's
/// `word`; both are `None` for the usages that do not take them.
#[must_use]
pub fn asked(sub: &str, program: Option<&str>, switch: Option<&str>) -> Asked {
    match sub {
        "load" => match program.map(str::trim) {
            Some(source) if !source.is_empty() => Asked::Do(Action::Load {
                program: source.to_owned(),
            }),
            _ => Asked::Refused(LOAD_USAGE),
        },
        "run" => Asked::Do(Action::Run),
        "screen" => Asked::Do(Action::Screen),
        "power" => match switch {
            Some("on") => Asked::Do(Action::Power { on: true }),
            Some("off") => Asked::Do(Action::Power { on: false }),
            _ => Asked::Refused(POWER_USAGE),
        },
        _ => Asked::Unknown,
    }
}

/// The one line `/computercraft screen <pos>` answers, and what the e2e parses with `jq`:
///
/// ```json
/// {"block":"computercraft:computer_normal","pos":[1,2,3],"powered":true,"running":true,"lines":["2"]}
/// ```
///
/// `block` is the block that was actually read: the family has three members, so a readout that
/// names one of them for all three would answer for a computer that is not there.
///
/// Written by hand rather than through a JSON crate: the port's only dependency is the
/// pinned API, so the escaping is ours and is tested.
#[must_use]
pub fn screen_json(
    block: &str,
    at: [i32; 3],
    powered: bool,
    running: bool,
    lines: &[String],
) -> String {
    let mut out = String::from("{\"block\":\"");
    escape_into(&mut out, block);
    out.push_str("\",\"pos\":[");
    out.push_str(&at[0].to_string());
    out.push(',');
    out.push_str(&at[1].to_string());
    out.push(',');
    out.push_str(&at[2].to_string());
    out.push_str("],\"powered\":");
    out.push_str(if powered { "true" } else { "false" });
    out.push_str(",\"running\":");
    out.push_str(if running { "true" } else { "false" });
    out.push_str(",\"lines\":[");
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('"');
        escape_into(&mut out, line);
        out.push('"');
    }
    out.push_str("]}");
    out
}

/// Writes `text` as the inside of a JSON string: `"` and `\` escaped, control characters as
/// `\u00XX`, and everything else as itself (the terminal holds Lua's output, which is UTF-8).
fn escape_into(out: &mut String, text: &str) {
    use core::fmt::Write as _;
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Every other C0 control and DEL are not printable in JSON.
            character if (character as u32) < 0x20 || character == '\u{7f}' => {
                let _ = write!(out, "\\u{:04x}", character as u32);
            }
            character => out.push(character),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_takes_the_typed_program() {
        assert_eq!(
            asked("load", Some("print(\"hi\")"), None),
            Asked::Do(Action::Load {
                program: "print(\"hi\")".to_owned()
            })
        );
    }

    #[test]
    fn load_surrounding_spaces_are_trimmed_but_inner_text_is_kept() {
        assert_eq!(
            asked("load", Some("  print(1); print(2)  "), None),
            Asked::Do(Action::Load {
                program: "print(1); print(2)".to_owned()
            })
        );
    }

    #[test]
    fn load_with_no_program_is_refused_with_its_usage() {
        assert_eq!(asked("load", None, None), Asked::Refused(LOAD_USAGE));
        assert_eq!(asked("load", Some("   "), None), Asked::Refused(LOAD_USAGE));
    }

    #[test]
    fn run_and_screen_take_no_argument() {
        assert_eq!(asked("run", None, None), Asked::Do(Action::Run));
        assert_eq!(asked("screen", None, None), Asked::Do(Action::Screen));
    }

    #[test]
    fn power_reads_on_and_off() {
        assert_eq!(
            asked("power", None, Some("on")),
            Asked::Do(Action::Power { on: true })
        );
        assert_eq!(
            asked("power", None, Some("off")),
            Asked::Do(Action::Power { on: false })
        );
    }

    #[test]
    fn power_refuses_a_switch_that_is_neither_on_nor_off() {
        assert_eq!(
            asked("power", None, Some("sideways")),
            Asked::Refused(POWER_USAGE)
        );
        assert_eq!(asked("power", None, None), Asked::Refused(POWER_USAGE));
    }

    #[test]
    fn a_subcommand_this_slice_does_not_have_is_unknown() {
        assert_eq!(asked("turtle", None, None), Asked::Unknown);
        assert_eq!(asked("", None, None), Asked::Unknown);
    }

    #[test]
    fn every_usage_acts_on_a_computer_by_coordinates_and_is_operator_only() {
        // A level-0 usage would let any player read, reprogram or power a computer
        // at any loaded position: there is no owner or reach check inside the
        // handlers, so the gate is the registration level.
        assert_eq!(OPERATOR_LEVEL, 2);
        let usages = usages();
        assert_eq!(usages.len(), 4);
        for (usage, level) in usages {
            assert_eq!(level, OPERATOR_LEVEL, "{usage}");
            assert!(usage.starts_with("computercraft "), "{usage}");
        }
    }

    #[test]
    fn the_readout_is_the_json_the_e2e_parses() {
        let lines = vec!["1+1".to_owned(), "2".to_owned()];
        assert_eq!(
            screen_json(
                "computercraft:computer_normal",
                [10, 64, -3],
                true,
                true,
                &lines
            ),
            "{\"block\":\"computercraft:computer_normal\",\"pos\":[10,64,-3],\
             \"powered\":true,\"running\":true,\"lines\":[\"1+1\",\"2\"]}"
        );
    }

    #[test]
    fn a_powered_off_computer_reports_an_empty_terminal() {
        assert_eq!(
            screen_json(
                "computercraft:computer_normal",
                [0, 0, 0],
                false,
                false,
                &[]
            ),
            "{\"block\":\"computercraft:computer_normal\",\"pos\":[0,0,0],\
             \"powered\":false,\"running\":false,\"lines\":[]}"
        );
    }

    #[test]
    fn the_readout_names_the_block_it_read() {
        // The family has three members: a readout that always said `computer_normal` would be
        // answering for a computer that is not at that position.
        for block in [
            "computercraft:computer_advanced",
            "computercraft:computer_command",
        ] {
            let json = screen_json(block, [4, 64, 4], true, true, &[]);
            assert!(
                json.contains(&format!("\"block\":\"{block}\",")),
                "got {json}"
            );
            assert!(!json.contains("computer_normal"), "got {json}");
        }
    }

    #[test]
    fn a_line_with_quotes_and_backslashes_stays_one_json_string() {
        let lines = vec!["say \"hi\" \\ bye".to_owned()];
        let json = screen_json(
            "computercraft:computer_normal",
            [1, 2, 3],
            true,
            true,
            &lines,
        );
        assert!(
            json.ends_with("\"lines\":[\"say \\\"hi\\\" \\\\ bye\"]}"),
            "got {json}"
        );
        // And the escapes are the only ones: a re-read of the text between the brackets
        // gives back the line.
        assert_eq!(json.matches('\\').count(), 4, "got {json}");
    }

    #[test]
    fn a_control_character_is_escaped_as_an_escape_not_written_raw() {
        let json = screen_json(
            "computercraft:computer_normal",
            [0, 0, 0],
            true,
            true,
            &["a\tb\u{7f}".to_owned()],
        );
        assert!(json.contains("\"a\\tb\\u007f\""), "got {json}");
    }
}
