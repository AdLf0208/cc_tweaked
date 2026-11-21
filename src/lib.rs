//! CC: Tweaked for Horizon — the computer block family, its block entity, the terminal it draws
//! and the four `/computercraft` usages.
//!
//! CC: Tweaked's computer is a Lua 5.2 machine (`data/computercraft/lua/bios.lua` and a ROM
//! of 104 files). The port runs a documented **subset**, kept in [`runtime`] with its tests;
//! [`commands`] owns the command surface and the JSON readout, [`family`] the three blocks that
//! are computers, [`screen`] the geometry of the drawn terminal. This file is the glue: one
//! `BlockEntityPart` for the computers and one `impl Hooks` for the registration and the
//! commands.
//!
//! What each part of the old bundle became, and what the pinned API cannot express, is in
//! `notes.md` §"Migration to the SDK format".
use mod_api::{
    BlockEntityEvent, BlockEntityPart, BlockEntitySubmitEvent, BlockEntityTickEvent,
    BlockEntityTicksEvent, BlockEntityUpdateTagEvent, BlockPart, BlockState, Capability,
    ClientHost, Command, CommandArgumentValue, Compound, Direction, Flow, Hooks, LevelHost,
    LevelRef, LevelSide, Registrar, RegistrationContext, RunModCommandEvent, ServerHost,
    StateForPlacementEvent, Tag, Text,
};

pub mod commands;
pub mod family;
pub mod peripheral;
pub mod runtime;
pub mod screen;

/// The computer's block entity's type, declared in code: the client sees this id, as it did
/// upstream. One type covers the whole family (`family::PATHS`).
pub const ENTITY: &str = "computercraft:computer";
/// The program a fresh computer runs: there is no ROM in this slice, so it reports the
/// version the ROM's `os.version()` answers and how to load one
/// (`data/computercraft/lua/bios.lua:38-40`).
const DEFAULT_PROGRAM: &str =
    "print(\"CraftOS 1.9\") print(\"no ROM: see\") print(\"/computercraft\")";
/// The terminal's ink: vanilla's sign-text grey, 0xRRGGBB (the API's text colour).
const TERMINAL_INK: u32 = 0x00d8_d8d8;

/// What a computer keeps in its saved compound: the program, the terminal it printed and
/// the two flags the tick and the commands act on.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Computer {
    program: String,
    buffer: Vec<String>,
    running: bool,
    powered: bool,
}

impl Computer {
    /// A freshly placed computer: the boot program, an empty terminal, powered on.
    fn fresh() -> Self {
        Self {
            program: DEFAULT_PROGRAM.to_owned(),
            buffer: Vec::new(),
            running: false,
            powered: true,
        }
    }

    /// The computer in `nbt`, with the defaults for anything a saved computer lacks (an
    /// older save, or a client's copy before the first tag arrives).
    fn read(nbt: &Compound) -> Self {
        let program = match nbt.get(&"Program".into()) {
            Some(Tag::String(text)) => text.to_string_lossy(),
            _ => DEFAULT_PROGRAM.to_owned(),
        };
        let buffer = match nbt.get(&"Buffer".into()) {
            Some(Tag::List(entries)) => entries
                .iter()
                .filter_map(|entry| match entry {
                    Tag::String(text) => Some(text.to_string_lossy()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        let flag = |name: &str, default: bool| match nbt.get(&name.into()) {
            Some(Tag::Byte(value)) => *value != 0,
            _ => default,
        };
        Self {
            program,
            buffer,
            running: flag("Running", false),
            powered: flag("Powered", true),
        }
    }

    /// Writes the whole computer into `nbt`, the saved compound the tick and the commands
    /// read back.
    fn write(&self, nbt: &mut Compound) {
        nbt.insert("Program".into(), Tag::String(self.program.as_str().into()));
        nbt.insert(
            "Buffer".into(),
            Tag::List(
                self.buffer
                    .iter()
                    .map(|line| Tag::String(line.as_str().into()))
                    .collect(),
            ),
        );
        nbt.insert("Running".into(), Tag::Byte(i8::from(self.running)));
        nbt.insert("Powered".into(), Tag::Byte(i8::from(self.powered)));
    }

    /// What clients get of the computer (`BlockEntity.getUpdateTag`): the terminal and the
    /// power flag, so the screen draws what the server ran.
    fn client_tag(&self) -> Compound {
        let mut tag = Compound::new();
        tag.insert(
            "Buffer".into(),
            Tag::List(
                self.buffer
                    .iter()
                    .map(|line| Tag::String(line.as_str().into()))
                    .collect(),
            ),
        );
        tag.insert("Powered".into(), Tag::Byte(i8::from(self.powered)));
        tag
    }
}

/// The computer's block entity: created for `computercraft:computer_normal`, ticked by the
/// server, sent to clients as its terminal, and drawn on its front face.
struct ComputerPart;

#[mod_api::horizon_part]
impl BlockPart for ComputerPart {
    /// `AbstractComputerBlock.getStateForPlacement`: the placed computer faces the player.
    /// A block entity Part is registered as a block Part for its blocks too, which is where
    /// a block moment like this one is asked.
    fn state_for_placement(
        &self,
        host: &dyn LevelHost,
        event: &mut StateForPlacementEvent<'_>,
    ) -> Option<Option<BlockState>> {
        if host
            .state_block(event.state)
            .is_none_or(|block| !family::is_computer(block))
        {
            return None;
        }
        let facing = runtime::facing_for_placement(event.context.yaw);
        let placed = host.state_with_property(event.state, "facing", facing)?;
        Some(Some(placed))
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for ComputerPart {
    /// `EntityBlock.newBlockEntity`: a computer block gets this port's entity, holding the
    /// boot program. On a client this creates the copy the server's tag is merged into.
    fn block_entity(
        &self,
        _level: LevelSide,
        _event: &mut BlockEntityEvent<'_>,
    ) -> Option<Compound> {
        let mut nbt = Compound::new();
        Computer::fresh().write(&mut nbt);
        Some(nbt)
    }

    /// `EntityBlock.getTicker`: the computer ticks.
    fn block_entity_ticks(&self, _host: &dyn mod_api::Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: a powered computer that has not run its program yet runs it now —
    /// with its neighbours answering `peripheral.*` — keeps the terminal, lights `state=on` and
    /// pushes both to clients.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let mut computer = Computer::read(&event.entity.nbt);
        if !computer.powered || computer.running {
            return;
        }
        let mut terminal = runtime::Terminal::new(runtime::TERM_WIDTH, runtime::TERM_LINES);
        let outcome = match in_place_level(host) {
            Some(level) => {
                let facing = facing_at(host, level, event.pos);
                let mut world = Lookup {
                    host: &mut *host,
                    level,
                    pos: event.pos,
                    facing,
                };
                runtime::run_with(&computer.program, &mut terminal, &mut world)
            }
            // The level is not in place (a parked level): the program still runs, and every side
            // then reaches no peripheral, which is what the lookup answers without a level.
            None => runtime::run(&computer.program, &mut terminal),
        };
        if let Err(error) = outcome {
            terminal.write_line(&format!("error: {error}"));
        }
        computer.buffer = terminal.lines().to_vec();
        computer.running = true;
        computer.write(&mut event.entity.nbt);
        let _ = host.set_block_property(event.pos, "state", "on");
        // `Level.sendBlockUpdated`: a client's copy of the block entity only gets a new tag
        // when its block changes, so the terminal is pushed explicitly.
        host.block_entity_sync(event.pos);
    }

    /// `BlockEntity.getUpdateTag`: clients get the terminal, so the screen draws what the
    /// server ran.
    fn block_entity_update_tag(
        &self,
        event: &mut BlockEntityUpdateTagEvent<'_>,
    ) -> Option<Compound> {
        Some(Computer::read(&event.entity.nbt).client_tag())
    }

    /// `BlockEntityRenderer.submit`: the terminal's lines, drawn on the block's `facing`
    /// face with the font atlas.
    fn block_entity_submit(&self, host: &dyn ClientHost, event: &mut BlockEntitySubmitEvent<'_>) {
        let lines = Computer::read(&event.entity.nbt).buffer;
        if lines.is_empty() {
            return;
        }
        // The API reads a *position's* state property (`level_block_property`), not a
        // `BlockState`'s, so the state's own string is parsed (a tested helper:
        // `screen::facing_of`).
        let state = host.block_state_string(event.state);
        let facing = state
            .as_deref()
            .and_then(screen::facing_of)
            .unwrap_or("north");
        let (origin, degrees) = screen::pose(facing);
        event.draw.push();
        event.draw.translate(origin);
        event
            .draw
            .rotate_about([0.0, 1.0, 0.0], degrees, [0.0, 0.0, 0.0]);
        event
            .draw
            .scale([screen::FONT_SCALE, -screen::FONT_SCALE, screen::FONT_SCALE]);
        for (index, line) in lines.iter().enumerate() {
            let _ = event.draw.text(
                &Text::literal(line.as_str()),
                screen::line_origin(index),
                TERMINAL_INK,
                true,
                None,
            );
        }
        event.draw.pop();
    }
}

/// The mod: no settings, only the two hooks and the Part's registration.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Once, at startup: declare the computers' block entity type over the whole family, attach the
    /// Part to it, and let the probe block answer the peripheral key.
    fn register(&self, r: &mut dyn Registrar, _context: &RegistrationContext) {
        let kind = r
            .block_entity_type("computer")
            .blocks(&family::PATHS)
            .build();
        r.block_entity_part(&kind, ComputerPart)
            .expect("computercraft: the computer Part attaches to the type declared beside it");
        let harness = r.block_key(peripheral::HARNESS_BLOCK);
        r.block_capability(
            &peripheral::PERIPHERAL,
            &harness,
            |_host, _at, _state, _entity| Some(Box::new(peripheral::Harness)),
        );
    }

    /// The four usages. `load`'s program is its last, greedy argument, so it can hold spaces
    /// (a chat line cannot hold a newline: separate statements with `;`).
    fn register_commands(
        &self,
        _host: &mut dyn ServerHost,
        event: &mut mod_api::RegisterCommandsEvent<'_>,
    ) {
        for (usage, level) in commands::usages() {
            event.commands.push(Command::op(usage, level));
        }
    }

    /// A player ran one of our usages: the server parsed it, so the arguments are typed and
    /// the permission check is done. Answer it and take it.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        let invocation = event.command;
        if invocation.literals.first().map(String::as_str) != Some("computercraft") {
            return Flow::Continue;
        }
        let sub = invocation.literals.get(1).map_or("", String::as_str);
        let program = match invocation.argument("program") {
            Some(CommandArgumentValue::String(text)) => Some(text.as_str()),
            _ => None,
        };
        let switch = match invocation.argument("switch") {
            Some(CommandArgumentValue::String(word)) => Some(word.as_str()),
            _ => None,
        };
        let asked = commands::asked(sub, program, switch);
        let action = match asked {
            commands::Asked::Do(action) => action,
            commands::Asked::Refused(hint) => {
                let _ = host.commands().feedback(Text::literal(hint));
                return Flow::Cancel;
            }
            commands::Asked::Unknown => {
                let _ = host.commands().feedback(Text::literal(format!(
                    "Unknown usage: /computercraft {sub}"
                )));
                return Flow::Cancel;
            }
        };
        let Some(at) = invocation.argument("pos").and_then(|value| match value {
            CommandArgumentValue::BlockPos(at) => Some(*at),
            _ => None,
        }) else {
            let _ = host
                .commands()
                .feedback(Text::literal("Usage: /computercraft <usage> <x> <y> <z>"));
            return Flow::Cancel;
        };
        act(host, invocation.dimension, at, action);
        Flow::Cancel
    }
}

/// Runs one answered command against the computer at `at` in `dimension`, answering the
/// player through the command's own feedback channel.
fn act(
    host: &mut dyn ServerHost,
    dimension: mod_api::DimensionKey,
    at: [i32; 3],
    action: commands::Action,
) {
    let Some((block, entity)) = computer_at(host, dimension, at) else {
        let _ = host.commands().feedback(Text::literal(format!(
            "No computer at {},{},{}.",
            at[0], at[1], at[2]
        )));
        return;
    };
    let Some(data) = entity.data(host) else {
        let _ = host
            .commands()
            .feedback(Text::literal("That computer could not be read."));
        return;
    };
    let mut computer = Computer::read(&data);
    let answer = match &action {
        commands::Action::Screen => commands::screen_json(
            &block,
            at,
            computer.powered,
            computer.running,
            &computer.buffer,
        ),
        commands::Action::Load { program } => {
            computer.program.clone_from(program);
            computer.buffer.clear();
            computer.running = false;
            "Program loaded; it runs on the next tick.".to_owned()
        }
        commands::Action::Run => {
            computer.buffer.clear();
            computer.running = false;
            "Running again.".to_owned()
        }
        commands::Action::RunProgram { program } => {
            computer.program.clone_from(program);
            computer.buffer.clear();
            computer.running = false;
            "Program loaded and running; the terminal updates on the next tick.".to_owned()
        }
        commands::Action::Power { on } => {
            computer.powered = *on;
            computer.buffer.clear();
            computer.running = false;
            if *on {
                "Computer powered on.".to_owned()
            } else {
                "Computer powered off.".to_owned()
            }
        }
    };
    if action != commands::Action::Screen {
        let mut saved = data;
        computer.write(&mut saved);
        // `merge_data` writes the compound and sends the block update, so the new terminal
        // reaches clients the way a changed block does upstream.
        let _ = entity.merge_data(host, saved);
    }
    if matches!(action, commands::Action::Power { .. }) {
        // Off while it boots: the tick that runs the program lights it again.
        let _ = host.set_block_property(at, "state", "off");
        host.block_entity_sync(at);
    }
    let _ = host.commands().feedback(Text::literal(answer));
}

/// The computer at `at`, with the block it is: a vanilla block entity or another mod's is
/// refused before its data is read.
fn computer_at(
    host: &mut dyn ServerHost,
    dimension: mod_api::DimensionKey,
    at: [i32; 3],
) -> Option<(String, mod_api::BlockEntityRef)> {
    let name = dimension.id(host)?;
    let level = host.level(&name)?;
    let block = level.block(host, at)?;
    let id = block.id(host).to_string();
    if !family::is_computer(&id) {
        return None;
    }
    let entity = host.block_entity(level, at)?;
    Some((id, entity))
}

// ---------------------------------------------------------------------------
// The peripheral lookup a running program performs
// ---------------------------------------------------------------------------

/// The level a computer is in, for a tick that is given no level: the one `in_place` names (the
/// levels the game is acting on, after parked ones), the same helper `bigreactors` and
/// `pneumaticcraft` use. A tick fires for one level, so the first level in place is that one.
fn in_place_level(host: &dyn ServerHost) -> Option<LevelRef> {
    host.levels()
        .list_levels()
        .into_iter()
        .find(|level| level.in_place(host))
}

/// The `facing` of the computer at `pos`, read from its block state: the API reads a *position's*
/// property, and this glue has the state string (`screen::facing_of` is the tested parser).
fn facing_at(host: &dyn ServerHost, level: LevelRef, pos: [i32; 3]) -> String {
    level
        .block_state(host, pos)
        .and_then(|state| host.block_state_string(state))
        .as_deref()
        .and_then(screen::facing_of)
        .unwrap_or("north")
        .to_owned()
}

/// The [`Direction`] a normal is, for the capability lookup's `side`.
fn direction_of(normal: [i32; 3]) -> Option<Direction> {
    Direction::ALL
        .into_iter()
        .find(|direction| direction.normal() == normal)
}

/// The world a running program sees: the six blocks beside the computer, asked for
/// `computercraft:peripheral`.
struct Lookup<'a> {
    host: &'a mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    facing: String,
}

impl Lookup<'_> {
    /// The peripheral `side` reaches, asked through the face of it the computer touches (a
    /// neighbour is reached from the side that looks back at us, as a pipe asks a tank).
    fn peripheral(&mut self, side: &str) -> Option<Capability<dyn peripheral::Peripheral>> {
        let (at, normal) = peripheral::neighbour(self.pos, &self.facing, side)?;
        let direction = direction_of(normal)?;
        self.level.capability(
            &mut *self.host,
            &peripheral::PERIPHERAL,
            at,
            Some(direction.opposite()),
        )
    }
}

impl runtime::World for Lookup<'_> {
    fn peripheral_type(&mut self, side: &str) -> Option<String> {
        self.peripheral(side).map(|found| found.peripheral_type())
    }

    fn peripheral_methods(&mut self, side: &str) -> Option<Vec<String>> {
        self.peripheral(side).map(|found| found.methods())
    }

    fn peripheral_call(
        &mut self,
        side: &str,
        method: &str,
        arguments: &[runtime::LuaValue],
    ) -> Result<runtime::LuaValue, String> {
        let Some(found) = self.peripheral(side) else {
            return Ok(runtime::LuaValue::Nil);
        };
        found
            .call(&mut *self.host, method, arguments)
            .unwrap_or_else(|| {
                Err(format!(
                    "attempt to call a nil value (method '{method}' of {})",
                    found.peripheral_type()
                ))
            })
    }
}
