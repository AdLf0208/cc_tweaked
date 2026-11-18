#!/usr/bin/env bash
# In-game check for CC: Tweaked (SDK format, slice 1).
#
# The claim: a placed `computercraft:computer_normal` runs a program and its
# terminal is observable. The readout is the port's own command,
# `/computercraft screen <x> <y> <z>`, which answers one line of JSON with the
# terminal's lines (the block also *draws* them on its front face, but a
# headless run has no renderer: `tools/visual.sh` is the screenshot leg).
#
# Since the SDK migration (2026-11-18) the code mod is one flat `.hmod` the
# launcher compiles into the game, so GAME must name a game the launcher built
# with it:
#   horizon-launcher native-build ports/computercraft --instance <instance> --trust-native-code
# and then `GAME=<that horizon-client> tools/e2e.sh`. There is no `--dev-mod`
# route any more: a content-only dev mod cannot carry the block entity, the
# tick, the screen or the commands. Without a launcher-built game the script
# stops with "missing horizon-client" rather than running an unmodified game,
# which is the intended behaviour (that is how imfast's leg reports too).
#
# The control binary is the shared one at the campaign pin (protocol-compatible
# with the nightly the mod is built for; only *content* is unshared).
#
# Legs
#   default        the modded game: the scenario below, all assertions true
#   SELFTEST=1     the same setup with two assertions inverted (the boot
#                  terminal and `print(1+1)`): the run must fail, proving the
#                  check can fail
#
# The scenario, in The End on a stone floor (an empty level: no ambient items):
#   1. the block places with the state the setblock asked for, and the item exists
#   2. its default program boots and the terminal shows the two boot lines
#   3. `/computercraft load … print(1+1)` then `screen` gives "2"
#   4. a loop and a concatenation give "item 1|item 2|item 3"
#   5. the block's `state` reads `on` while it runs
#   6. CONTROL: `/computercraft power … off` empties the terminal and the same
#      program on the powered-off computer prints nothing; powering it back on
#      runs it again
#   7. CONTROL: `/computercraft screen` on a stone block says there is no computer
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
BLOCK=computercraft:computer_normal
X=2; Y=64; Z=2        # the computer
CX=5; CZ=2            # a stone block, the "no computer here" control

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -n "$GAME" ] || {
    echo "missing GAME: the SDK-format mod is compiled into the game by the launcher;" >&2
    echo "  horizon-launcher native-build $PORT --instance <instance> --trust-native-code" >&2
    echo "  then GAME=<that horizon-client> $0" >&2
    exit 1
}
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }
[ -f "$PORT/computercraft.hmod" ] || { echo "missing package: $PORT/computercraft.hmod" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/computercraft-home.XXXXXX)
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
fail() { echo "FAIL: $*" >&2; exit 1; }

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
    local world=${1:-$WORLD}
    rm -f "$SOCK"
    # No --dev-mod: the block, its block entity, the tick, the screen and the
    # commands are all compiled in from the one `.hmod`.
    "$CTL" --socket "$SOCK" launch --game "$GAME" --world "$world" --seed 20261104 \
        --creative --headless --wait --resources "$RES" >"$SCRATCH/game-console.log" 2>&1
    ctl wait-ready 120
    ctl wait 10
    ctl command "execute in minecraft:the_end run tp @s 0.5 64 0.5" >/dev/null
    ctl wait 5
    ctl command "execute in minecraft:the_end run fill -8 63 -8 8 63 8 minecraft:stone" >/dev/null
    ctl command "execute in minecraft:the_end run tp @s 0.5 64 0.5" >/dev/null
    ctl wait 10
    ctl command "kill @e[type=minecraft:enderman]" >/dev/null 2>&1 || true
    ctl wait 3
}

# The terminal the server keeps, as the port's command reports it.
screen() {
    ctl chat "/computercraft screen $1 $2 $3" >/dev/null
    ctl wait 2 >/dev/null
    ctl messages 4 | jq -r '.chat[0]'
}
screen_lines() { screen "$@" | jq -r '.lines | join("|")'; }
screen_field() { local field=$1; shift; screen "$@" | jq -r ".$field"; }
# `ctl block` answers `block` (the name) and `state` (name plus properties).
block_state() { ctl block "$1" "$2" "$3" | jq -r '.state // "unknown"'; }
# A command's own answer (the reply the mod gives through `Commands::feedback`).
reply() {
    ctl chat "$1" >/dev/null
    ctl wait 2 >/dev/null
    ctl messages 4 | jq -r '.chat[0]'
}
load_program() {
    ctl chat "/computercraft load $X $Y $Z $1" >/dev/null
    ctl wait 4 >/dev/null
}
expect_lines() {
    local expected=$1 what=$2 got
    got=$(screen_lines "$X" "$Y" "$Z")
    [ "$got" = "$expected" ] || fail "$what: expected [$expected], got [$got] ($(screen "$X" "$Y" "$Z"))"
    echo "  $what: $got"
}

echo "== launch (creative, The End arena, one SDK package) =="
launch

echo "== the item exists and the block places with the state asked for =="
ctl give "$BLOCK" 1 >/dev/null || fail "the package did not provide $BLOCK"
ctl wait 4 >/dev/null
ctl chat "/clear @s" >/dev/null
ctl command "execute in minecraft:the_end run setblock $X $Y $Z ${BLOCK}[facing=south,state=off]" >/dev/null \
    || fail "cannot place the computer"
ctl wait 4 >/dev/null
placed=$(block_state "$X" "$Y" "$Z")
case "$placed" in
    "$BLOCK[facing=south,state=on]"|"$BLOCK[facing=south,state=off]") echo "  placed: $placed" ;;
    *) fail "the computer did not place (got $placed)" ;;
esac
[ "$(block_state "$CX" "$Y" "$CZ")" = "minecraft:air" ] || fail "the control cell is not air"

echo "== its default program boots and the terminal shows it =="
expect_lines "CraftOS 1.9|no ROM: see|/computercraft" "boot terminal"
[ "$(screen_field running "$X" "$Y" "$Z")" = "true" ] || fail "the computer did not report that it ran"
[ "$(screen_field block "$X" "$Y" "$Z")" = "$BLOCK" ] || fail "the screen reply names another block"
if [ "$SELFTEST" = 1 ]; then
    echo "  selftest: asserting the boot terminal is something else (must FAIL)"
    [ "$(screen_lines "$X" "$Y" "$Z")" = "selftest" ] || fail "selftest: the boot terminal is not 'selftest'"
fi

echo "== print(1+1) shows 2 =="
load_program 'print(1+1)'
if [ "$SELFTEST" = 1 ]; then
    echo "  selftest: asserting print(1+1) answers 3 (must FAIL)"
    expect_lines "3" "selftest arithmetic"
else
    expect_lines "2" "print(1+1)"
fi

echo "== a loop and a concatenation =="
load_program 'local n=0 while n<3 do n=n+1 print("item " .. n) end'
expect_lines "item 1|item 2|item 3" "loop and concatenation"

echo "== the block is lit while it has run =="
lit=$(block_state "$X" "$Y" "$Z")
[ "$lit" = "$BLOCK[facing=south,state=on]" ] || fail "the running computer's state is $lit"
echo "  block state: $lit"

echo "== control: a powered-off computer runs nothing =="
ctl chat "/computercraft power $X $Y $Z off" >/dev/null
ctl wait 4 >/dev/null
[ "$(screen_field powered "$X" "$Y" "$Z")" = "false" ] || fail "the computer is not powered off"
[ "$(screen_lines "$X" "$Y" "$Z")" = "" ] || fail "powering off left a terminal: $(screen_lines "$X" "$Y" "$Z")"
before=$(block_state "$X" "$Y" "$Z")
[ "$before" = "$BLOCK[facing=south,state=off]" ] || fail "the powered-off computer's state is $before"
load_program 'print("broken")'
expect_lines "" "the powered-off computer printed nothing"
ctl wait 10 >/dev/null
expect_lines "" "still nothing ten ticks later"
echo "  powered off: $(screen "$X" "$Y" "$Z")"

echo "== control: a stone block is not a computer =="
ctl command "execute in minecraft:the_end run setblock $CX $Y $CZ minecraft:stone" >/dev/null
ctl wait 2 >/dev/null
answer=$(reply "/computercraft screen $CX $Y $CZ")
case "$answer" in
    "No computer at"*) echo "  $answer" ;;
    *) fail "a stone block answered the screen command: $answer" ;;
esac

echo "== powering it back on boots the loaded program =="
ctl chat "/computercraft power $X $Y $Z on" >/dev/null
ctl wait 6 >/dev/null
expect_lines "broken" "the loaded program ran after power-on"

echo "PASS: the computer placed, its default program booted to a readable terminal, print(1+1) answered 2, a loop and a concatenation answered item 1|item 2|item 3, the block read state=on while it ran, a powered-off computer ran nothing (its terminal stayed empty through a load and ten ticks), a stone block answered that there is no computer, and powering it back on ran the loaded program"
