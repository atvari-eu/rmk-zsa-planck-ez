# rmk-zsa-planck-ez

RMK firmware for the [ZSA Planck EZ](https://www.zsa.io/planck) keyboard
(base and Glow variants).

The Planck EZ ships an STM32F303CC-compatible MCU (GD32F303CC). The firmware
targets that chip directly using [RMK](https://rmk.rs) and the
[Embassy](https://embassy.dev) async embedded Rust ecosystem, replacing ZSA's
stock QMK-based firmware.

## Features

- **Live keymap editing** via [Vial](https://get.vial.today/)
- **Underglow RGB** (Glow variant) via the IS31FL3737 over I2C — plain, breathe and
  rainbow modes with global hue/brightness, plus 5-level CIE-corrected "teeth"
  indicator LEDs on TIM4 PWM
- **Layer status LEDs** (teeth): left on LOWER, right on RAISE, both on ADJUST
- **STM32 ROM DFU** bootloader entry from the firmware (stock QMK `stm32_dfu.c`
  style)
- **ZSA firmware25 default keymap** (`DEFAULT`/`LOWER`/`RAISE`/`ADJUST`),
  incl. Caps Word on the ZSA `CW` key

## Building

Requires the Rust toolchain and `flip-link` in `$PATH`:

```sh
rustup target add thumbv7em-none-eabihf
cargo install flip-link
```

Build a release binary:

```sh
cargo build --release --bin rmk-zsa-planck-ez          # base variant
cargo build --release --bin rmk-zsa-planck-ez-glow --features glow   # Glow variant
```

Then produce a `.bin` file:

```sh
rust-objcopy -O binary target/thumbv7em-none-eabihf/release/rmk-zsa-planck-ez-glow \
  rmk-zsa-planck-ez-glow.bin
```

## Flashing

Put the Planck EZ into DFU mode (hold `Esc` while plugging in, or press the
reset button nested beneath the spacebar), then flash with `dfu-util`:

```sh
dfu-util -d 0483:df11 -a 0 -s 0x08000000:leave -D rmk-zsa-planck-ez-glow.bin
```

Flashing is safe: the STM32 system-memory DFU bootloader lives in unmodifiable
system ROM and cannot be corrupted by a firmware image.

### flash.sh

`flash.sh` builds the selected variant, converts it to a `.bin`, and flashes it
in one step:

```sh
./flash.sh                    # base variant
./flash.sh glow               # Glow variant
```

The second argument is a comma-separated list of full cargo feature names;
`glow` is added automatically for the Glow variant. Keymap persistence is built
in, so no `storage` flag is needed.

The script requires `rust-objcopy` and `dfu-util` in `$PATH`, or run it inside
`nix develop`.

## Dependencies

| Crate                                | Role                                         |
| ------------------------------------ | -------------------------------------------- |
| [rmk](https://github.com/rmk-rs/rmk) | Keyboard framework (key scanning, HID, Vial) |
| [embassy-stm32](https://embassy.dev) | Async HAL for STM32F303                      |

## Acknowledgements

This port builds on prior work on
[jpds/rmk-zsa-voyager](https://github.com/jpds/rmk-zsa-voyager/), an RMK port for
another ZSA keyboard. Its STM32F303 RMK setup, warm-boot USB handling and flash
storage layout laid the groundwork for this firmware. Thanks to jpds for the
upstream work that made it possible.

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or
[MIT](LICENSE-MIT) at your option.

---

This project is not affiliated with or endorsed by ZSA Technology Labs, Inc.
