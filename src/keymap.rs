//! Default Planck EZ keymap (ZSA firmware25 `keymaps/default`), matrix-indexed.
//!
//! The Planck EZ matrix is 8 rows x 6 cols: rows 0-3 are the left half, rows 4-7
//! the right (wired half of ~ rows). Physical 4x12 grid positions map to matrix
//! coordinates per ZSA's `LAYOUT_planck_1x2uC` (see `vial.json`).
//!
//! Layer 1 = LOWER, layer 2 = RAISE, layer 3 = ADJUST. Unimplemented ZSA
//! actions (DB_TOGG, MIDI/audio/one-shot keys) map to `No`.
use rmk::types::action::{EncoderAction, KeyAction};
use rmk::{a, encoder, k, kbctrl, layer, light, mo, shifted};

pub(crate) const COL: usize = 6;
pub(crate) const ROW: usize = 8;

pub(crate) const NUM_LAYER: usize = 4;
pub(crate) const NUM_ENCODER: usize = 1; // padding for the optional PB12/PB13 encoder


pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    // Layer 1 = LOWER, layer 2 = RAISE, layer 3 = ADJUST. Unimplemented ZSA
    // actions (DB_TOGG, MIDI/audio/one-shot keys) map to `No`.
    [
        // Layer 0: DEFAULT
        layer!([
        [k!(Tab), k!(Q), k!(W), k!(E), k!(R), k!(T)],
        [k!(Escape), k!(A), k!(S), k!(D), k!(F), k!(G)],
        [k!(LShift), k!(Z), k!(X), k!(C), k!(V), k!(B)],
        [kbctrl!(CapsWordToggle), k!(LCtrl), k!(LAlt), k!(Up), k!(Right), a!(No)],
        [k!(Y), k!(U), k!(I), k!(O), k!(P), k!(Backspace)],
        [k!(H), k!(J), k!(K), k!(L), k!(Semicolon), k!(Quote)],
        [k!(N), k!(M), k!(Comma), k!(Dot), k!(Slash), k!(Enter)],
        [mo!(2), k!(Left), k!(Down), k!(LGui), mo!(1), k!(Space)]
        ]),
        layer!([
        // Layer 1: LOWER
        [k!(Grave), shifted!(Kc1), shifted!(Kc2), shifted!(Kc3), shifted!(Kc4), shifted!(Kc5)],
        [k!(Delete), k!(F1), k!(F2), k!(F3), k!(F4), k!(F5)],
        [a!(Transparent), k!(F7), k!(F8), k!(F9), k!(F10), k!(F11)],
        [a!(Transparent), a!(Transparent), a!(Transparent), k!(AudioVolUp), k!(MediaPlayPause), a!(No)],
        [shifted!(Kc6), shifted!(Kc7), shifted!(Kc8), shifted!(Kc9), shifted!(Kc0), k!(Backspace)],
        [k!(F6), shifted!(Minus), shifted!(Equal), shifted!(LeftBracket), shifted!(RightBracket), shifted!(Backslash)],
        [k!(F12), shifted!(Grave), shifted!(Backslash), k!(Home), k!(End), a!(Transparent)],
        [a!(Transparent), k!(MediaNextTrack), k!(AudioVolDown), a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
        layer!([
        // Layer 2: RAISE
        [k!(Grave), k!(Kc1), k!(Kc2), k!(Kc3), k!(Kc4), k!(Kc5)],
        [k!(Delete), k!(F1), k!(F2), k!(F3), k!(F4), k!(F5)],
        [a!(Transparent), k!(F7), k!(F8), k!(F9), k!(F10), k!(F11)],
        [a!(Transparent), a!(Transparent), a!(Transparent), k!(AudioVolUp), k!(MediaPlayPause), a!(No)],
        [k!(Kc6), k!(Kc7), k!(Kc8), k!(Kc9), k!(Kc0), k!(Backspace)],
        [k!(F6), k!(Minus), k!(Equal), k!(LeftBracket), k!(RightBracket), k!(Backslash)],
        [k!(F12), k!(Grave), k!(Backslash), k!(PageUp), k!(PageDown), a!(Transparent)],
        [a!(Transparent), k!(MediaNextTrack), k!(AudioVolDown), a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
        layer!([
        // Layer 3: ADJUST
        [a!(Transparent), ::rmk::types::action::KeyAction::Single(::rmk::types::action::Action::User(1)), a!(No) /* DB_TOGG */, light!(RgbTog), light!(RgbModeForward), light!(RgbHui)],
        [a!(Transparent), a!(Transparent), a!(No) /* MU_NEXT */, a!(No) /* AU_ON */, a!(No) /* AU_OFF */, a!(No) /* AG_NORM */],
        [a!(Transparent), a!(No) /* AU_PREV */, a!(No) /* AU_NEXT */, a!(No) /* MU_ON */, a!(No) /* MU_OFF */, a!(No) /* MI_ON */],
        [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(No)],
        [light!(RgbHud), light!(RgbSai), light!(RgbSad), light!(RgbVai), light!(RgbVad), k!(Delete)],
        [a!(No) /* AG_SWAP */, a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
        [a!(No) /* MI_OFF */, a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
        [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
    ]
}

#[rustfmt::skip]
#[allow(dead_code)]
pub const fn get_default_encoder_map() -> [[EncoderAction; NUM_ENCODER]; NUM_LAYER] {
    [
        [encoder!(k!(AudioVolUp), k!(AudioVolDown))],
        [encoder!(k!(AudioVolUp), k!(AudioVolDown))],
        [encoder!(k!(AudioVolUp), k!(AudioVolDown))],
        [encoder!(k!(AudioVolUp), k!(AudioVolDown))],
    ]
}