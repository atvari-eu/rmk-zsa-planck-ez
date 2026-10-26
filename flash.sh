#!/usr/bin/env sh
# Flash a built RMK firmware to the Planck EZ (or Glow) over STM32 DFU.
#
# Board must be in bootloader mode (physical reset button, or boot while
# holding Esc under ZSA/QMK firmware); it then enumerates as 0483:df11.
#
# Runnable from anywhere; operates on the zsa-planck-ez directory it lives in.
#
# Usage: ./flash.sh                    # base variant, default keymap
#   ./flash.sh glow                    # Glow variant
#
# The 2nd argument is passed to cargo as --features verbatim (comma-separated
# full feature names). The glow feature is added automatically for the glow bin.
#
# Requires dfu-util (e.g. nix run nixpkgs#dfu-util -- ...) and rust-objcopy
# (or run inside `nix develop`).

set -eu
cd "$(dirname "$0")"

variant=${1:-base}
case "$variant" in
base) bin="rmk-zsa-planck-ez" ;;
glow) bin="rmk-zsa-planck-ez-glow" ;;
*) echo "unknown variant '$variant' (use base or glow)" >&2; exit 1 ;;
esac

target="target/thumbv7em-none-eabihf/release/$bin"

features=${2:-}
[ "$variant" = glow ] && features="$features${features:+,}glow"

cargo build --release --target thumbv7em-none-eabihf --bin "$bin" ${features:+--features "$features"}
[ -f "$target" ] || { echo "missing $target (build failed?)" >&2; exit 1; }
rust-objcopy -O binary "$target" "$target.bin"

exec dfu-util -a 0 -d 0483:df11 -s 0x08000000:leave -D "$target.bin"
