#!/usr/bin/env bash
# In-game check for CC: Tweaked (SDK format, slice 3: the peripheral surface).
#
# The claim: the jar's three computers — `computer_normal`, `computer_advanced` and
# `computer_command` — are all real blocks in game; each carries its own terminal,
# each is craftable from its own recipe, each drops itself, and the port's readout
# names the block it actually read. And a placed computer **asks its neighbours**
# through `computercraft:peripheral`: `peripheral.call('right', 'getType')` on a block
# that answers the key prints its type, while the same call on a plain stone neighbour
# prints `nil`.
#
# The readout is the port's own command, `/computercraft screen <x> <y> <z>`, which
# answers one line of JSON with the block it read and the terminal's lines (the
# block also *draws* them on its front face; `tools/visual.sh` is the screenshot
# leg).
#
# Since the SDK migration the code mod is one flat `.hmod` the launcher compiles
# into the game, so GAME must name a launcher-built game:
#   ports/tools/launcher-game.sh build computercraft
#   eval "$(ports/tools/launcher-game.sh env computercraft)"
#   bash tools/e2e.sh
# Without one the script stops with "missing GAME" rather than running an
# unmodified game.
#
# Legs
#   default        the modded game: every phase below, all assertions true
#   SELFTEST=1     the peripheral assertion inverted (the harness side must print
#                  `nil`): the run must reach phase H and fail there, proving the
#                  slice's own assertion can fail
#   SELFTEST=2     the recipe phase inverted (`craft_family.py`)
#   SELFTEST=3     the advanced computer's terminal inverted (slice 2's pair)
#   GAME=$CONTROL_GAME   the same Nightly without the package: the first
#                  assertion (the item exists) fails, which is the differential
#                  that the ids come from this package
#
# The scenario, in The End on a stone floor (an empty level: no ambient items):
#   A  the three items exist and the three blocks place with the state asked for,
#      and `state=dancing` is refused where `state=blinking` is accepted
#   B  each computer boots its own terminal; a program loaded into the advanced
#      computer leaves the normal and command terminals unchanged (differential)
#   C  the readout names the block it read, for all three
#   D  all three recipes craft in a crafting table with the ingredients consumed,
#      and the advanced recipe with a stone in place of a gold ingot makes nothing
#   E  each block breaks and drops one of itself (the 26.3 loot dialect)
#   F  CONTROLS: a stone block is not a computer; a powered-off computer runs
#      nothing through a load and ten ticks; powering it back on runs the program
#   H  PERIPHERALS: a computer facing north calls `peripheral.call('right', …)` on
#      the harness block beside it and prints its type, its method names, an echoed
#      string and a summed number; the same calls on a plain stone neighbour, and
#      above the computer where there is air, print `nil` (the differential)
#   G  a computer placed by hand from two yaws faces the player (the block moment)
set -euo pipefail

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/computercraft
CTL=${CTL:-$CLONE/ports/_shared/bin/horizon-1d63e0a5/horizon-ctl}
# The launcher-built game; there is no per-port `horizon.py build` any more.
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/computercraft-e2e.sock}
SELFTEST=${SELFTEST:-0}
WORLD=${WORLD:-CCE2E}
NORMAL=computercraft:computer_normal
ADVANCED=computercraft:computer_advanced
COMMAND=computercraft:computer_command
# The port's peripheral probe (slice 3): the block that answers `computercraft:peripheral`.
HARNESS=computercraft:peripheral_harness
# The arena: the End's void, far from the central island (a fill at 0,0 lands inside it).
X0=200
FLOOR=$((X0 - 4))
X=$X0; Y=101; Z=0       # the normal computer
AX=203; AZ=0            # the advanced computer
CX=206; CZ=0            # the command computer
SX=209; SZ=0            # a stone block, the "no computer here" control
TABLE_X=200; TABLE_Z=4  # the crafting table
LOT_X=197; LOT_Z=0      # where a block is placed, broken and counted
LOT_PX=196              # where the player stands to break it

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -n "$GAME" ] || {
    echo "missing GAME: the SDK-format mod is compiled into the game by the launcher;" >&2
    echo "  ports/tools/launcher-game.sh build computercraft" >&2
    echo "  eval \"\$(ports/tools/launcher-game.sh env computercraft)\"; bash tools/e2e.sh" >&2
    exit 1
}
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }
command -v jq >/dev/null || { echo "missing jq (every assertion parses a ctl reply)" >&2; exit 1; }

# The game's scratch home. `launcher-game.sh env` exports `E2E_TMP` (under ~/.cache, never the
# 7.5 GB `/tmp` tmpfs, which filled mid-run on 2026-11-21 and made the game exit 3 with "No space
# left on device" before its socket); otherwise the campaign's usual `/tmp/opencode`.
SCRATCH_BASE=${E2E_TMP:-/tmp/opencode}
mkdir -p "$SCRATCH_BASE"
SCRATCH=$(mktemp -d "$SCRATCH_BASE/computercraft-home.XXXXXX")
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$SCRATCH/config"
rm -f "$SOCK"
# The game folder is the working directory: keep saves/options out of the port.
cd "$SCRATCH"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 40 >&2 || true; exit 1; }

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" 2>/dev/null || true
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() {
    local world=${1:-$WORLD} game=${2:-$GAME}
    rm -f "$SOCK"
    # No --dev-mod: the blocks, their block entity, the tick, the screen and the
    # commands are all compiled in from the one `.hmod`. `--offscreen`, not
    # `--headless`: the recipe phase opens a crafting-table menu and clicks its
    # slots, and a headless client never opens a screen.
    "$CTL" --socket "$SOCK" launch --game "$game" --world "$world" --seed 20261104 \
        --creative --offscreen --wait --resources "$RES" >"$SCRATCH/game-console.log" 2>&1
    ctl wait-ready 120 >/dev/null
    ctl wait 20 >/dev/null
    # Teleport first so the chunks the fills touch are loaded: a player over the void falls.
    ctl command "execute in minecraft:the_end run tp @s $X0.5 104 0.5" >/dev/null
    local loaded=false
    for _ in $(seq 1 60); do
        ctl wait 5 >/dev/null
        if [ "$(ctl block "$X0" 100 0 | jq -r '.loaded // false')" = true ]             || [ "$(ctl block "$X0" 100 0 | jq -r '.name // empty')" = "minecraft:end_stone" ]; then
            loaded=true
            break
        fi
    done
    ctl command "execute in minecraft:the_end run fill $FLOOR 100 -8 $((X0 + 14)) 100 8 minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run fill $FLOOR 101 -8 $((X0 + 14)) 108 8 minecraft:air" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $X0.5 101 0.5" >/dev/null
    ctl wait 10 >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null 2>&1 || true
    ctl wait 5 >/dev/null
    floor=$(block_state "$X0" 100 0)
    [ "$floor" = "minecraft:stone" ] || fail "the arena floor is not stone (got '$floor')"
    echo "  arena: stone floor at y=100, computers at y=$Y, loaded=$loaded"
}

# The terminal the server keeps, as the port's command reports it.
screen() { # screen X Z — the computers all sit at $Y
    ctl chat "/computercraft screen $1 $Y $2" >/dev/null
    ctl wait 2 >/dev/null
    ctl messages 4 | jq -r '.chat[0]'
}
screen_lines() { screen "$@" | jq -r '.lines | join("|")'; }
screen_field() { local field=$1; shift; screen "$@" | jq -r ".$field"; }
# `ctl block` answers `block` (the name) and `state` (name plus properties).
block_state() { ctl block "$1" "$2" "$3" | jq -r '.state // .name // "unknown"'; }
# A command's own answer (the reply the mod gives through `Commands::feedback`).
reply() {
    ctl chat "$1" >/dev/null
    ctl wait 2 >/dev/null
    ctl messages 4 | jq -r '.chat[0]'
}
# The newest N chat lines joined into one string: a vanilla command's refusal is two lines (the
# message, then a pointer to the bad argument), and `messages` answers newest first.
reply_lines() { # reply_lines N command
    ctl chat "$2" >/dev/null
    ctl wait 2 >/dev/null
    ctl messages 8 | jq -r --argjson n "$1" '.chat[0:$n] | join(" | ")'
}
load_program() { # load_program X Z program
    ctl chat "/computercraft load $1 $Y $2 $3" >/dev/null
    ctl wait 4 >/dev/null
}
expect_lines() { # expect_lines X Z expected what
    local x=$1 z=$2 expected=$3 what=$4 got
    got=$(screen_lines "$x" "$z")
    [ "$got" = "$expected" ] || fail "$what: expected [$expected], got [$got] ($(screen "$x" "$z"))"
    echo "  $what: $got"
}
# Place one computer of `block` at x/z, facing south, and read the state back.
place_computer() { # place_computer BLOCK X Z
    local block=$1 x=$2 z=$3 placed
    ctl give "$block" 1 >/dev/null || fail "the package did not provide $block"
    ctl command "execute in minecraft:the_end run setblock $x $Y $z ${block}[facing=south,state=off]" >/dev/null \
        || fail "cannot place $block"
    ctl wait 4 >/dev/null
    placed=$(block_state "$x" "$Y" "$z")
    case "$placed" in
        "$block[facing=south,state=on]"|"$block[facing=south,state=off]") echo "  placed $block: $placed" ;;
        *) fail "$block did not place (got $placed)" ;;
    esac
}
item_count() { ctl inventory | jq -r --arg b "$1" '.counts[$b] // 0'; }

echo "== the package under test =="
[ -f "$PORT/computercraft.hmod" ] || fail "missing package: $PORT/computercraft.hmod"
echo "  $(cd "$PORT" && sha256sum computercraft.hmod)"
echo "  $(stat -c '%s bytes' "$PORT/computercraft.hmod"), $(grep -m1 '^version' "$PORT/horizon-mod.toml")"
echo "  game: $GAME ($(sha256sum "$GAME" | cut -d' ' -f1))"

echo "== launch (creative, The End arena, one SDK package, offscreen) =="
launch

echo "== A: the three computers exist and place; the state enum is the jar's =="
ctl command "clear @s" >/dev/null
place_computer "$NORMAL" "$X" "$Z"
place_computer "$ADVANCED" "$AX" "$AZ"
place_computer "$COMMAND" "$CX" "$CZ"
# The blockstate's `state` enum: `blinking` is a value, `dancing` is not. Both writes go
# through the same command, so the acceptance and the refusal are a differential — and both
# answers are the game's own parsed reply, not the presence or absence of an error.
ok_reply=$(reply "/setblock $SX $Y $SZ ${NORMAL}[facing=south,state=blinking]")
case "$ok_reply" in
    "Changed the block"*) echo "  state=blinking accepted: $ok_reply" ;;
    *) fail "state=blinking was refused on $NORMAL: $ok_reply" ;;
esac
bad_reply=$(reply_lines 2 "/setblock $SX $Y $SZ ${NORMAL}[facing=south,state=dancing]")
case "$bad_reply" in
    *"does not accept 'dancing'"*) echo "  state=dancing refused: $bad_reply" ;;
    *) fail "state=dancing was not refused, so the state property is not the jar's: $bad_reply" ;;
esac
ctl command "execute in minecraft:the_end run setblock $SX $Y $SZ minecraft:air" >/dev/null

echo "== B: each computer boots its own terminal =="
expect_lines "$X" "$Z" "CraftOS 1.9|no ROM: see|/computercraft" "normal boot terminal"
expect_lines "$AX" "$AZ" "CraftOS 1.9|no ROM: see|/computercraft" "advanced boot terminal"
expect_lines "$CX" "$CZ" "CraftOS 1.9|no ROM: see|/computercraft" "command boot terminal"
for pair in "$X:$Z" "$AX:$AZ" "$CX:$CZ"; do
    [ "$(screen_field running "${pair%%:*}" "${pair##*:}")" = "true" ] \
        || fail "the computer at ${pair%%:*},${pair##*:} did not report that it ran"
done
if [ "$SELFTEST" = 3 ]; then
    echo "  selftest 3: asserting the advanced computer's terminal is 'selftest' (must FAIL)"
    expect_lines "$AX" "$AZ" "selftest" "selftest advanced terminal"
fi

echo "== B2: a program runs on one computer only (differential) =="
load_program "$AX" "$AZ" 'print(2+2)'
expect_lines "$AX" "$AZ" "4" "advanced after its own program"
expect_lines "$X" "$Z" "CraftOS 1.9|no ROM: see|/computercraft" "normal unchanged by the advanced load"
expect_lines "$CX" "$CZ" "CraftOS 1.9|no ROM: see|/computercraft" "command unchanged by the advanced load"

echo "== C: the readout names the block it read =="
readout_check() { # readout_check BLOCK X Z
    local block=$1 x=$2 z=$3 got
    got=$(screen_field block "$x" "$z")
    [ "$got" = "$block" ] || fail "the readout at $x,$z names '$got', expected '$block'"
    echo "  readout at $x,$z names $got"
}
readout_check "$NORMAL" "$X" "$Z"
readout_check "$ADVANCED" "$AX" "$AZ"
readout_check "$COMMAND" "$CX" "$CZ"
# And the address is the one asked for, not a neighbour's.
[ "$(screen "$AX" "$AZ" | jq -c '.pos')" = "[$AX,$Y,$AZ]" ] || fail "the advanced readout names another position"

echo "== D: all three recipes craft, and a wrong ingredient makes nothing =="
python3 "$PORT/tools/craft_family.py" "$SOCK" "$CTL" "$TABLE_X" "$Y" "$TABLE_Z" \
    || fail "the recipe phase failed"
echo "  recipes PASS"

echo "== E: each block drops one of itself (the 26.3 loot dialect) =="
loot_leg() { # loot_leg BLOCK [STATE] -> sets DROPPED
    local block=$1 state=${2-[facing=north,state=off]} placed
    ctl command "execute in minecraft:the_end run tp @s $LOT_PX.5 $Y 0.5" >/dev/null
    ctl command "clear @s" >/dev/null
    ctl command "gamemode survival" >/dev/null
    ctl wait 4 >/dev/null
    ctl command "execute in minecraft:the_end run setblock $LOT_X $Y $LOT_Z ${block}${state}" >/dev/null
    ctl wait 4 >/dev/null
    placed=$(block_state "$LOT_X" "$Y" "$LOT_Z")
    case "$placed" in
        "$block$state"|"$block[facing=north,state=on]"|"$block[facing=north,state=blinking]") ;;
        *) fail "the drop-test $block did not place (got $placed)" ;;
    esac
    ctl break "$LOT_X" "$Y" "$LOT_Z" >/dev/null
    ctl wait 30 >/dev/null
    [ "$(block_state "$LOT_X" "$Y" "$LOT_Z")" = "minecraft:air" ] \
        || fail "the $block at $LOT_X,$Y,$LOT_Z did not break"
    # The drop may still be lying there or already picked up: count both, and demand exactly one.
    on_ground=$(ctl entities 8 item \
        | jq --arg b "$block" '[.entities[] | select(.kind == "item" and .item.item == $b) | .item.count] | add // 0')
    held=$(item_count "$block")
    DROPPED=$((on_ground + held))
    echo "  $block: broke; $on_ground on the ground, $held picked up"
}
for block in "$NORMAL" "$ADVANCED" "$COMMAND" "$HARNESS"; do
    case "$block" in
        "$HARNESS") loot_leg "$block" "" ;;
        *) loot_leg "$block" ;;
    esac
    [ "$DROPPED" = 1 ] \
        || fail "mining $block dropped $DROPPED of itself, expected the one its 26.3 loot table names"
done
ctl command "gamemode creative" >/dev/null

echo "== F: controls =="
ctl command "execute in minecraft:the_end run setblock $SX $Y $SZ minecraft:stone" >/dev/null
ctl wait 2 >/dev/null
answer=$(reply_lines 2 "/computercraft screen $SX $Y $SZ")
case "$answer" in
    "No computer at"*) echo "  a stone block answers: $answer" ;;
    *) fail "a stone block answered the screen command: $answer" ;;
esac
# An id the jar has but this slice does not register is refused by the game.
if ctl give computercraft:monitor_normal 1 >/dev/null 2>&1; then
    fail "the game accepted computercraft:monitor_normal, which this slice does not register"
fi
ctl command "clear @s" >/dev/null
echo "  an unregistered id is refused"

echo "== F2: a powered-off computer runs nothing, and power-on runs the loaded program =="
ctl chat "/computercraft power $CX $Y $CZ off" >/dev/null
ctl wait 4 >/dev/null
[ "$(screen_field powered "$CX" "$CZ")" = "false" ] || fail "the command computer is not powered off"
[ "$(screen_lines "$CX" "$CZ")" = "" ] || fail "powering off left a terminal: $(screen_lines "$CX" "$CZ")"
load_program "$CX" "$CZ" 'print("broken")'
expect_lines "$CX" "$CZ" "" "the powered-off computer printed nothing"
ctl wait 10 >/dev/null
expect_lines "$CX" "$CZ" "" "still nothing ten ticks later"
ctl chat "/computercraft power $CX $Y $CZ on" >/dev/null
ctl wait 6 >/dev/null
expect_lines "$CX" "$CZ" "broken" "the loaded program ran after power-on"

echo "== H: the peripheral surface (a placed computer asks its neighbours) =="
# A computer facing north: its screen faces north, so looking out of it the computer's *right*
# — the name the program passes — is the east neighbour.
ctl command "execute in minecraft:the_end run setblock $X $Y $Z ${NORMAL}[facing=north,state=off]" >/dev/null
ctl command "execute in minecraft:the_end run setblock $((X + 1)) $Y $Z $HARNESS" >/dev/null
ctl command "execute in minecraft:the_end run setblock $((X - 1)) $Y $Z minecraft:stone" >/dev/null
ctl command "execute in minecraft:the_end run setblock $X $((Y + 1)) $Z minecraft:air" >/dev/null
ctl wait 6 >/dev/null
[ "$(block_state "$((X + 1))" "$Y" "$Z")" = "$HARNESS" ] \
    || fail "the harness block did not place at the computer's right (got '$(block_state "$((X + 1))" "$Y" "$Z")')"
[ "$(block_state "$((X - 1))" "$Y" "$Z")" = "minecraft:stone" ] \
    || fail "the stone control did not place at the computer's left (got '$(block_state "$((X - 1))" "$Y" "$Z")')"
echo "  neighbours: $HARNESS at right, minecraft:stone at left, air at top"

# The brief's own command: `peripheral.call('right', 'getType')` printed. The terminal is 16
# columns wide, so the type name wraps at its width (the port's one-block screen, not a monitor).
p1_expected="computercraft:ha|rness"
if [ "$SELFTEST" = 1 ]; then
    echo "  selftest: asserting the harness side answers nil (must FAIL)"
    p1_expected="nil"
fi
load_program "$X" "$Z" "print(peripheral.call('right','getType'))"
expect_lines "$X" "$Z" "$p1_expected" "peripheral.call('right','getType')"

load_program "$X" "$Z" "print(peripheral.getNames('right'))"
expect_lines "$X" "$Z" $'add\techo' "peripheral.getNames('right')"

# Values cross in both directions: a string out and back, and two numbers summed.
load_program "$X" "$Z" "print(peripheral.call('right','echo','hi'))"
expect_lines "$X" "$Z" "hi" "echo round-trips a string"
load_program "$X" "$Z" "print(peripheral.call('right','add',2,3))"
expect_lines "$X" "$Z" "5" "add sums two numbers"

# The differential: the left neighbour is a plain stone block, and it answers nothing. A call
# that answered `nil` because the side was empty would not prove the lookup read a block, so the
# stone is asserted to be there first (above) and read back again here.
load_program "$X" "$Z" "print(peripheral.call('left','getType'))"
expect_lines "$X" "$Z" "nil" "call on a stone neighbour"
load_program "$X" "$Z" "print(peripheral.getType('left'), peripheral.getNames('left'))"
expect_lines "$X" "$Z" $'nil\tnil' "getType and getNames on a stone neighbour"
# And a side that reaches air: the block above the computer is air, and answers nothing either,
# so the nil above is about the *block* that answers, not about the side's existence.
load_program "$X" "$Z" "print(peripheral.call('top','echo','x'))"
expect_lines "$X" "$Z" "nil" "call on the air above the computer"
# A method the harness does not have is the program's error, not a silent nil.
load_program "$X" "$Z" "print(peripheral.call('right','explode'))"
expect_lines "$X" "$Z" "error: line 1: a|ttempt to call a| nil value (meth|od 'explode' of |computercraft:ha|rness)" \
    "an unknown method is an error"

echo "== G: a computer placed by hand faces the player (the block moment) =="
# `state_for_placement` is a block moment, so `/setblock` never exercises it: the item has to be
# placed by a player. The expected facing is `runtime::facing_for_placement`'s own table (yaw 0 =
# looking south -> the block's front faces north, towards the player standing north of it).
hand_place() { # hand_place BLOCK BX BZ PX PZ YAW WANT
    local block=$1 bx=$2 bz=$3 px=$4 pz=$5 yaw=$6 want=$7 placed
    ctl command "execute in minecraft:the_end run setblock $bx $Y $bz minecraft:air" >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $px $Y $pz" >/dev/null \
        || fail "cannot teleport to $px,$Y,$pz"
    ctl look "$yaw" 0 >/dev/null || fail "cannot look yaw $yaw"
    ctl command "clear @s" >/dev/null
    ctl wait 3 >/dev/null
    ctl command "item replace entity @s hotbar.0 with $block 1" >/dev/null
    ctl wait 3 >/dev/null
    ctl place "$bx" "$Y" "$bz" --item "$block" >"$SCRATCH/place.log" 2>&1 \
        || fail "ctl place of $block at $bx,$Y,$bz failed: $(cat "$SCRATCH/place.log")"
    ctl wait 5 >/dev/null
    placed=$(block_state "$bx" "$Y" "$bz")
    case "$placed" in
        "$block[facing=$want,state=on]"|"$block[facing=$want,state=off]") echo "  $block hand-placed facing $want" ;;
        *) fail "a hand-placed $block is '$placed', expected facing=$want" ;;
    esac
}
hand_place "$ADVANCED" 200 -2 200.5 -3.5 0 north
hand_place "$COMMAND" 203 -2 204.5 -1.5 90 east

echo "PASS: all three computers (normal, advanced, command) exist and place with the jar's state property (blinking accepted, dancing refused), each boots its own terminal, a program loaded into the advanced one left the other two unchanged, the readout names the block and position it read, all three recipes crafted in a table with their ingredients consumed while the advanced pattern with a stone instead of gold made nothing, each block broke and dropped exactly one of itself, a stone block answered that there is no computer, an unregistered id was refused, a powered-off computer ran nothing through a load and ten ticks before power-on ran the loaded program, a computer facing north called peripheral.call('right','getType'), getNames('right'), echo and add on the harness block beside it through computercraft:peripheral and printed computercraft:harness, add echo, hi and 5 while the same calls on a plain stone neighbour and on the air above printed nil and a method the harness does not have raised an error, and a computer placed by hand from two different yaws faced the player both times"
