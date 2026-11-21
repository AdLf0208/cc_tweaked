//! The peripheral surface: what a block beside a computer answers, the capability key it is asked
//! through, and the lookup a placed computer performs.
//!
//! CC: Tweaked's computers reach the world through `peripheral` (`IPeripheral`, `IPeripheralProvider`,
//! `ComputerSide`): every block next to a computer is asked whether it is a peripheral, and a
//! peripheral answers a type name, its method names and one method call at a time. This port keeps
//! the same three questions, in this module, and the *asking* half the pin has no surface for:
//!
//! - [`Peripheral`] is the trait a block answers with (`getType`, `getNames`, `call`);
//! - [`PERIPHERAL`] is this port's own [`CapabilityKey`] for it — the provider half of the pin
//!   (`Registrar::block_capability`) already lets a block answer an arbitrary key, and this is the
//!   key it answers;
//! - [`neighbour`] is the pure geometry: which block a computer's `side` reaches, given the
//!   computer's own `facing`.
//!
//! [`LuaValue`] is the interpreter's value set, so a peripheral call takes and answers the values a
//! program can print. Monitors, modems, disk drives, turtles and pocket upgrades answer the same key
//! in later slices; nothing here assumes which block is on the other side.
//!
//! **The key is namespace-owned.** A second crate can only answer it by publishing this trait from a
//! shared `[api]` crate (the pin refuses one capability name answered with two handler types), so
//! today the answering half is this package's own — the recorded gap `blocked(peripheral-api-crate)`
//! for any add-on.
use mod_api::{CapabilityKey, ServerHost};

use crate::runtime::LuaValue;

/// The capability name a peripheral answers. Upstream has no capability of its own: it asks every
/// block next to a computer (`IPeripheralProvider`), which the pin cannot express, so the lookup is
/// a capability the block registers for.
pub const KEY_NAME: &str = "computercraft:peripheral";

/// The key [`KEY_NAME`] names, as the registrar takes it and the lookup asks it.
pub const PERIPHERAL: CapabilityKey<dyn Peripheral> = CapabilityKey::new(KEY_NAME);

/// A block that answers [`PERIPHERAL`]: what `peripheral.getType` reports, the method names
/// `peripheral.getNames` lists, and `peripheral.call`'s dispatch.
///
/// One handler is built per lookup and asked through `&self` (the pin hands a capability's
/// handler out read-only), so a method keeps nothing in the handler: whatever it changes lives in
/// the block entity's saved compound or in the world, as NeoForge's capability handlers do.
pub trait Peripheral {
    /// The peripheral's type name (`IPeripheral.getType`), such as `computercraft:harness`.
    fn peripheral_type(&self) -> String;

    /// The methods this peripheral answers besides the two built-ins [`BUILTIN`] names
    /// (`IPeripheral.getMethodNames`).
    fn methods(&self) -> Vec<String>;

    /// One method call (`IPeripheral.callMethod`): `None` when this peripheral has no such method,
    /// `Some(Err(…))` when the method ran and refused its arguments. The message is the one the
    /// program sees, as Lua's own errors are.
    fn call(
        &self,
        host: &mut dyn ServerHost,
        method: &str,
        arguments: &[LuaValue],
    ) -> Option<Result<LuaValue, String>>;
}

/// The two methods `peripheral.call` answers for every peripheral, whatever [`Peripheral::methods`]
/// lists: upstream's `getType`/`getNames` are the wrapper's, so the interpreter answers them from
/// [`Peripheral::peripheral_type`] and [`Peripheral::methods`] and never asks a method list.
pub const BUILTIN: [&str; 2] = crate::runtime::PERIPHERAL_BUILTIN;

/// The six sides a computer answers, in the order upstream's `ComputerSide` lists them.
pub const SIDES: [&str; 6] = ["front", "back", "left", "right", "top", "bottom"];

/// The four horizontal facings a computer's blockstate may hold.
pub const FACINGS: [&str; 4] = ["north", "south", "west", "east"];

/// The block a computer at `pos` facing `facing` reaches through `side`, as `(position, normal)`:
/// the neighbour's position and the direction from the computer to it.
///
/// The six relative names are the computer's own, as upstream's `ComputerSide` has them: `front` is
/// the face the terminal is drawn on, and `left`/`right` are that face's left and right **seen from
/// the computer looking out of its screen** (a computer facing north reaches west on its left). The
/// four horizontal absolutes and `up`/`down` are accepted too, because a program may name a side
/// either way. `None` for a side or a facing this slice does not have.
#[must_use]
pub fn neighbour(pos: [i32; 3], facing: &str, side: &str) -> Option<([i32; 3], [i32; 3])> {
    let normal = normal(facing, side)?;
    Some((
        [pos[0] + normal[0], pos[1] + normal[1], pos[2] + normal[2]],
        normal,
    ))
}

/// The direction vector `side` names on a computer facing `facing` (`neighbour`'s second half).
#[must_use]
pub fn normal(facing: &str, side: &str) -> Option<[i32; 3]> {
    // The computer looks out of its screen; `front` is that face.
    let front = match facing {
        "north" => [0, 0, -1],
        "south" => [0, 0, 1],
        "west" => [-1, 0, 0],
        "east" => [1, 0, 0],
        _ => return None,
    };
    let left = match front {
        // Looking along `front` with `up` as up, "left" is `front` turned a quarter to the left:
        // north -> west, west -> south, south -> east, east -> north.
        [0, 0, -1] => [-1, 0, 0],
        [-1, 0, 0] => [0, 0, 1],
        [0, 0, 1] => [1, 0, 0],
        _ => [0, 0, -1],
    };
    Some(match side {
        "front" => front,
        "back" => opposite(front),
        "left" => left,
        "right" => opposite(left),
        "top" | "up" => [0, 1, 0],
        "bottom" | "down" => [0, -1, 0],
        "north" => [0, 0, -1],
        "south" => [0, 0, 1],
        "west" => [-1, 0, 0],
        "east" => [1, 0, 0],
        _ => return None,
    })
}

/// The direction opposite `normal`.
fn opposite(normal: [i32; 3]) -> [i32; 3] {
    [-normal[0], -normal[1], -normal[2]]
}

/// The probe block the in-game leg uses as a neighbour: upstream has no such block, and a
/// peripheral path cannot be proved without one (monitors and modems are later slices), so this
/// package ships a block whose only behaviour is to answer [`PERIPHERAL`]. It is not craftable and
/// not in the creative tab: the leg places it with `/setblock`.
pub const HARNESS_BLOCK: &str = "computercraft:peripheral_harness";

/// The type name the probe block answers (`peripheral.getType`).
pub const HARNESS_TYPE: &str = "computercraft:harness";

/// The probe peripheral: `echo` answers its first argument and `add` sums two numbers, so the
/// in-game leg can show a string and a number crossing the boundary in both directions.
pub struct Harness;

impl Peripheral for Harness {
    fn peripheral_type(&self) -> String {
        HARNESS_TYPE.to_owned()
    }

    fn methods(&self) -> Vec<String> {
        vec!["add".to_owned(), "echo".to_owned()]
    }

    fn call(
        &self,
        _host: &mut dyn ServerHost,
        method: &str,
        arguments: &[LuaValue],
    ) -> Option<Result<LuaValue, String>> {
        self.answer(method, arguments)
    }
}

impl Harness {
    /// The probe's answers, without the host: it touches no world, which is what makes them
    /// unit-testable (`call` is the same function with a host it ignores).
    fn answer(&self, method: &str, arguments: &[LuaValue]) -> Option<Result<LuaValue, String>> {
        match method {
            "echo" => Some(match arguments.first() {
                Some(value) => Ok(value.clone()),
                None => Err("echo expects one argument".to_owned()),
            }),
            "add" => Some(match (arguments.first(), arguments.get(1)) {
                (Some(LuaValue::Number(left)), Some(LuaValue::Number(right))) => {
                    Ok(LuaValue::Number(left + right))
                }
                _ => Err("add expects two numbers".to_owned()),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_key_is_this_package_namespace() {
        assert_eq!(PERIPHERAL.name, KEY_NAME);
        assert_eq!(KEY_NAME, "computercraft:peripheral");
    }

    #[test]
    fn each_side_reaches_the_neighbour_the_computer_names() {
        // A computer facing north: its terminal faces north, so looking out of the screen its left
        // is west and its right is east. Every case is a distinct position, so a table that mixed
        // two sides up would answer a neighbour that is not there.
        let at = [10, 64, 10];
        assert_eq!(
            neighbour(at, "north", "front"),
            Some(([10, 64, 9], [0, 0, -1]))
        );
        assert_eq!(
            neighbour(at, "north", "back"),
            Some(([10, 64, 11], [0, 0, 1]))
        );
        assert_eq!(
            neighbour(at, "north", "left"),
            Some(([9, 64, 10], [-1, 0, 0]))
        );
        assert_eq!(
            neighbour(at, "north", "right"),
            Some(([11, 64, 10], [1, 0, 0]))
        );
        assert_eq!(
            neighbour(at, "north", "top"),
            Some(([10, 65, 10], [0, 1, 0]))
        );
        assert_eq!(
            neighbour(at, "north", "bottom"),
            Some(([10, 63, 10], [0, -1, 0]))
        );
    }

    #[test]
    fn the_relative_sides_turn_with_the_computer() {
        // The same six names on a computer facing east: front is east, left is north, right is
        // south. A table that ignored `facing` would answer north's neighbours here.
        let at = [0, 0, 0];
        assert_eq!(neighbour(at, "east", "front"), Some(([1, 0, 0], [1, 0, 0])));
        assert_eq!(
            neighbour(at, "east", "left"),
            Some(([0, 0, -1], [0, 0, -1]))
        );
        assert_eq!(neighbour(at, "east", "right"), Some(([0, 0, 1], [0, 0, 1])));
        // Facing south, left is east; facing west, left is south.
        assert_eq!(normal("south", "left"), Some([1, 0, 0]));
        assert_eq!(normal("west", "left"), Some([0, 0, 1]));
    }

    #[test]
    fn the_absolute_names_are_accepted_as_well() {
        // `peripheral.wrap` takes either spelling upstream, so a program may say `north` where it
        // said `left`; both must reach the same block on a north-facing computer.
        assert_eq!(normal("north", "north"), normal("north", "front"));
        assert_eq!(normal("north", "up"), normal("north", "top"));
        assert_eq!(normal("north", "down"), normal("north", "bottom"));
        assert_eq!(normal("east", "south"), normal("east", "right"));
    }

    #[test]
    fn a_side_or_a_facing_this_slice_does_not_have_reaches_nothing() {
        let at = [0, 0, 0];
        for side in ["sideways", "", "FRONT", "north_east"] {
            assert_eq!(neighbour(at, "north", side), None, "{side}");
        }
        for facing in ["up", "", "North"] {
            assert_eq!(neighbour(at, facing, "front"), None, "{facing}");
        }
    }

    #[test]
    fn every_side_reaches_a_different_neighbour() {
        // Six sides, six distinct positions on one computer: a collision would make two names
        // answer for one block, which is how a monitor and a modem would shadow each other.
        let at = [3, 4, 5];
        let mut reached: Vec<[i32; 3]> = SIDES
            .iter()
            .map(|side| neighbour(at, "north", side).expect("a side").0)
            .collect();
        reached.sort_unstable();
        reached.dedup();
        assert_eq!(reached.len(), SIDES.len());
    }

    #[test]
    fn the_probe_answers_its_type_and_its_methods() {
        let harness = Harness;
        assert_eq!(harness.peripheral_type(), "computercraft:harness");
        assert_eq!(harness.methods(), ["add", "echo"]);
        assert_eq!(HARNESS_BLOCK, "computercraft:peripheral_harness");
    }

    #[test]
    fn the_probe_round_trips_a_string_and_a_number() {
        let harness = Harness;
        assert_eq!(
            harness.answer("echo", &[LuaValue::Str("hi".to_owned())]),
            Some(Ok(LuaValue::Str("hi".to_owned())))
        );
        assert_eq!(
            harness.answer("add", &[LuaValue::Number(2.0), LuaValue::Number(3.0)]),
            Some(Ok(LuaValue::Number(5.0)))
        );
    }

    #[test]
    fn the_probe_refuses_a_method_it_does_not_have_and_arguments_it_cannot_take() {
        let harness = Harness;
        assert_eq!(harness.answer("explode", &[]), None);
        assert_eq!(
            harness.answer("echo", &[]),
            Some(Err("echo expects one argument".to_owned()))
        );
        assert_eq!(
            harness.answer("add", &[LuaValue::Str("2".to_owned())]),
            Some(Err("add expects two numbers".to_owned()))
        );
    }
}
