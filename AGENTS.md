# AGENTS.md

`no_std` bare-metal RMK firmware for the ZSA Planck EZ (STM32F303CC-compatible
GD32F303CC), base and Glow variants. RMK 0.9 comes from crates.io — there is no
local rmk fork; do not add a path patch.

## Build & verify

Prerequisites: `nix develop` provides the whole toolchain (Rust with the
`thumbv7em-none-eabihf` target, flip-link, cargo-binutils, probe-rs,
dfu-util). Without Nix: `rustup target add thumbv7em-none-eabihf`, `cargo
install flip-link` (set as the linker in `.cargo/config.toml`; linking fails
without it), `rustup component add llvm-tools` + `cargo install cargo-binutils`
for `rust-objcopy`, and `dfu-util` for flashing. The flake must stay tracked by
git (`git add flake.nix flake.lock`) or `nix develop` will not see it.

```sh
cargo build --release                                              # base
cargo build --release --bin rmk-zsa-planck-ez-glow --features glow
cargo build --release --bin rmk-zsa-planck-ez      --features felschr-keymap
cargo build --release --bin rmk-zsa-planck-ez-glow --features glow,felschr-keymap
```

- `.cargo/config.toml` already sets `build.target = thumbv7em-none-eabihf`, so
  `--target` is optional. `cargo run` uses probe-rs over SWD
  (`STM32F303CCTx`); `flash.sh` uses DFU.
- There are no tests and no CI; both bins are `test = false`. A successful build
  is the only automated check — behavior requires hardware.
- `Makefile.toml` is a stale template leftover (references a nonexistent
  `rmk.hex` bin); use the README/`flash.sh` commands instead.

## Features

`default = []`. `glow` gates the glow bin (`required-features`).
`felschr-keymap` swaps the ZSA default keymap for the Oryx port. RMK's `storage`
and `watchdog` features are always enabled; keymap/Vial persistence uses the
last two 2 KB flash sectors reserved in `memory.x`.

## Source layout

- `src/main.rs` and `src/glow.rs` are separate crate roots (`[[bin]]`), each with
  its own `mod` list. Any new shared module must be declared in both.
- Keymaps are matrix-indexed `[8][6]` arrays, not the physical 4x12 grid: rows
  0-3 are the left half, rows 4-7 the right. The bottom row is scrambled per
  `vial.json`; cell `(3,5)` is unused and the 2u space is `(7,5)`.
- `keymap.rs` = ZSA default (4 layers), `keymap_felschr.rs` = Oryx port (5
  layers, layer 4 = NAV). The `felschr-keymap` feature selects it, and
  `NUM_LAYER` follows.
- `Action::User(0)` = stock LED_LEVEL key (only the Glow `rgb.rs` task acts on
  it). `Action::User(1)` = STM32 ROM-DFU jump, handled by base `teeth.rs` and
  Glow `rgb.rs`.
- Glow init is order-sensitive: IS31FL3737 blocking I2C bring-up runs in `main`
  before USB, and the raw-register PA12/USB-clock "warm-boot disconnect" is held
  until keymap/storage init finishes. Reordering breaks USB enumeration after a
  DFU jump.
- Base "teeth" LEDs are plain GPIO; Glow drives them on TIM4 PWM with
  CIE-corrected LED_LEVEL brightness.

## Generated / config files

- `build.rs` compresses `vial.json` into `VIAL_KEYBOARD_DEF` and hardcodes
  `VIAL_KEYBOARD_ID`; changing the ID makes Vial treat the board as a new
  keyboard. `src/vial.rs` only includes the generated file from `OUT_DIR`.
- `keyboard.toml` is not keymap config: it only sizes RMK's compile-time event
  channels (loaded via `KEYBOARD_TOML_PATH`). Bump the matching `subs` when
  adding an `EventSubscriber` (currently layer_change 2, action 1).
- `build.rs` copies `memory.x` into `OUT_DIR` and adds it to the linker search
  path. `memory.x` reserves the last 4 KB of flash for RMK storage
  (`StorageConfig::default()` uses two 2 KB sectors); do not remove that
  reservation or re-enable embassy-stm32's `memory-x` feature — storage init
  would erase the running firmware.
- `.opencode/plan` is an untracked, stale design doc from the initial port
  (pre-rename, mentions a local rmk fork, `board.rs`, `rmk-palettefx`). Trust the
  code and README instead.

## Flashing

`./flash.sh [base|glow] [features]` builds, objcopies, and runs `dfu-util`
(`0483:df11`, `0x08000000:leave`). `features` is a comma-separated list of full
cargo feature names; `glow` is added automatically for the Glow variant. Enter
DFU by holding Esc while plugging in, or via the reset button under the
spacebar; the ROM bootloader cannot be bricked.

## Conventions

Commit messages follow Conventional Commits (see `git log`). Keymap tables are
`#[rustfmt::skip]`; preserve their matrix-row formatting when editing.
