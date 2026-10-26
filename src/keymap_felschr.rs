//! Keymap ported from the user's Oryx layout (Planck EZ, layout `bvBwP`,
//! revision `NowGRX`, compiled with Oryx QMK 25.0):
//! <https://configure.zsa.io/planck-ez/layouts/bvBwP/latest/0>
//!
//! The Planck EZ matrix is 8 rows x 6 cols: rows 0-3 are the left half, rows
//! 4-7 the right (wired half of ~ rows). Physical bottom-row grid positions
//! map to matrix coordinates per QMK `LAYOUT_planck_1x2uC` (11 physical keys;
//! grid5 is the 2u space, matrix cell (3,5) is unpopulated):
//!    grid 0,1,2 -> matrix 3,0-3,2 | grid 3,4,5 -> matrix 7,3-7,5
//!    grid 6,7,8 -> matrix 7,0-7,2 | grid 9,10  -> matrix 3,3,3,4
//!
//! Layers (matching Oryx layer positions): 0 Base, 1 Lower, 2 Raise,
//! 3 Adjust, 4 Nav.
//!
//! Oryx's hold-only keys (tap action unset) become tap-hold actions with
//! `Action::No` as the tap. Oryx config `permissiveHold` + `holdOnOtherKeyPress`
//! is reflected by the default morse profile mode in `main.rs`.

use rmk::types::action::{EncoderAction, KeyAction};
use rmk::{a, encoder, k, layer, light, mo, mt, shifted};

pub(crate) const COL: usize = 6;
pub(crate) const ROW: usize = 8;
pub(crate) const NUM_LAYER: usize = 5;
pub(crate) const NUM_ENCODER: usize = 1; // padding for the optional PB12/PB13 encoder

/// Hold-only modifier key (Oryx "hold action", tap = nothing).
macro_rules! mh {
    (MEH) => {
        ::rmk::types::action::KeyAction::TapHold(
            ::rmk::types::action::Action::No,
            ::rmk::types::action::Action::Modifier(
                ::rmk::types::modifier::ModifierCombination::new_from_vals(true, true, true, false, false, false, false, false),
            ),
            ::core::primitive::u8::MAX,
        )
    };
    (ALL) => {
        ::rmk::types::action::KeyAction::TapHold(
            ::rmk::types::action::Action::No,
            ::rmk::types::action::Action::Modifier(
                ::rmk::types::modifier::ModifierCombination::new_from_vals(true, true, true, true, false, false, false, false),
            ),
            ::core::primitive::u8::MAX,
        )
    };
    ($m: ident) => {
        ::rmk::types::action::KeyAction::TapHold(
            ::rmk::types::action::Action::No,
            ::rmk::types::action::Action::Modifier(::rmk::types::modifier::ModifierCombination::$m),
            ::core::primitive::u8::MAX,
        )
    };
}

#[rustfmt::skip]
pub const fn get_felschr_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    // Bottom-row matrix columns (grid -> matrix per LAYOUT_planck_1x2uC):
    //   [3,0(g0)] [3,1(g1)] [3,2(g2)] [3,3(g9)] [3,4(g10)] [3,5(unused)]
    //   [7,0(g6)] [7,1(g7)] [7,2(g8)] [7,3(g3)] [7,4(g4)]  [7,5(g5=space)]
    [
        // Layer 0: Base
        layer!([
        [k!(Tab), k!(Q), k!(W), k!(E), k!(R), k!(T)],
        [k!(Escape), k!(A), k!(S), k!(D), k!(F), k!(G)],
        [k!(LShift), k!(Z), k!(X), k!(C), k!(V), k!(B)],
        [mh!(ALL), k!(LCtrl), k!(LAlt), k!(RCtrl), mh!(MEH), a!(No)],
        [k!(Y), k!(U), k!(I), k!(O), k!(P), k!(Backspace)],
        [k!(H), k!(J), k!(K), k!(L), k!(Semicolon), k!(Quote)],
        [k!(N), k!(M), k!(Comma), k!(Dot), k!(Slash), mt!(Enter, ::rmk::types::modifier::ModifierCombination::RSHIFT)],
        [mo!(2), mo!(4), k!(RAlt), k!(LGui), mo!(1), k!(Space)]
        ]),
        layer!([
        // Layer 1: Lower
        [shifted!(Grave), shifted!(Kc1), shifted!(Kc2), shifted!(Kc3), shifted!(Kc4), shifted!(Kc5)],
        [k!(Delete), k!(F1), k!(F2), k!(F3), k!(F4), k!(F5)],
        [a!(Transparent), k!(F7), k!(F8), k!(F9), k!(F10), k!(F11)],
        [a!(Transparent), a!(Transparent), a!(Transparent), k!(AudioVolUp), k!(MediaPlayPause), a!(No)],
        [shifted!(Kc6), shifted!(Kc7), shifted!(Kc8), shifted!(Kc9), shifted!(Kc0), a!(Transparent)],
        [k!(F6), shifted!(Minus), shifted!(Equal), shifted!(LeftBracket), shifted!(RightBracket), shifted!(Backslash)],
        [k!(F12), k!(NonusHash), k!(NonusBackslash), k!(Home), k!(End), a!(Transparent)],
        [a!(Transparent), a!(Transparent), k!(AudioVolDown), a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
        layer!([
        // Layer 2: Raise
        [k!(Grave), k!(Kc1), k!(Kc2), k!(Kc3), k!(Kc4), k!(Kc5)],
        [k!(Delete), k!(F1), k!(F2), k!(F3), k!(F4), k!(F5)],
        [a!(Transparent), k!(F7), k!(F8), k!(F9), k!(F10), k!(F11)],
        [a!(Transparent), a!(Transparent), a!(Transparent), k!(AudioVolUp), k!(MediaPlayPause), a!(No)],
        [k!(Kc6), k!(Kc7), k!(Kc8), k!(Kc9), k!(Kc0), a!(Transparent)],
        [k!(F6), k!(Minus), k!(Equal), k!(LeftBracket), k!(RightBracket), k!(Backslash)],
        [k!(F12), k!(NonusHash), k!(NonusBackslash), k!(PageUp), k!(PageDown), a!(Transparent)],
        [a!(Transparent), k!(MediaNextTrack), k!(AudioVolDown), a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
        layer!([
        // Layer 3: Adjust
        [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
        [k!(Delete), a!(Transparent), a!(No) /* AU_ON */, a!(No) /* AU_OFF */, a!(No) /* AU_TOGG */, a!(Transparent)],
        [a!(Transparent), a!(Transparent), a!(No) /* MU_ON */, a!(No) /* MU_OFF */, a!(No) /* MU_TOGG */, a!(Transparent)],
        [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
        [a!(Transparent), ::rmk::types::action::KeyAction::Single(::rmk::types::action::Action::User(0)), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
        [a!(Transparent), light!(RgbTog), light!(RgbVai), light!(RgbVad), a!(Transparent), ::rmk::types::action::KeyAction::Single(::rmk::types::action::Action::User(1))],
        [a!(Transparent), light!(RgbModeForward), light!(RgbHui), light!(RgbHud), a!(Transparent), a!(Transparent)],
        [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
        layer!([
        // Layer 4: Nav
        [a!(Transparent), k!(MouseAccel0), k!(MouseBtn1), k!(MouseUp), k!(MouseBtn2), a!(Transparent)],
        [a!(Transparent), k!(MouseAccel1), k!(MouseLeft), k!(MouseDown), k!(MouseRight), a!(Transparent)],
        [a!(Transparent), k!(MouseAccel2), k!(MouseWheelDown), k!(MouseBtn3), k!(MouseWheelUp), a!(Transparent)],
        [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
        [a!(Transparent), k!(End), k!(Up), k!(Home), a!(Transparent), a!(Transparent)],
        [a!(Transparent), k!(Left), k!(Down), k!(Right), a!(Transparent), a!(Transparent)],
        [a!(Transparent), k!(PageDown), k!(Application), k!(PageUp), a!(Transparent), a!(Transparent)],
        [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)]
        ]),
    ]
}

#[rustfmt::skip]
#[allow(dead_code)]
pub const fn get_felschr_encoder_map() -> [[EncoderAction; NUM_ENCODER]; NUM_LAYER] {
    [
        [encoder!(k!(AudioVolUp), k!(AudioVolDown))],
        [encoder!(k!(AudioVolUp), k!(AudioVolDown))],
        [encoder!(k!(AudioVolUp), k!(AudioVolDown))],
        [encoder!(k!(AudioVolUp), k!(AudioVolDown))],
        [encoder!(k!(AudioVolUp), k!(AudioVolDown))],
    ]
}
